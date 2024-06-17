#[ft_sdk::data]
fn get_posts(mut conn: ft_sdk::Connection, headers: http::HeaderMap) -> ft_sdk::data::Result {
    let user_id = todayhasbeen::get_user_from_header(&mut conn, &headers)?.0;
    let output = get_posts_by_user_id(&mut conn, user_id.0)?;
    ft_sdk::data::api_ok(output)
}

pub(crate) fn get_posts_by_user_id(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
) -> Result<Vec<Post>, ft_sdk::Error> {
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
    pub id: i64,
    pub user_id: i64,
    pub post_content: Option<String>,
    pub media_url: Option<String>,
    pub created_on: chrono::DateTime<chrono::Utc>,
}
