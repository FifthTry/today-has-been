extern crate self as thb_stripe;

mod charge_subscription;
mod get_stripe_link;
mod payment_link;
mod scripts;
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

impl SubscriptionPlan {
    pub fn from_subscription_plan_ui(
        subscription_plan: common::SubscriptionPlanUI,
    ) -> SubscriptionPlan {
        SubscriptionPlan {
            id: subscription_plan.id,
            plan: subscription_plan.plan,
            price_id: subscription_plan.price_id,
            amount: subscription_plan.amount.parse::<f64>().unwrap(),
            interval: subscription_plan.interval,
            trial_period_days: subscription_plan
                .trial_period_days
                .map(|x| x.parse::<i32>().unwrap()),
            discount: subscription_plan.discount,
            created_on: subscription_plan.created_on,
        }
    }
}

pub(crate) fn timestamp_to_date_string(timestamp: i64) -> String {
    use chrono::{TimeZone, Utc};
    // Convert Unix timestamp to chrono DateTime<Utc>
    let datetime_utc = Utc.timestamp_opt(timestamp, 0).unwrap();
    common::datetime_to_date_string(&datetime_utc)
}
