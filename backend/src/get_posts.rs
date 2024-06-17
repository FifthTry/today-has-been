#[ft_sdk::data]
fn get_posts(
    mut conn: ft_sdk::Connection,
    headers: http::HeaderMap,
) -> ft_sdk::data::Result {
    let user = todayhasbeen::get_user_from_access_token(&mut conn, &headers)?;
    let output = get_posts_(&mut conn, user.id)?;
    ft_sdk::data::api_ok(output)
}

fn get_posts_(conn: &mut ft_sdk::Connection, user_id: i64) -> Result<Vec<Post>, ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::posts;

    let all_posts = posts::table
        .select(Post::as_select())
        .filter(posts::user_id.eq(user_id))
        .load::<Post>(conn)?;

    Ok(all_posts)
}

#[derive(Debug, diesel::Selectable, diesel::Queryable, serde::Serialize)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = todayhasbeen::schema::posts)]
pub struct Post {
    #[serde(rename = "post_id")]
    id: i64,
    user_id: i64,
    post_content: Option<String>,
    media_url: Option<String>,
    created_on: chrono::DateTime<chrono::Utc>,
}