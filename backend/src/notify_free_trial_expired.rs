#[ft_sdk::data]
fn notify_free_trial_expired(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(secret_key): ft_sdk::Query<"secret_key">,
) -> ft_sdk::data::Result {
    // Check secret key
    if common::SECRET_KEY.ne(&secret_key) {
        return Err(ft_sdk::SpecialError::Single(
            "secret_key".to_string(),
            "Invalid secret key.".to_string(),
        )
        .into());
    }

    // Find free trial expired
    let data = find_free_trial_expired(&mut conn)?;

    if data.is_empty() {
        return ft_sdk::data::api_ok(data);
    }

    // Notify via gupshup
    gupshup_notify_free_trial_expired(&data)?;

    // Mark as inactive
    update_subscription_mark_inactive(&mut conn, &data)?;

    ft_sdk::data::api_ok(data)
}

#[derive(Debug, serde::Serialize)]
struct FreeTrialData {
    subscription_pk_id: i64,
    user_name: String,
    mobile_number: i64,
}

fn find_free_trial_expired(
    conn: &mut ft_sdk::Connection,
) -> Result<Vec<FreeTrialData>, ft_sdk::Error> {
    use common::schema::{subscriptions, users};
    use diesel::prelude::*;

    let now = common::datetime_to_date_string(&ft_sdk::env::now());

    let result = subscriptions::table
        .inner_join(users::table)
        .filter(subscriptions::plan_type.eq(common::FREE_TRIAL_PLAN_NAME))
        .filter(subscriptions::is_active.eq("Yes"))
        .filter(subscriptions::end_date.lt(now))
        .select((subscriptions::id, users::user_name, users::mobile_number))
        .load::<(i64, String, i64)>(conn)?;

    let result = result
        .into_iter()
        .map(
            |(subscription_pk_id, user_name, mobile_number)| FreeTrialData {
                subscription_pk_id,
                user_name,
                mobile_number,
            },
        )
        .collect();

    Ok(result)
}

fn update_subscription_mark_inactive(
    conn: &mut ft_sdk::Connection,
    data: &[FreeTrialData],
) -> Result<(), ft_sdk::Error> {
    use common::schema::subscriptions;
    use diesel::prelude::*;

    conn.transaction(|conn| {
        for d in data {
            diesel::update(subscriptions::table)
                .filter(subscriptions::id.eq(d.subscription_pk_id))
                .set((
                    subscriptions::is_active.eq("No"),
                    subscriptions::updated_on.eq(ft_sdk::env::now()),
                    subscriptions::status.eq("inactive"),
                ))
                .execute(conn)?;
        }

        Ok::<(), ft_sdk::Error>(())
    })?;

    Ok(())
}

fn gupshup_notify_free_trial_expired(data: &[FreeTrialData]) -> Result<(), ft_sdk::Error> {
    todo!()
}
