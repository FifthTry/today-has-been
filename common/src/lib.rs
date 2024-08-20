extern crate self as common;
pub mod schema;

pub const SECRET_KEY: &str = "SECRET_KEY";
pub const STRIPE_SECRET_KEY: &str = "STRIPE_SECRET_KEY";
pub const STRIPE_PUBLIC_KEY: &str = "STRIPE_PUBLIC_KEY";
pub const STRIPE_WEBHOOK_SECRET_KEY: &str = "STRIPE_WEBHOOK_SECRET_KEY";
pub const GUPSHUP_AUTHORIZATION: &str = "GUPSHUP_AUTHORIZATION";
pub const STRIPE_ANNUAL_PRICE_ID: &str = "STRIPE_ANNUAL_PRICE_ID";
pub const STRIPE_MONTHLY_PRICE_ID: &str = "STRIPE_MONTHLY_PRICE_ID";
pub const DURATION_TO_EXPIRE_ACCESS_TOKEN_IN_HOURS: i64 = 2;
pub const DURATION_TO_EXPIRE_FREE_TRIAL_IN_DAYS: i64 = 14;
pub const FREE_TRIAL_PLAN_NAME: &str = "Free";
pub const GUPSHUP_CALLBACK_SERVICE_URL: &str = "https://notifications.gupshup.io/notifications/callback/service/ipass/project/730/integration/19770066040f26502c05494f2";

#[derive(Debug, serde::Serialize, diesel::Selectable, diesel::Queryable)]
#[diesel(table_name = common::schema::users)]
pub struct UserData {
    pub id: i64,
    pub mobile_number: i64,
    pub user_name: String,
    pub time_zone: Option<String>,
    pub language: Option<String>,
    pub subscription_type: Option<String>,
    pub subscription_end_time: Option<String>,
    pub customer_id: Option<String>,
    pub access_token: String,
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub updated_on: chrono::DateTime<chrono::Utc>,
}

impl UserData {
    pub fn is_access_token_expired(&self) -> bool {
        use std::ops::Add;

        let now = ft_sdk::env::now();
        self.updated_on
            .add(chrono::Duration::hours(
                DURATION_TO_EXPIRE_ACCESS_TOKEN_IN_HOURS,
            ))
            .lt(&now)
    }
}

pub fn get_user_from_header(
    conn: &mut ft_sdk::Connection,
    headers: &http::HeaderMap,
) -> Result<UserData, ft_sdk::Error> {
    // Extract access token from headers
    let access_token = get_access_token(headers)?;

    get_user_from_access_token(conn, &access_token)
}

pub fn get_user_from_access_token(
    conn: &mut ft_sdk::Connection,
    access_token: &str,
) -> Result<UserData, ft_sdk::Error> {
    use crate::schema::users;
    use diesel::prelude::*;

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

pub fn get_user_from_customer_id(
    conn: &mut ft_sdk::Connection,
    customer_id: &str,
) -> Result<common::UserData, ft_sdk::Error> {
    use common::schema::users;
    use diesel::prelude::*;

    // Query user based on access_token
    let user = users::table
        .filter(users::customer_id.eq(customer_id))
        .select(common::UserData::as_select())
        .first(conn)?;

    // Check if token has expired
    if user.is_access_token_expired() {
        return Err(
            ft_sdk::SpecialError::Unauthorised("Access token has expired!".to_string()).into(),
        );
    }

    Ok(user)
}

fn get_access_token(headers: &http::HeaderMap) -> Result<String, ft_sdk::Error> {
    let auth_value = headers.get("Authorization").and_then(|header_value| {
        header_value.to_str().ok().and_then(|auth_value| {
            if let Some(auth_value) = auth_value
                .strip_prefix("Bearer ")
                .or(auth_value.strip_prefix("bearer "))
            {
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

pub fn datetime_to_date_string(datetime: &chrono::DateTime<chrono::Utc>) -> String {
    // Format DateTime<Utc> to 'Y-m-d' format
    let formatted_date = datetime.format("%Y-%m-%d").to_string();

    formatted_date
}

pub fn date_string_to_datetime(
    date_str: &str,
) -> Result<chrono::DateTime<chrono::Utc>, chrono::ParseError> {
    use chrono::TimeZone;

    let naive_date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    let datetime_utc = chrono::Utc.from_utc_date(&naive_date).and_hms(0, 0, 0);

    Ok(datetime_utc)
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SubscriptionPlanUI {
    pub id: i64,
    pub plan: String,
    pub price_id: String,
    pub amount: String,
    pub interval: String,
    pub trial_period_days: Option<String>,
    pub discount: Option<String>,
    pub created_on: chrono::DateTime<chrono::Utc>,
}
pub fn get_subscription_plans() -> Result<Vec<SubscriptionPlanUI>, ft_sdk::Error> {
    let subscription_plans = vec![
        SubscriptionPlanUI {
            id: 2,
            plan: "Annual".to_string(),
            price_id: common::STRIPE_ANNUAL_PRICE_ID.to_string(),
            amount: "48".to_string(),
            interval: "year".to_string(),
            trial_period_days: Some("14".to_string()),
            discount: Some("20%".to_string()),
            created_on: ft_sdk::env::now(),
        },
        SubscriptionPlanUI {
            id: 1,
            plan: "Monthly".to_string(),
            price_id: common::STRIPE_MONTHLY_PRICE_ID.to_string(),
            amount: "5".to_string(),
            interval: "month".to_string(),
            trial_period_days: Some("7".to_string()),
            discount: None,
            created_on: ft_sdk::env::now(),
        },
    ];

    Ok(subscription_plans)
}

#[derive(Debug, serde::Serialize, diesel::Insertable, diesel::AsChangeset)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = common::schema::subscriptions)]
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

impl NewSubscription {
    pub fn insert_into_subscriptions(
        self,
        conn: &mut ft_sdk::Connection,
    ) -> Result<(), ft_sdk::Error> {
        use common::schema::subscriptions;
        use diesel::prelude::*;

        diesel::insert_into(subscriptions::table)
            .values(self)
            .execute(conn)?;

        Ok(())
    }
}

#[derive(Debug, serde::Serialize, diesel::Selectable, diesel::Queryable)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = common::schema::subscriptions)]
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
    pub fn to_new_subscription(self) -> NewSubscription {
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

pub fn update_user(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    subscription_type: Option<String>,
    subscription_end_time: Option<String>,
) -> Result<(), ft_sdk::Error> {
    use common::schema::users;
    use diesel::prelude::*;

    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set((
            users::subscription_type.eq(subscription_type),
            users::subscription_end_time.eq(subscription_end_time),
        ))
        .execute(conn)?;

    Ok(())
}
