#[ft_sdk::data]
fn change_media_urls(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(secret_key): ft_sdk::Query<"secret_key">,
    ft_sdk::Form(payload): ft_sdk::Form<Vec<Payload>>,
) -> ft_sdk::data::Result {
    // Check secret key
    if common::SECRET_KEY.ne(&secret_key) {
        return Err(ft_sdk::SpecialError::Single(
            "secret_key".to_string(),
            "Invalid secret key.".to_string(),
        )
        .into());
    }

    // Change gupshup medias
    change_media_urls_(&mut conn, payload)?;

    ft_sdk::data::api_ok("Media URL changed successfully.")
}

fn change_media_urls_(
    conn: &mut  ft_sdk::Connection,
    payload: Vec<Payload>,
) -> Result<(), ft_sdk::Error> {
    use common::schema::posts;
    use diesel::prelude::*;

    for payload in payload {
        diesel::update(posts::table.filter(posts::id.eq(payload.post_id)))
            .set(posts::media_url.eq(payload.media_url))
            .execute(conn)?;
    }

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct Payload {
    post_id: i64,
    media_url: String,
}