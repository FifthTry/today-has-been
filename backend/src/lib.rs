extern crate core;
extern crate self as todayhasbeen;

mod add_post;
mod get_posts;
mod login;
mod logout;
mod register;
mod user;
mod user_timezone;

pub(crate) fn set_session_cookie(
    sid: &str,
    host: &ft_sdk::Host,
) -> Result<http::HeaderValue, ft_sdk::Error> {
    let cookie = cookie::Cookie::build((ft_sdk::auth::SESSION_KEY, sid))
        .domain(host.without_port())
        .path("/")
        .max_age(cookie::time::Duration::seconds(34560000))
        .same_site(cookie::SameSite::Strict)
        .build();

    Ok(http::HeaderValue::from_str(cookie.to_string().as_str())?)
}

pub(crate) fn set_light_mode(host: &ft_sdk::Host) -> Result<http::HeaderValue, ft_sdk::Error> {
    let cookie = cookie::Cookie::build(("fastn-dark-mode", "light"))
        .domain(host.without_port())
        .path("/")
        .max_age(cookie::time::Duration::seconds(34560000))
        .same_site(cookie::SameSite::Strict)
        .build();

    Ok(http::HeaderValue::from_str(cookie.to_string().as_str())?)
}

pub(crate) fn expire_session_cookie(
    host: &ft_sdk::Host,
) -> Result<http::HeaderValue, ft_sdk::Error> {
    let cookie = cookie::Cookie::build((ft_sdk::auth::SESSION_KEY, ""))
        .domain(host.without_port())
        .path("/")
        .expires(convert_now_to_offsetdatetime())
        .build();

    Ok(http::HeaderValue::from_str(cookie.to_string().as_str())?)
}

fn convert_now_to_offsetdatetime() -> cookie::time::OffsetDateTime {
    let now = ft_sdk::env::now();
    let timestamp = now.timestamp();
    let nanoseconds = now.timestamp_subsec_nanos();
    cookie::time::OffsetDateTime::from_unix_timestamp_nanos(
        (timestamp * 1_000_000_000 + nanoseconds as i64) as i128,
    )
    .unwrap()
}

pub(crate) fn date_string_to_datetime(
    date_str: &str,
) -> Result<chrono::DateTime<chrono::Utc>, chrono::ParseError> {
    use chrono::TimeZone;

    let naive_date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    let datetime_utc = chrono::Utc.from_utc_date(&naive_date).and_hms(0, 0, 0);

    Ok(datetime_utc)
}

#[derive(Debug, diesel::Selectable, diesel::Queryable, serde::Serialize)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = common::schema::posts)]
pub struct Post {
    #[serde(rename = "post_id")]
    pub id: i64,
    pub user_id: i64,
    pub post_content: Option<String>,
    pub media_url: Option<String>,
    pub created_on: chrono::DateTime<chrono::Utc>,
}

// Helper function to get a random post date
fn get_random_post_date_data(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    ignore_post_id: Option<i64>,
    min_length: usize,
) -> Result<
    Option<(
        i64,
        chrono::DateTime<chrono::Utc>,
        Option<String>,
        Option<String>,
    )>,
    ft_sdk::Error,
> {
    use common::schema::posts;
    use diesel::prelude::*;

    let dates: Vec<(
        i64,
        chrono::DateTime<chrono::Utc>,
        Option<String>,
        Option<String>,
    )> = posts::table
        .select((
            posts::id,
            posts::created_on,
            posts::media_url,
            posts::post_content,
        ))
        .filter(posts::user_id.eq(user_id))
        .load::<(
            i64,
            chrono::DateTime<chrono::Utc>,
            Option<String>,
            Option<String>,
        )>(conn)?;

    if dates.len() <= min_length {
        return Ok(None);
    }

    let mut current_post_id = ignore_post_id.unwrap_or(-1);
    let ignore_post_id = current_post_id;
    let mut random_date_data: (
        i64,
        chrono::DateTime<chrono::Utc>,
        Option<String>,
        Option<String>,
    ) = dates[0].clone();

    while ignore_post_id == current_post_id {
        let random_number = ft_sdk::env::random();
        let scaled_number = (random_number * dates.len() as f64).floor() as usize;
        random_date_data = dates[scaled_number].clone();
        (current_post_id, _, _, _) = random_date_data;
    }

    Ok(Some(random_date_data))
}
