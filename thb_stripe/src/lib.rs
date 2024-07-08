extern crate self as thb_stripe;

mod charge_subscription;
mod get_stripe_link;
mod payment_link;
mod stripe_webhooks;

fn get_user_from_customer_id(
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

#[derive(Debug, serde::Serialize, diesel::Selectable, diesel::Queryable)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = common::schema::subscription_plans)]
#[serde(rename_all = "kebab-case")]
pub struct SubscriptionPlan {
    pub id: i64,
    pub plan: String,
    pub price_id: String,
    pub amount: f64,
    pub interval: i32,
    pub trial_period_days: Option<i32>,
    pub discount: Option<String>,
    pub created_on: chrono::DateTime<chrono::Utc>,
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
