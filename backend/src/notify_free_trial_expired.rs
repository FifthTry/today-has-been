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
    let notified_data = gupshup_notify_free_trial_expired(&data)?;

    // Mark as inactive
    update_subscription_mark_inactive(&mut conn, &notified_data)?;

    ft_sdk::data::api_ok(Output {
        all_data: data,
        notified_data,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
struct Output {
    all_data: Vec<FreeTrialData>,
    notified_data: Vec<FreeTrialData>,
}

#[derive(Debug, Clone, serde::Serialize)]
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
}

fn gupshup_notify_free_trial_expired(
    data: &[FreeTrialData],
) -> Result<Vec<FreeTrialData>, ft_sdk::Error> {
    let now = ft_sdk::env::now();
    let formatted_date = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let mut notified_free_trial_expired = vec![];

    for d in data {
        let fields = GupshupFields {
            event_name: "free_trial_expired".to_string(),
            event_time: formatted_date.clone(),
            user: GupshupUserData {
                phone: d.mobile_number.to_string(),
                name: d.user_name.to_string(),
            },
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
                notified_free_trial_expired.push(d.clone())
            }
            Err(e) => {
                ft_sdk::println!("call_gupshup_callback_service error: {e:?}");
            }
        };
    }

    Ok(notified_free_trial_expired)
}
