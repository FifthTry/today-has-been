#[ft_sdk::processor]
fn get_user_details(
    mut conn: ft_sdk::Connection,
    cookie: ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
) -> ft_sdk::processor::Result {
    let access_token = match cookie.0 {
        Some(access_token) => access_token,
        None => return ft_sdk::processor::json(Response {
            error_message: Some("Login required".to_string()),
            data: vec![],
        })
    };

    let user = common::get_user_from_access_token(&mut conn, access_token.as_str())?;
    if !has_access(&user) {
        return ft_sdk::processor::json(Response {
            error_message: Some("Access denied".to_string()),
            data: vec![],
        })
    }

    ft_sdk::processor::json(Response {
        error_message: None,
        data: get_all_users(&mut conn)?
    })
}



fn get_all_users(conn: &mut ft_sdk::Connection) -> Result<Vec<common::UserData>, ft_sdk::Error> {
    use common::schema::users;
    use diesel::prelude::*;

    let users = users::table
        .select(common::UserData::as_select())
        .load(conn)?;

    Ok(users)
}


fn has_access(user: &common::UserData) -> bool {
    common::ADMINS.contains(&user.mobile_number)
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Response {
    error_message: Option<String>,
    data: Vec<common::UserData>,
}

