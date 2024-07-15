extern crate self as thb_stripe;

mod charge_subscription;
mod get_stripe_link;
mod payment_link;
mod stripe_webhooks;


#[derive(Debug, serde::Serialize, diesel::Selectable, diesel::Queryable)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = common::schema::subscription_plans)]
#[serde(rename_all = "kebab-case")]
pub struct SubscriptionPlan {
    pub id: i64,
    pub plan: String,
    pub price_id: String,
    pub amount: f64,
    pub interval: String,
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

pub(crate) fn update_user(
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

pub(crate) fn timestamp_to_date_string(timestamp: i64) -> String {
    use chrono::{TimeZone, Utc};
    // Convert Unix timestamp to chrono DateTime<Utc>
    let datetime_utc = Utc.timestamp_opt(timestamp, 0).unwrap();
    common::datetime_to_date_string(&datetime_utc)
}
