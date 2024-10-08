#[ft_sdk::processor]
fn user(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(date): ft_sdk::Query<"date", Option<String>>,
    cookie: ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    host: ft_sdk::Host,
    mountpoint: ft_sdk::Mountpoint,
) -> ft_sdk::processor::Result {
    ft_sdk::println!("Get user details:: {date:?}");
    let access_token = cookie.0;

    match access_token {
        Some(access_token) => {
            let user_data = get_user_data(&mut conn, mountpoint, access_token.as_str(), date);
            ft_sdk::println!("Get user_data:: {user_data:?}");
            match user_data {
                Ok(user_data) => Ok(ft_sdk::processor::json(user_data)?
                    .with_cookie(todayhasbeen::set_light_mode(&host)?)),
                Err(_) => Ok(ft_sdk::processor::json(UserData {
                    is_logged_in: false,
                    auth_url: "https://wa.me/919910807891?text=Hi".to_string(),
                    posts: vec![],
                    mobile_number: 0,
                    user_name: "".to_string(),
                    subscription_end_time: None,
                    subscription_type: None,
                    time_zone: None,
                    is_staff: false,
                    older_date_url: None,
                    newer_date_url: None,
                    random_date_url: None,
                })?
                .with_cookie(todayhasbeen::expire_session_cookie(&host)?)
                .with_cookie(todayhasbeen::set_light_mode(&host)?)),
            }
        }
        None => Ok(ft_sdk::processor::json(UserData {
            is_logged_in: false,
            auth_url: "https://wa.me/919910807891?text=Hi".to_string(),
            posts: vec![],
            mobile_number: 0,
            user_name: "".to_string(),
            subscription_end_time: None,
            subscription_type: None,
            time_zone: None,
            is_staff: false,
            older_date_url: None,
            newer_date_url: None,
            random_date_url: None,
        })?
        .with_cookie(todayhasbeen::set_light_mode(&host)?)),
    }
}

fn get_user_data(
    conn: &mut ft_sdk::Connection,
    ft_sdk::Mountpoint(mountpoint): ft_sdk::Mountpoint,
    access_token: &str,
    date: Option<String>,
) -> Result<UserData, ft_sdk::Error> {
    let user = common::get_user_from_access_token(conn, access_token)?;

    ft_sdk::println!("Get user:: {user:?}");
    let date = match date {
        Some(date) => Some(common::date_string_to_datetime(date.as_str())?),
        None => None,
    };

    let (posts, older_date, newer_date) = get_posts_for_latest_or_given_date(conn, user.id, date)?;

    ft_sdk::println!("Get posts:: {posts:?}");
    let mut post_data_hash: std::collections::HashMap<String, Vec<PostDataByDate>> =
        std::collections::HashMap::new();

    for post in posts {
        let date = common::datetime_to_date_string(&post.created_on);
        let post_by_date = PostDataByDate {
            time: post.created_on.time().to_string(),
            post: post.post_content,
            media_url: post.media_url,
        };

        match post_data_hash.get_mut(&date) {
            Some(posts) => posts.push(post_by_date),
            None => {
                post_data_hash.insert(date, vec![post_by_date]);
            }
        }
    }

    ft_sdk::println!("Reorganise posts::");

    let random_date_data = todayhasbeen::get_random_post_date_data(conn, user.id, None, 1)?;

    ft_sdk::println!("random_date_data:: {random_date_data:?}");

    Ok(UserData {
        is_logged_in: true,
        auth_url: format!("{mountpoint}logout/"),
        posts: post_data_hash
            .into_iter()
            .map(|(date, post_data_by_date)| PostData {
                date,
                data: post_data_by_date,
            })
            .collect(),
        mobile_number: user.mobile_number,
        user_name: user.user_name,
        subscription_end_time: user.subscription_end_time,
        subscription_type: user.subscription_type,
        time_zone: user.time_zone,
        is_staff: user.is_staff,
        older_date_url: older_date
            .map(|dt| format!("/?date={}", common::datetime_to_date_string(&dt))),
        newer_date_url: newer_date
            .map(|dt| format!("/?date={}", common::datetime_to_date_string(&dt))),
        random_date_url: random_date_data
            .map(|(_, dt, _, _)| format!("/?date={}", common::datetime_to_date_string(&dt))),
    })
}

pub fn get_posts_for_latest_or_given_date(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    date: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<
    (
        Vec<todayhasbeen::Post>,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
    ),
    ft_sdk::Error,
> {
    // Determine the date to use
    let date_to_use = match date {
        Some(d) => d,
        None => match get_latest_post_date(conn, user_id)? {
            Some(d) => d,
            None => return Ok((vec![], None, None)), // No posts found
        },
    };

    // Get the posts for the determined date
    let posts_for_date = get_posts_for_date(conn, user_id, date_to_use)?;

    ft_sdk::println!("Get posts_for_date:: {posts_for_date:?}");

    // Get the adjacent dates
    let (older_date, newer_date) = get_adjacent_dates(conn, user_id, date_to_use)?;

    ft_sdk::println!("Get adjacent_dates:: {older_date:?} {newer_date:?}");

    Ok((posts_for_date, older_date, newer_date))
}

// Helper function to get the latest post date
fn get_latest_post_date(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, ft_sdk::Error> {
    use common::schema::posts;
    use diesel::dsl::max;
    use diesel::prelude::*;

    let latest_date = posts::table
        .select(max(posts::created_on))
        .filter(posts::user_id.eq(user_id))
        .first::<Option<chrono::DateTime<chrono::Utc>>>(conn)
        .optional()?
        .flatten();

    Ok(latest_date)
}

// Helper function to get all posts for a specific date
fn get_posts_for_date(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    date: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<todayhasbeen::Post>, ft_sdk::Error> {
    use common::schema::posts;
    use diesel::prelude::*;

    let start_of_day = date.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_of_day = date.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();

    let posts = posts::table
        .select(todayhasbeen::Post::as_select())
        .filter(posts::user_id.eq(user_id))
        .filter(posts::created_on.ge(start_of_day))
        .filter(posts::created_on.le(end_of_day))
        .load::<todayhasbeen::Post>(conn)?;

    Ok(posts)
}

// Helper function to find the next and previous post dates
fn get_adjacent_dates(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    date: chrono::DateTime<chrono::Utc>,
) -> Result<
    (
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
    ),
    ft_sdk::Error,
> {
    use common::schema::posts;
    use diesel::prelude::*;

    let start_of_day = date.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

    let older_date = posts::table
        .select(posts::created_on)
        .filter(posts::user_id.eq(user_id))
        .filter(posts::created_on.lt(start_of_day))
        .order_by(posts::created_on.desc())
        .first::<chrono::DateTime<chrono::Utc>>(conn)
        .optional()?;

    let end_of_day = date.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();

    let newer_date = posts::table
        .select(posts::created_on)
        .filter(posts::user_id.eq(user_id))
        .filter(posts::created_on.gt(end_of_day))
        .order_by(posts::created_on.asc())
        .first::<chrono::DateTime<chrono::Utc>>(conn)
        .optional()?;

    Ok((older_date, newer_date))
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct UserData {
    is_logged_in: bool,
    auth_url: String,
    posts: Vec<PostData>,
    mobile_number: i64,
    user_name: String,
    subscription_end_time: Option<String>,
    subscription_type: Option<String>,
    time_zone: Option<String>,
    is_staff: bool,
    older_date_url: Option<String>,
    newer_date_url: Option<String>,
    random_date_url: Option<String>,
}

#[derive(serde::Serialize, Debug)]
struct PostData {
    date: String,
    data: Vec<PostDataByDate>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct PostDataByDate {
    time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    post: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_url: Option<String>,
}
