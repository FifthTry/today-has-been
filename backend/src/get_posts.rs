#[ft_sdk::data]
fn get_posts(mut conn: ft_sdk::Connection, headers: http::HeaderMap) -> ft_sdk::data::Result {
    let user = todayhasbeen::get_user_from_header(&mut conn, &headers)?;
    let output = get_posts_by_user_id(&mut conn, user.id)?;
    ft_sdk::data::api_ok(output)
}

pub(crate) fn get_posts_by_user_id(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
) -> Result<Vec<todayhasbeen::Post>, ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::posts;

    let all_posts = posts::table
        .select(todayhasbeen::Post::as_select())
        .filter(posts::user_id.eq(user_id))
        .load::<todayhasbeen::Post>(conn)?;

    Ok(all_posts)
}
