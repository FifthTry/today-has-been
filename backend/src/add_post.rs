#[ft_sdk::data]
fn add_post(
    mut conn: ft_sdk::Connection,
    headers: http::HeaderMap,
    ft_sdk::Form(payload): ft_sdk::Form<Payload>,
) -> ft_sdk::data::Result {
    let user = todayhasbeen::get_user_from_access_token(&mut conn, &headers)?;

    todo!()
}

#[derive(Debug, serde::Deserialize)]
struct Payload {
    post_content: String,
    media_url: String,
}
