extern crate core;
extern crate self as todayhasbeen;

use std::ops::Add;

mod add_post;
mod login;
mod logout;
mod register;
mod schema;
mod user;

const SECRET_KEY: &str = "SECRET_KEY";
const DURATION_TO_EXPIRE_ACCESS_TOKEN_IN_DAYS: i64 = 60;

pub(crate) fn set_session_cookie(
    sid: &str,
    host: ft_sdk::Host,
) -> Result<http::HeaderValue, ft_sdk::Error> {
    let cookie = cookie::Cookie::build((ft_sdk::auth::SESSION_KEY, sid))
        .domain(host.without_port())
        .path("/")
        .max_age(cookie::time::Duration::seconds(34560000))
        .same_site(cookie::SameSite::Strict)
        .build();

    Ok(http::HeaderValue::from_str(cookie.to_string().as_str())?)
}

pub(crate) fn expire_session_cookie(
    host: ft_sdk::Host,
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

pub(crate) fn get_user_from_access_token(
    conn: &mut ft_sdk::Connection,
    headers: &http::HeaderMap,
) -> Result<UserData, ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::users;

    // Extract access token from headers
    let access_token = get_access_token(headers)?;

    // Query user based on access_token
    let user = users::table
        .filter(users::access_token.eq(&access_token))
        .select(UserData::as_select())
        .first(conn)?;

    // Check if token has expired
    if user.is_access_token_expired() {
        return Err(
            ft_sdk::SpecialError::Unauthorised("Access token has expired!".to_string()).into(),
        );
    }

    Ok(user)
}

#[derive(Debug, diesel::Selectable, diesel::Queryable)]
#[diesel(table_name = todayhasbeen::schema::users)]
struct UserData {
    id: i64,
    mobile_number: i64,
    user_name: String,
    time_zone: Option<String>,
    language: Option<String>,
    subscription_type: Option<String>,
    subscription_end_time: Option<chrono::DateTime<chrono::Utc>>,
    customer_id: Option<String>,
    access_token: String,
    created_on: chrono::DateTime<chrono::Utc>,
    updated_on: chrono::DateTime<chrono::Utc>,
}

impl UserData {
    pub(crate) fn is_access_token_expired(&self) -> bool {
        let now = ft_sdk::env::now();
        self.updated_on
            .add(chrono::Duration::days(
                DURATION_TO_EXPIRE_ACCESS_TOKEN_IN_DAYS,
            ))
            .lt(&now)
    }
}

fn get_access_token(headers: &http::HeaderMap) -> Result<String, ft_sdk::Error> {
    let auth_value = headers.get("Authorization").and_then(|header_value| {
        header_value.to_str().ok().and_then(|auth_value| {
            if let Some(auth_value) = auth_value.strip_prefix("Bearer ") {
                Some(auth_value.to_string())
            } else {
                None
            }
        })
    });
    auth_value.ok_or_else(|| {
        ft_sdk::SpecialError::Unauthorised("No Authorization header found.".to_string()).into()
    })
}
