#[ft_sdk::data]
fn add_post(
    mut conn: ft_sdk::Connection,
    headers: http::HeaderMap,
    ft_sdk::Form(payload): ft_sdk::Form<Payload>,
) -> ft_sdk::data::Result {
    let user = todayhasbeen::get_user_from_header(&mut conn, &headers)?;
    let output = insert_post(&mut conn, user.id, payload)?;
    ft_sdk::data::api_ok(output)
}

fn insert_post(conn: &mut ft_sdk::Connection, user_id: i64, payload: Payload) -> Result<Output, ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::posts;

    // Create a new Post object
    let new_post = NewPost {
        user_id,
        post_content: Some(payload.post_content),
        media_url: Some(payload.media_url),
        created_on: ft_sdk::env::now(),
    };

    // Insert into the database using Diesel
    let post_id = diesel::insert_into(posts::table)
        .values(&new_post.clone())
        .returning(posts::id)
        .get_result::<i64>(conn)?;

    Ok(new_post.into_output(post_id))
}

#[derive(diesel::Insertable, Clone)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = todayhasbeen::schema::posts)]
pub struct NewPost {
    user_id: i64,
    post_content: Option<String>,
    media_url: Option<String>,
    created_on: chrono::DateTime<chrono::Utc>,
}

impl NewPost {
    pub fn into_output(self, post_id: i64) -> Output {
        Output {
            post_id,
            post_content: self.post_content,
            media_url: self.media_url,
            created_on: self.created_on,
        }
    }
}

#[derive(serde::Serialize)]
pub struct Output {
    post_id: i64,
    post_content: Option<String>,
    media_url: Option<String>,
    created_on: chrono::DateTime<chrono::Utc>,
}


#[derive(Debug, serde::Deserialize)]
struct Payload {
    post_content: String,
    media_url: String,
}
