#[ft_sdk::data]
fn user(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(order): ft_sdk::Query<"order", Option<String>>,
    cookie: ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
) -> ft_sdk::data::Result {
    let session_id = cookie.0.clone();

    let order = order.unwrap_or("new".to_string());

    match session_id {
        Some(_) => {
            let posts = get_posts_by_order(&mut conn, cookie, order.as_str())?;
            ft_sdk::data::json(UserData {
                is_logged_in: true,
                auth_url: "/backend/logout/".to_string(),
                posts,
            })
        }
        None => ft_sdk::data::json(UserData {
            is_logged_in: false,
            auth_url: "https://wa.me/919910807891?text=Hi".to_string(),
            posts: vec![],
        }),
    }
}

fn get_posts_by_order(
    conn: &mut ft_sdk::Connection,
    cookie: ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    _order: &str,
) -> Result<Vec<PostData>, ft_sdk::Error> {
    let user_id = todayhasbeen::get_user_from_cookie(conn, cookie)?.id;
    let output = todayhasbeen::get_posts::get_posts_by_user_id(conn, user_id)?;
    let mut post_data_hash: std::collections::HashMap<String, Vec<PostDataByDate>> =
        std::collections::HashMap::new();

    for post in output {
        let date = post.created_on.date_naive().to_string();
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

    Ok(post_data_hash
        .into_iter()
        .map(|(date, post_data_by_date)| PostData {
            date,
            data: post_data_by_date,
        })
        .collect())
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct UserData {
    is_logged_in: bool,
    auth_url: String,
    posts: Vec<PostData>,
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
    post: Option<String>,
    media_url: Option<String>,
}
