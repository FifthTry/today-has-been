/*#[ft_sdk::processor]
fn stripe_webhooks(
    mut conn: ft_sdk::Connection,
    ft_sdk::Form(payload): ft_sdk::Form<String>,
    headers: http::HeaderMap,
) -> ft_sdk::processor::Result {
    let stripe_signature = get_stripe_signature(&headers)?;

    let event = ft_stripe::Webhook::construct_event(
        payload.as_str(),
        stripe_signature.as_str(),
        todayhasbeen::STRIPE_WEBHOOK_SECRET_KEY,
    )?;

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
            let subscription = get_subscription_from_event_obj(event.data.object);

            if let Some(subscription) = is_subscription_exists(&mut conn, subscription.id.as_str())?
            {
                let subscription_id = subscription.id;
                let mut new_subscription = subscription.to_new_subscription();

                // Todo: new_subscription.status = ft_stripe::SubscriptionStatus::InActive.to_string();
                new_subscription.status = Some("inactive".to_string());
                new_subscription.is_active = Some("No".to_string());
                new_subscription.updated_on = ft_sdk::env::now();

                todayhasbeen::update_user(&mut conn, new_subscription.user_id, None, None)?;
                update_subscription(&mut conn, subscription_id, new_subscription)?;
            }
            subscription
        }
        ft_stripe::EventType::CustomerSubscriptionUpdated => {
            let subscription = get_subscription_from_event_obj(event.data.object);

            if let Some(subscription_from_table) =
                is_subscription_exists(&mut conn, subscription.id.as_str())?
            {
                let subscription_id = subscription_from_table.id;

                // todo: check the database to know whether we have start_timestamp field
                let start_timestamp =
                    date_string_to_timestamp(subscription_from_table.start_date.as_str())?;
                if start_timestamp < subscription.current_period_start {
                    let start_date =
                        todayhasbeen::timestamp_to_date_string(subscription.current_period_start);
                    let end_date =
                        todayhasbeen::timestamp_to_date_string(subscription.current_period_end);
                    let mut new_subscription = subscription_from_table.to_new_subscription();
                    new_subscription.start_date = start_date;
                    new_subscription.end_date = end_date.clone();
                    new_subscription.status = Some(subscription.status.to_string());
                    new_subscription.updated_on = ft_sdk::env::now();

                    update_user_subscription_end_time(
                        &mut conn,
                        new_subscription.user_id,
                        end_date,
                    )?;
                    update_subscription(&mut conn, subscription_id, new_subscription)?;
                }
            }

            subscription
        }
        t => {
            return Err(
                ft_sdk::SpecialError::NotFound(format!("Received unknown event type: {t}")).into(),
            )
        }
    };

    insert_into_stripe_logs(
        &mut conn,
        StripeLog {
            event: Some(event.type_.to_string()),
            response: Some(serde_json::to_value(subscription).unwrap().to_string()),
            created_on: ft_sdk::env::now(),
        },
    )?;

    ft_sdk::processor::json("")
}

pub(crate) fn update_user_subscription_end_time(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    subscription_end_time: String,
) -> Result<(), ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::users;

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
) -> Result<Option<todayhasbeen::Subscription>, ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::subscriptions;

    Ok(subscriptions::table
        .filter(subscriptions::subscription_id.eq(subscription_id))
        .select(todayhasbeen::Subscription::as_select())
        .first(conn)
        .optional()?)
}

fn update_subscription(
    conn: &mut ft_sdk::Connection,
    id: i64,
    new_subscription: todayhasbeen::NewSubscription,
) -> Result<(), ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::subscriptions;

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
    Ok(todayhasbeen::date_string_to_datetime(date_str)?.timestamp())
}

#[derive(diesel::Insertable)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = todayhasbeen::schema::stripe_logs)]
pub struct StripeLog {
    pub event: Option<String>,
    pub response: Option<String>,
    pub created_on: chrono::DateTime<chrono::Utc>,
}

fn insert_into_stripe_logs(
    conn: &mut ft_sdk::Connection,
    stripe_log: StripeLog,
) -> Result<(), ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::stripe_logs;

    diesel::insert_into(stripe_logs::table)
        .values(stripe_log)
        .execute(conn)?;

    Ok(())
}
*/
