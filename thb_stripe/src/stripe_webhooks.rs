#[ft_sdk::raw_data]
fn stripe_webhooks(
    mut conn: ft_sdk::Connection,
    ft_sdk::Text(payload): ft_sdk::Text,
    headers: http::HeaderMap,
) -> ft_sdk::data::Result {
    let stripe_signature = get_stripe_signature(&headers)?;

    ft_sdk::println!("stripe_webhooks:: stripe_signature: {stripe_signature:?} payload: {payload}");

    let event = ft_stripe::Webhook::construct_event(
        payload.as_str(),
        stripe_signature.as_str(),
        common::STRIPE_WEBHOOK_SECRET_KEY,
    )?;

    ft_sdk::println!("stripe_webhooks:: event constructed: {event:?}");

    let subscription = match event.type_ {
        ft_stripe::EventType::CustomerSubscriptionCreated
        | ft_stripe::EventType::CustomerSubscriptionPaused
        | ft_stripe::EventType::CustomerSubscriptionPendingUpdateApplied
        | ft_stripe::EventType::CustomerSubscriptionPendingUpdateExpired
        | ft_stripe::EventType::CustomerSubscriptionResumed
        | ft_stripe::EventType::CustomerSubscriptionTrialWillEnd => {
            get_subscription_from_event_obj(event.data.object)
        }
        ft_stripe::EventType::CustomerSubscriptionDeleted => {
            ft_sdk::println!("stripe_webhooks:: subscription deleted");
            let subscription = get_subscription_from_event_obj(event.data.object);

            ft_sdk::println!("stripe_webhooks:: subscription deleted: {subscription:?}");

            if let Some(subscription) = is_subscription_exists(&mut conn, subscription.id.as_str())?
            {
                ft_sdk::println!("stripe_webhooks:: is_subscription_exists: {subscription:?}");
                let subscription_id = subscription.id;
                let mut new_subscription = subscription.to_new_subscription();

                // Todo: new_subscription.status = ft_stripe::SubscriptionStatus::InActive.to_string();
                new_subscription.status = Some("inactive".to_string());
                new_subscription.is_active = Some("No".to_string());
                new_subscription.updated_on = ft_sdk::env::now();

                ft_sdk::println!("stripe_webhooks:: new_subscription: {new_subscription:?}");

                common::update_user(&mut conn, new_subscription.user_id, None, None)?;
                ft_sdk::println!("stripe_webhooks:: update_user");
                update_subscription(&mut conn, subscription_id, new_subscription)?;
                ft_sdk::println!("stripe_webhooks:: update_subscription");
            }
            subscription
        }
        ft_stripe::EventType::CustomerSubscriptionUpdated => {
            ft_sdk::println!("stripe_webhooks:: subscription updated");
            let subscription = get_subscription_from_event_obj(event.data.object);

            ft_sdk::println!("stripe_webhooks:: subscription updated: {subscription:?}");

            if let Some(subscription_from_table) =
                is_subscription_exists(&mut conn, subscription.id.as_str())?
            {
                ft_sdk::println!(
                    "stripe_webhooks:: is_subscription_exists: {subscription_from_table:?}"
                );
                let subscription_id = subscription_from_table.id;

                // todo: check the database to know whether we have start_timestamp field
                let start_timestamp =
                    date_string_to_timestamp(subscription_from_table.start_date.as_str())?;
                if start_timestamp < subscription.current_period_start {
                    ft_sdk::println!(
                        "stripe_webhooks:: start_timestamp < current_period_start: {} < {}",
                        start_timestamp,
                        subscription.current_period_start
                    );
                    let start_date =
                        thb_stripe::timestamp_to_date_string(subscription.current_period_start);
                    let end_date =
                        thb_stripe::timestamp_to_date_string(subscription.current_period_end);
                    let mut new_subscription = subscription_from_table.to_new_subscription();
                    new_subscription.start_date = start_date;
                    new_subscription.end_date = end_date.clone();
                    new_subscription.status = Some(subscription.status.to_string());
                    new_subscription.updated_on = ft_sdk::env::now();

                    ft_sdk::println!("stripe_webhooks:: new_subscription: {new_subscription:?}");

                    update_user_subscription_end_time(
                        &mut conn,
                        new_subscription.user_id,
                        end_date,
                    )?;

                    ft_sdk::println!("stripe_webhooks:: update_user_subscription_end_time");
                    update_subscription(&mut conn, subscription_id, new_subscription)?;
                    ft_sdk::println!("stripe_webhooks:: update_subscription");
                }
            }

            subscription
        }
        t => {
            ft_sdk::println!("stripe_webhooks:: Received unknown event type: {t:?}");
            return Err(ft_sdk::SpecialError::NotFound(format!(
                "Received unknown event type: {t}"
            ))
            .into());
        }
    };

    ft_sdk::println!("stripe_webhooks:: subscription: {subscription:?}");

    insert_into_stripe_logs(
        &mut conn,
        StripeLog {
            event: Some(event.type_.to_string()),
            response: Some(serde_json::to_value(subscription).unwrap().to_string()),
            created_on: ft_sdk::env::now(),
        },
    )?;

    ft_sdk::println!("stripe_webhooks:: insert_into_stripe_logs");

    ft_sdk::data::json("")
}

pub(crate) fn update_user_subscription_end_time(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    subscription_end_time: String,
) -> Result<(), ft_sdk::Error> {
    use common::schema::users;
    use diesel::prelude::*;

    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set(users::subscription_end_time.eq(subscription_end_time))
        .execute(conn)?;

    Ok(())
}

fn get_subscription_from_event_obj(
    event_object: ft_stripe::EventObject,
) -> ft_stripe::Subscription {
    match event_object {
        ft_stripe::EventObject::Subscription(s) => s,
        t => unreachable!("Unknown event: {:?}", t),
    }
}

fn is_subscription_exists(
    conn: &mut ft_sdk::Connection,
    subscription_id: &str,
) -> Result<Option<common::Subscription>, ft_sdk::Error> {
    use common::schema::subscriptions;
    use diesel::prelude::*;

    Ok(subscriptions::table
        .filter(subscriptions::subscription_id.eq(subscription_id))
        .select(common::Subscription::as_select())
        .first(conn)
        .optional()?)
}

fn update_subscription(
    conn: &mut ft_sdk::Connection,
    id: i64,
    new_subscription: common::NewSubscription,
) -> Result<(), ft_sdk::Error> {
    use common::schema::subscriptions;
    use diesel::prelude::*;

    diesel::update(subscriptions::table)
        .filter(subscriptions::id.eq(id))
        .set(new_subscription)
        .execute(conn)?;

    Ok(())
}

fn get_stripe_signature(headers: &http::HeaderMap) -> Result<String, ft_sdk::Error> {
    let auth_value = headers
        .get("Stripe-Signature")
        .and_then(|header| header.to_str().ok())
        .map(|v| v.to_string());
    auth_value.ok_or_else(|| {
        ft_sdk::SpecialError::Unauthorised("No Stripe-Signature header found.".to_string()).into()
    })
}

fn date_string_to_timestamp(date_str: &str) -> Result<i64, chrono::ParseError> {
    Ok(common::date_string_to_datetime(date_str)?.timestamp())
}

#[derive(diesel::Insertable)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = common::schema::stripe_logs)]
pub struct StripeLog {
    pub event: Option<String>,
    pub response: Option<String>,
    pub created_on: chrono::DateTime<chrono::Utc>,
}

fn insert_into_stripe_logs(
    conn: &mut ft_sdk::Connection,
    stripe_log: StripeLog,
) -> Result<(), ft_sdk::Error> {
    use common::schema::stripe_logs;
    use diesel::prelude::*;

    diesel::insert_into(stripe_logs::table)
        .values(stripe_log)
        .execute(conn)?;

    Ok(())
}
