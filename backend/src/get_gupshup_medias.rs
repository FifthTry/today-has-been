#[ft_sdk::data]
fn get_gupshup_medias(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(secret_key): ft_sdk::Query<"secret_key">,
    ft_sdk::Query(all): ft_sdk::Query<"all", Option<String>>,
) -> ft_sdk::data::Result {
    // Check secret key
    if common::SECRET_KEY.ne(&secret_key) {
        return Err(ft_sdk::SpecialError::Single(
            "secret_key".to_string(),
            "Invalid secret key.".to_string(),
        )
        .into());
    }

    // Get all gupshup medias or just last two days
    let all = all_flag(&all);

    // Get medias
    let medias = get_gupshup_medias_from_db(&mut conn, all)?;
    ft_sdk::data::api_ok(medias)
}



fn get_gupshup_medias_from_db(
    conn: &mut ft_sdk::Connection,
    all: bool,
) -> Result<Vec<GupshupMedia>, ft_sdk::Error> {
    use common::schema::posts;
    use diesel::prelude::*;

    if all {
        let gupshup_media_urls = posts::table
            .select((posts::id, posts::media_url))
            .filter(posts::media_url.is_not_null())
            .filter(posts::media_url.like("%filemanager.gupshup.io%"))
            .order_by(posts::created_on.desc())
            .load::<(i64, Option<String>)>(conn)?
            .into_iter()
            .filter_map(|( id, media_url )| media_url.map(|media_url| GupshupMedia { id, media_url }))
            .collect();

        Ok(gupshup_media_urls)
    } else {
        // Get last two days
        let date = ft_sdk::env::now() - chrono::Duration::days(2);
        let gupshup_media_urls = posts::table
            .select((posts::id, posts::media_url))
            .filter(posts::media_url.is_not_null())
            .filter(posts::media_url.like("%filemanager.gupshup.io%"))
            .filter(posts::created_on.gt(date))
            .order_by(posts::created_on.desc())
            .load::<(i64, Option<String>)>(conn)?
            .into_iter()
            .filter_map(|( id, media_url )| media_url.map(|media_url| GupshupMedia { id, media_url }))
            .collect();

        Ok(gupshup_media_urls)
    }
}

#[derive(Debug, serde::Serialize)]
struct GupshupMedia {
    id: i64,
    media_url: String,
}


fn all_flag(all: &Option<String>) -> bool {
    all.as_ref().unwrap_or(&"false".to_string()).eq("true")
}