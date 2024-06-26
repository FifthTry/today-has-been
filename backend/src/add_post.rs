use crate::Post;

#[ft_sdk::data]
fn add_post(
    mut conn: ft_sdk::Connection,
    headers: http::HeaderMap,
    ft_sdk::Form(payload): ft_sdk::Form<Payload>,
) -> ft_sdk::data::Result {
    let user = match todayhasbeen::get_user_from_header(&mut conn, &headers) {
        Ok(user) => user,
        Err(_) => {
            return ft_sdk::data::json(serde_json::json!({
                "success": false,
                "message": "Token expired"
            }))
        }
    };

    if !payload.is_valid() {
        return ft_sdk::data::json(serde_json::json!({
            "success": false,
            "message": "Please send mandatory fields"
        }));
    }
    let output = insert_post(&mut conn, user.id, payload)?;
    ft_sdk::data::json(serde_json::json!({
        "data": serde_json::to_value(output)?,
        "success": true,
        "message": "Post added successfully"
    }))
}

fn insert_post(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    payload: Payload,
) -> Result<Output, ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::posts;

    // Create a new Post object
    let new_post = NewPost {
        user_id,
        post_content: payload.get_post_content(),
        media_url: payload.get_post_image_url(),
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
            created_on: self.created_on.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct Output {
    #[serde(rename = "postid")]
    post_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "postcontent")]
    post_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mediaurl")]
    media_url: Option<String>,
    #[serde(rename = "createdon")]
    created_on: String,
}

#[derive(Debug, serde::Deserialize)]
struct Payload {
    post_content: Option<String>,
    media_url: Option<String>,
}

impl Payload {
    fn is_valid(&self) -> bool {
        if self.post_content.is_none() && self.media_url.is_none() {
            return false;
        }
        match self.post_content {
            Some(ref post_content) if post_content.is_empty() => return false,
            _ => {}
        }
        match self.media_url {
            Some(ref media_url) if media_url.is_empty() => return false,
            _ => {}
        }
        true
    }

    fn get_post_image_url(&self) -> Option<String> {
        match self.post_content {
            Some(ref content) if content.starts_with(GUPSHUP_WA_IMAGE_START_PATTERN) => {
                Some(content.to_string())
            }
            _ => self.media_url.clone(),
        }
    }

    fn get_post_content(&self) -> Option<String> {
        match self.post_content {
            Some(ref content) if content.starts_with(GUPSHUP_WA_IMAGE_START_PATTERN) => None,
            _ => self.post_content.clone(),
        }
    }
}

const GUPSHUP_WA_IMAGE_START_PATTERN: &str = "https://filemanager.gupshup.io/wa/";
