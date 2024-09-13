#[ft_sdk::processor]
fn subscribe_free_trial(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(access_token): ft_sdk::Query<"access_token">,
) -> ft_sdk::processor::Result {
    let user = common::get_user_from_access_token(&mut conn, &access_token)?;

    if does_subscription_exists_for_user(&mut conn, user.id)? {
        call_gupshup_callback_service(&user, false)?;
        return ft_sdk::processor::temporary_redirect("/free-trial-failure/");
    }

    subscribe_free_trial_for_user(&mut conn, user.id)?;

    call_gupshup_callback_service(&user, true)?;

    ft_sdk::processor::temporary_redirect("/free-trial-success/")
}

fn does_subscription_exists_for_user(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
) -> Result<bool, ft_sdk::Error> {
    use common::schema::subscriptions;
    use diesel::prelude::*;

    let subscription = subscriptions::table
        .filter(subscriptions::user_id.eq(user_id))
        .select(subscriptions::id)
        .first::<i64>(conn)
        .optional()?;

    Ok(subscription.is_some())
}

fn subscribe_free_trial_for_user(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
) -> Result<(), ft_sdk::Error> {
    use std::ops::Add;

    let now = ft_sdk::env::now();

    let start_date = common::datetime_to_date_string(&now);
    let end_date = common::datetime_to_date_string(&now.add(chrono::Duration::days(
        common::DURATION_TO_EXPIRE_FREE_TRIAL_IN_DAYS,
    )));

    let new_subscription = common::NewSubscription {
        user_id,
        subscription_id: "".to_string(),
        start_date,
        end_date: end_date.to_string(),
        status: Some("active".to_string()),
        is_active: Some("Yes".to_string()),
        plan_type: Some(common::FREE_TRIAL_PLAN_NAME.to_string()),
        created_on: now,
        updated_on: now,
    };

    new_subscription.insert_into_subscriptions(conn)?;

    common::update_user(
        conn,
        user_id,
        Some(common::FREE_TRIAL_PLAN_NAME.to_string()),
        Some(end_date),
    )?;

    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct GupshupUserData {
    phone: String,
    name: String,
}

#[derive(Debug, serde::Serialize)]
struct GupshupFields {
    event_name: String,
    event_time: String,
    user: GupshupUserData,
    free_trial_successful: bool,
    timezone: Option<String>,
}

pub(crate) fn call_gupshup_callback_service(
    user_data: &common::UserData,
    subscription_status: bool,
) -> Result<(), ft_sdk::Error> {
    let now = ft_sdk::env::now();
    let formatted_date = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    let fields = GupshupFields {
        event_name: "free_trial_started".to_string(),
        event_time: formatted_date,
        user: GupshupUserData {
            phone: user_data.mobile_number.to_string(),
            name: user_data.user_name.to_string(),
        },
        timezone: user_data.time_zone.clone(),
        free_trial_successful: subscription_status,
    };

    let body = serde_json::to_string(&fields)?;

    let request = http::Request::builder()
        .method("POST")
        .uri(common::GUPSHUP_CALLBACK_SERVICE_URL)
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
