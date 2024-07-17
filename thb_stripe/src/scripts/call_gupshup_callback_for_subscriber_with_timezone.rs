// Currently all users are Indians. So timezone is +5:30. However we can check based on mobile
// number but this is not needed right now.
const TIMEZONE: &str = "+05:30";

#[ft_sdk::data]
fn call_gupshup_callback_for_subscriber_with_timezone(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(secret_key): ft_sdk::Query<"secret_key">,
) -> ft_sdk::data::Result {
    if common::SECRET_KEY.ne(&secret_key) {
        return Err(ft_sdk::SpecialError::Single(
            "secret_key".to_string(),
            "Invalid secret key.".to_string(),
        )
        .into());
    }

    let mut users = get_all_subscribed_users(&mut conn)?;

    call_gupshup_callback_service_and_update_table(&mut conn, &mut users)?;
    ft_sdk::data::json("Called gupshup successfully")
}

fn call_gupshup_callback_service_and_update_table(
    conn: &mut ft_sdk::Connection,
    users: &mut Vec<common::UserData>,
) -> Result<(), ft_sdk::Error> {
    use common::schema::users;
    use diesel::prelude::*;

    for user in users {
        let previous_timezone = user.time_zone.clone();
        // Set timezone
        user.time_zone = Some(TIMEZONE.to_string());
        ft_sdk::println!(
            "User: id: {}, name: {}, mobile_number: {}, previous_timezone: {:?}, subscription_type: {:?}",
            user.id,
            user.user_name,
            user.mobile_number,
            previous_timezone,
            user.subscription_type
        );

        // Call gupshup
        if let Some(ref subscription_type) = user.subscription_type {
            ft_sdk::println!("Calling gupshup");

            thb_stripe::charge_subscription::call_gupshup_callback_service(
                user,
                subscription_type,
                true,
            )?;

            if previous_timezone.is_none() {
                ft_sdk::println!("Updating table now");

                diesel::update(users::table)
                    .filter(users::id.eq(user.id))
                    .set(users::time_zone.eq(TIMEZONE))
                    .execute(conn)?;
            }

            ft_sdk::println!("Updated user successfully");
        }
    }
    Ok(())
}

fn get_all_subscribed_users(
    conn: &mut ft_sdk::Connection,
) -> Result<Vec<common::UserData>, ft_sdk::Error> {
    use common::schema::users;
    use diesel::prelude::*;

    let users = users::table
        .filter(users::subscription_type.is_not_null())
        .select(common::UserData::as_select())
        .load(conn)?;

    Ok(users)
}
