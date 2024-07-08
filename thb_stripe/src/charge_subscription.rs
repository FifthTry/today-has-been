#[ft_sdk::processor]
fn charge_subscription(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(customer_id): ft_sdk::Query<"customer_id">,
    ft_sdk::Query(price_id): ft_sdk::Query<"price_id">,
    ft_sdk::Query(setup_intent): ft_sdk::Query<"setup_intent", Option<String>>,
    ft_sdk::Query(redirect_status): ft_sdk::Query<"redirect_status">,
    host: ft_sdk::Host,
) -> ft_sdk::processor::Result {
    let user_data = thb_stripe::get_user_from_customer_id(&mut conn, customer_id.as_str())?;
    let plan_info = get_subscription_plan(&mut conn, price_id.as_str())?;

    let subscription = get_subscription_status(
        &mut conn,
        customer_id.as_str(),
        price_id.as_str(),
        setup_intent,
        redirect_status.as_str(),
        &user_data,
        &plan_info,
    )?;

    call_gupshup_callback_service(&user_data, &plan_info, &subscription)?;

    let url = format!("https://{host}/subscription/payment/?status=",);

    if subscription.status {
        ft_sdk::processor::temporary_redirect(format!("{url}success"))
    } else {
        ft_sdk::processor::temporary_redirect(format!("{url}failed"))
    }
}

fn call_gupshup_callback_service(
    user_data: &common::UserData,
    plan_info: &thb_stripe::SubscriptionPlan,
    subscription: &SubscriptionResult,
) -> Result<(), ft_sdk::Error> {
    let url = common::GUPSHUP_CALLBACK_SERVICE_URL;
    let now = ft_sdk::env::now();
    let formatted_date = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    let fields = GupshupFields {
        event_name: "stripe_payment_status".to_string(),
        event_time: formatted_date,
        user: GupshupUserData {
            phone: user_data.mobile_number.to_string(),
            name: user_data.user_name.to_string(),
        },
        payment_successful: subscription.status,
        subscription_type: plan_info.plan.to_string(),
    };

    let body = serde_json::to_string(&fields)?;

    let request = http::Request::builder()
        .method("POST")
        .uri(url)
        .header("Authorization", common::GUPSHUP_AUTHORIZATION)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(bytes::Bytes::from(body))?;

    match ft_sdk::http::send(request) {
        Ok(response) => {
            ft_sdk::println!(
                "call_gupshup_callback_service response: {} {}",
                response.status(),
                String::from_utf8_lossy(response.body())
            );
        }
        Err(e) => {
            ft_sdk::println!("call_gupshup_callback_service error: {e:?}");
        }
    };

    Ok(())
}

struct SubscriptionResult {
    status: bool,
    txid: Option<String>,
    msg: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct GupshupUserData {
    phone: String,
    name: String,
}

#[derive(Debug, serde::Serialize)]
struct GupshupFields {
    event_name: String,
    event_time: String, // Assuming dateTime is a String
    user: GupshupUserData,
    payment_successful: bool,  // Assuming status is a boolean
    subscription_type: String, // Assuming planInfo['plan'] is a String
}

fn get_subscription_status(
    conn: &mut ft_sdk::Connection,
    customer_id: &str,
    price_id: &str,
    setup_intent: Option<String>,
    redirect_status: &str,
    user_data: &common::UserData,
    plan_info: &thb_stripe::SubscriptionPlan,
) -> Result<SubscriptionResult, ft_sdk::Error> {
    use std::str::FromStr;

    let mut subscription = SubscriptionResult {
        status: false,
        txid: None,
        msg: None,
    };

    if redirect_status.eq("succeeded") && setup_intent.is_some() {
        let client = ft_stripe::Client::new(common::STRIPE_SECRET_KEY);
        let setup_intent_id =
            ft_stripe::SetupIntentId::from_str(setup_intent.unwrap().as_str()).unwrap();
        let setup_intent = ft_stripe::SetupIntent::retrieve(&client, &setup_intent_id, &[])?;
        if let Some(payment_method) = setup_intent.payment_method {
            let card_id = payment_method.id();

            //Todo: This condition look uneccessary
            if !customer_id.is_empty() || !price_id.is_empty() || !card_id.is_empty() {
                subscription = apply_customer_subscription(
                    conn,
                    customer_id,
                    price_id,
                    &card_id,
                    user_data,
                    &plan_info,
                );
            }
        }
    }
    Ok(subscription)
}

fn apply_customer_subscription(
    conn: &mut ft_sdk::Connection,
    customer_id: &str,
    price_id: &str,
    card: &ft_stripe::PaymentMethodId,
    user_data: &common::UserData,
    plan_info: &thb_stripe::SubscriptionPlan,
) -> SubscriptionResult {
    let subscription_id =
        apply_customer_subscription_(conn, customer_id, price_id, card, user_data, plan_info);
    match subscription_id {
        Ok(subscription_id) => SubscriptionResult {
            status: true,
            txid: Some(subscription_id.to_string()),
            msg: None,
        },
        Err(e) => SubscriptionResult {
            status: false,
            txid: None,
            msg: Some(format!("Error creating subscription: {e:?}")),
        },
    }
}

fn apply_customer_subscription_(
    conn: &mut ft_sdk::Connection,
    customer_id: &str,
    price_id: &str,
    card: &ft_stripe::PaymentMethodId,
    user_data: &common::UserData,
    plan_info: &thb_stripe::SubscriptionPlan,
) -> Result<ft_stripe::SubscriptionId, ft_sdk::Error> {
    use std::str::FromStr;

    let stripe_subscription = {
        let client = ft_stripe::Client::new(common::STRIPE_SECRET_KEY);

        let create_subscription = {
            let mut create_subscription_items = ft_stripe::CreateSubscriptionItems::new();
            create_subscription_items.price = Some(price_id.to_string());

            let mut create_subscription = ft_stripe::CreateSubscription::new(
                ft_stripe::CustomerId::from_str(customer_id).unwrap(),
            );

            create_subscription.trial_period_days = plan_info.trial_period_days.map(|v| v as u32);
            // The default for `missing_payment_method` is `cancel`.
            let trial_setting = ft_stripe::CreateSubscriptionTrialSettings::default();
            create_subscription.trial_settings = Some(trial_setting);

            create_subscription.items = Some(vec![create_subscription_items]);
            create_subscription.default_payment_method = Some(card.as_str());
            create_subscription.automatic_tax = Some(ft_stripe::CreateSubscriptionAutomaticTax {
                enabled: true,
                liability: None,
            });

            create_subscription
        };

        ft_stripe::Subscription::create(&client, create_subscription)?
    };

    let start_date =
        timestamp_to_date_string(stripe_subscription.current_period_start);
    let end_date = timestamp_to_date_string(stripe_subscription.current_period_end);

    let now = ft_sdk::env::now();

    let subscription = thb_stripe::NewSubscription {
        user_id: user_data.id,
        subscription_id: stripe_subscription.id.to_string(),
        start_date,
        end_date: end_date.clone(),
        status: Some(stripe_subscription.status.to_string()),
        is_active: Some("Yes".to_string()),
        plan_type: Some(plan_info.plan.to_string()),
        created_on: now,
        updated_on: now,
    };

    insert_into_subscriptions(conn, subscription)?;
    update_user(conn, user_data.id, Some(plan_info.plan.to_string()), Some(end_date))?;

    Ok(stripe_subscription.id)
}

fn insert_into_subscriptions(
    conn: &mut ft_sdk::Connection,
    subscription: thb_stripe::NewSubscription,
) -> Result<(), ft_sdk::Error> {
    use diesel::prelude::*;
    use common::schema::subscriptions;

    diesel::insert_into(subscriptions::table)
        .values(subscription)
        .execute(conn)?;

    Ok(())
}

fn get_subscription_plan(
    conn: &mut ft_sdk::Connection,
    price_id: &str,
) -> Result<thb_stripe::SubscriptionPlan, ft_sdk::Error> {
    use diesel::prelude::*;
    use common::schema::subscription_plans;

    let subscription_plan = subscription_plans::table
        .select(thb_stripe::SubscriptionPlan::as_select())
        .filter(subscription_plans::price_id.eq(price_id))
        .first(conn)?;
    Ok(subscription_plan)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("diesel error {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("diesel error {0}")]
    Stripe(#[from] ft_stripe::StripeError),
}


pub(crate) fn timestamp_to_date_string(timestamp: i64) -> String {
    use chrono::{TimeZone, Utc};
    // Convert Unix timestamp to chrono DateTime<Utc>
    let datetime_utc = Utc.timestamp_opt(timestamp, 0).unwrap();
    common::datetime_to_date_string(&datetime_utc)
}


pub(crate) fn update_user(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    subscription_type: Option<String>,
    subscription_end_time: Option<String>,
) -> Result<(), ft_sdk::Error> {
    use diesel::prelude::*;
    use common::schema::users;

    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set((
            users::subscription_type.eq(subscription_type),
            users::subscription_end_time.eq(subscription_end_time),
        ))
        .execute(conn)?;

    Ok(())
}
