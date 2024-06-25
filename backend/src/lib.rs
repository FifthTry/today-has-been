extern crate core;
extern crate self as todayhasbeen;

use std::ops::Add;

mod add_post;
mod charge_subscription;
mod get_posts;
mod get_stripe_link;
mod login;
mod logout;
mod payment_link;
mod register;
mod schema;
mod stripe_webhooks;
mod user;

const SECRET_KEY: &str = "SECRET_KEY";
const STRIPE_SECRET_KEY: &str = "STRIPE_SECRET_KEY";
const STRIPE_PUBLIC_KEY: &str = "STRIPE_PUBLIC_KEY";
const DURATION_TO_EXPIRE_ACCESS_TOKEN_IN_DAYS: i64 = 60;
const GUPSHUP_CALLBACK_SERVICE_URL: &str = "https://notifications.gupshup.io/notifications/callback/service/ipass/project/730/integration/19770066040f26502c05494f2";
const GUPSHUP_AUTHORIZATION: &str = "1fe499df-e424-46a0-af05-802d97353d42";
const STRIPE_WEBHOOK_SECRET_KEY: &str = "STRIPE_WEBHOOK_SECRET_KEY";

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
) -> Result<UserData, ft_sdk::Error> {
    // Extract access token from headers
    let access_token = get_access_token(headers)?;

    get_user_from_access_token(conn, &access_token)
}

pub(crate) fn get_user_from_access_token(
    conn: &mut ft_sdk::Connection,
    access_token: &str,
) -> Result<UserData, ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::users;

    // Query user based on access_token
    let user = users::table
        .filter(users::access_token.eq(access_token))
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

pub(crate) fn get_user_from_customer_id(
    conn: &mut ft_sdk::Connection,
    customer_id: &str,
) -> Result<todayhasbeen::UserData, ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::users;

    // Query user based on access_token
    let user = users::table
        .filter(users::customer_id.eq(customer_id))
        .select(todayhasbeen::UserData::as_select())
        .first(conn)?;

    // Check if token has expired
    if user.is_access_token_expired() {
        return Err(
            ft_sdk::SpecialError::Unauthorised("Access token has expired!".to_string()).into(),
        );
    }

    Ok(user)
}

#[derive(Debug, serde::Serialize, diesel::Selectable, diesel::Queryable)]
#[diesel(table_name = todayhasbeen::schema::users)]
struct UserData {
    id: i64,
    mobile_number: i64,
    user_name: String,
    time_zone: Option<String>,
    language: Option<String>,
    subscription_type: Option<String>,
    subscription_end_time: Option<String>,
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

#[derive(Debug, serde::Serialize, diesel::Selectable, diesel::Queryable)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = todayhasbeen::schema::subscription_plans)]
#[serde(rename_all = "kebab-case")]
pub struct SubscriptionPlan {
    pub id: i64,
    pub plan: String,
    pub price_id: String,
    pub amount: f64,
    pub created_on: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, diesel::Insertable, diesel::AsChangeset)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = todayhasbeen::schema::subscriptions)]
pub struct NewSubscription {
    pub user_id: i64,
    pub subscription_id: String,
    pub start_date: String,
    pub end_date: String,
    pub status: Option<String>,
    pub is_active: Option<String>,
    pub plan_type: Option<String>,
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub updated_on: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, diesel::Selectable, diesel::Queryable)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = todayhasbeen::schema::subscriptions)]
pub struct Subscription {
    pub id: i64,
    pub user_id: i64,
    pub subscription_id: String,
    pub start_date: String,
    pub end_date: String,
    pub status: Option<String>,
    pub is_active: Option<String>,
    pub plan_type: Option<String>,
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub updated_on: chrono::DateTime<chrono::Utc>,
}

impl Subscription {
    pub(crate) fn to_new_subscription(self) -> NewSubscription {
        NewSubscription {
            user_id: self.user_id,
            subscription_id: self.subscription_id,
            start_date: self.start_date,
            end_date: self.end_date,
            status: self.status,
            is_active: self.is_active,
            plan_type: self.plan_type,
            created_on: self.created_on,
            updated_on: self.updated_on,
        }
    }
}

pub(crate) fn update_user(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    subscription_type: Option<String>,
    subscription_end_time: Option<String>,
) -> Result<(), ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::users;

    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set((
            users::subscription_type.eq(subscription_type),
            users::subscription_end_time.eq(subscription_end_time),
        ))
        .execute(conn)?;

    Ok(())
}

pub(crate) fn timestamp_to_date_string(timestamp: i64) -> String {
    use chrono::{TimeZone, Utc};

    // Convert Unix timestamp to chrono DateTime<Utc>
    let datetime_utc = Utc.timestamp_opt(timestamp, 0).unwrap();

    // Format DateTime<Utc> to 'Y-m-d' format
    let formatted_date = datetime_utc.format("%Y-%m-%d").to_string();

    formatted_date
}
