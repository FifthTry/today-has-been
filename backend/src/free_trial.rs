#[ft_sdk::data]
fn subscribe_free_trial(
    mut conn: ft_sdk::Connection,
    headers: http::HeaderMap,
) -> ft_sdk::data::Result {
    let user = common::get_user_from_header(&mut conn, &headers)?;

    if does_subscription_exists_for_user(&mut conn, user.id)? {
        return Err(ft_sdk::SpecialError::Unauthorised("Already subscribed".to_string()).into());
    }

    subscribe_free_trial_for_user(&mut conn, user.id)?;

    ft_sdk::data::api_ok("Free trial subscribed!")
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
    let end_date = common::datetime_to_date_string(&now.add(chrono::Duration::hours(
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
