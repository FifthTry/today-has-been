extern crate core;
extern crate self as todayhasbeen;

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

pub(crate) fn get_user_from_header(
    conn: &mut ft_sdk::Connection,
    headers: &http::HeaderMap,
) -> Result<ft_sdk::auth::UserId, ft_sdk::Error> {
    // Extract access token from headers
    let access_token = get_access_token(headers)?;
    get_user_from_access_token(conn, access_token.as_str())
}

fn get_user_from_access_token(
    conn: &mut ft_sdk::Connection,
    access_token: &str,
) -> Result<ft_sdk::auth::UserId, ft_sdk::Error> {
    let user_data = match ft_sdk::auth::ud_from_session_key(&ft_sdk::auth::SessionID(access_token.to_string()), conn)? {
        Some(v) => v,
        None => return Err(ft_sdk::SpecialError::NotFound(format!("User not found for given session ID: {access_token}")).into()),
    };

    Ok(ft_sdk::auth::UserId(user_data.id))
}


pub(crate) fn get_user_from_cookie(
    conn: &mut ft_sdk::Connection,
    cookie: ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
) -> Result<ft_sdk::auth::UserId, ft_sdk::Error> {
    let session_cookie = cookie.0.clone();
    let user_data = match ft_sdk::auth::ud(cookie, conn)? {
        Some(v) => v,
        None => return Err(ft_sdk::SpecialError::NotFound(format!("User not found for given session cookie: {session_cookie:?}")).into()),
    };

    Ok(ft_sdk::auth::UserId(user_data.id))
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
