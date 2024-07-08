#[ft_sdk::data]
fn user_timezone(
    mut conn: ft_sdk::Connection,
    cookie: ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    ft_sdk::Required(timezone): ft_sdk::Required<"timezone">,
) -> ft_sdk::data::Result {
    let access_token = cookie.0;

    match access_token {
        Some(access_token) => {
            let user = common::get_user_from_access_token(&mut conn, access_token.as_str())?;
            // insert timezone in users table
            update_user_timezone(&mut conn, user.id, timezone.as_str())?;
            ft_sdk::data::api_ok("Timezone updated successfully")
        }
        None => ft_sdk::data::api_error(std::collections::HashMap::from([(
            "error".to_string(),
            "User not logged in".to_string(),
        )])),
    }
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
