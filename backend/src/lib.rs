extern crate core;
extern crate self as todayhasbeen;

use std::ops::Add;

mod add_post;
mod get_posts;
mod login;
mod logout;
mod register;
mod schema;
mod user;

const SECRET_KEY: &str = "SECRET_KEY";
const DURATION_TO_EXPIRE_ACCESS_TOKEN_IN_DAYS: i64 = 60;
const PROVIDER_ID: &str = "whatsapp";

const CUSTOM_ATTRIBUTE_ACCESS_TOKEN: &str = "access_token";
#[derive(serde::Deserialize, serde::Serialize)]
struct Custom {
    pub access_token: String,
    pub access_token_expires_at: i64,
}

impl Custom {
    pub(crate) fn new() -> Custom {
        let access_token = generate_access_token();
        let access_token_expires_at = ft_sdk::env::now()
            .add(chrono::Duration::days(
                DURATION_TO_EXPIRE_ACCESS_TOKEN_IN_DAYS,
            ))
            .timestamp_nanos_opt()
            .expect("Cannot convert time into nanos.");

        Custom {
            access_token,
            access_token_expires_at,
        }
    }
    pub(crate) fn from_provider_data(data: &ft_sdk::auth::ProviderData) -> Custom {
        serde_json::from_value(data.custom.clone()).expect("Cannot deserialize custom")
    }
    pub(crate) fn is_access_token_expired(&self) -> bool {
        let now = ft_sdk::env::now();
        let access_token_expires_at =
            chrono::DateTime::from_timestamp_nanos(self.access_token_expires_at);
        access_token_expires_at.le(&now)
    }
}

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

pub(crate) fn get_user_from_header(
    conn: &mut ft_sdk::Connection,
    headers: &http::HeaderMap,
) -> Result<(ft_sdk::auth::UserId, ft_sdk::auth::ProviderData), ft_sdk::Error> {
    // Extract access token from headers
    let access_token = get_access_token(headers)?;
    get_user_from_access_token(conn, access_token.as_str())
}

pub(crate) fn get_user_from_access_token(
    conn: &mut ft_sdk::Connection,
    access_token: &str,
) -> Result<(ft_sdk::auth::UserId, ft_sdk::auth::ProviderData), ft_sdk::Error> {
    let (user_id, provider_data) = ft_sdk::auth::provider::user_data_by_custom_attribute(
        conn,
        todayhasbeen::PROVIDER_ID,
        CUSTOM_ATTRIBUTE_ACCESS_TOKEN,
        access_token,
    )?;
    let custom = todayhasbeen::Custom::from_provider_data(&provider_data);

    // Check if token has expired
    if custom.is_access_token_expired() {
        return Err(
            ft_sdk::SpecialError::Unauthorised("Access token has expired!".to_string()).into(),
        );
    }

    Ok((user_id, provider_data))
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

fn generate_access_token() -> String {
    use rand_core::RngCore;

    let mut rand_buf: [u8; 16] = Default::default();
    ft_sdk::Rng::fill_bytes(&mut ft_sdk::Rng {}, &mut rand_buf);
    uuid::Uuid::new_v8(rand_buf).to_string()
}
