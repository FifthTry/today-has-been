#[ft_sdk::data]
fn user_timezone(
    conn: ft_sdk::Connection,
    sid: ft_sdk::Required<"sid">,
    timezone: ft_sdk::Required<"timezone">,
) -> ft_sdk::data::Result {
    match user_timezone_(conn, sid, timezone) {
        Ok(_) => ft_sdk::data::api_ok("Timezone updated successfully"),
        Err(e) => ft_sdk::data::api_error(std::collections::HashMap::from([(
            "error".to_string(),
            e.to_string(),
        )])),
    }
}

fn user_timezone_(
    mut conn: ft_sdk::Connection,
    ft_sdk::Required(sid): ft_sdk::Required<"sid">,
    ft_sdk::Required(timezone): ft_sdk::Required<"timezone">,
) -> Result<(), ft_sdk::Error> {
    let user = common::get_user_from_access_token(&mut conn, sid.as_str())?;
    // insert timezone in users table
    update_user_timezone(&mut conn, user.id, timezone.as_str())?;
    Ok(())
}

fn update_user_timezone(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    timezone: &str,
) -> Result<(), ft_sdk::Error> {
    use common::schema::users;
    use diesel::prelude::*;

    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set(users::time_zone.eq(timezone))
        .execute(conn)?;

    Ok(())
}
