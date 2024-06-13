#[ft_sdk::data]
fn user(
    ft_sdk::Query(access_token): ft_sdk::Query<"access-token", Option<String>>,
    ft_sdk::Query(order): ft_sdk::Query<"order", Option<String>>,
    cookie: ft_sdk::Cookie<{ ft_sdk::auth::SESSION_KEY }>,
    host: ft_sdk::Host,
) -> ft_sdk::data::Result {
    let access_token = match access_token {
        Some(access_token) if !access_token.is_empty() => Some(access_token),
        _ => match cookie.0 {
            Some(access_token) => Some(access_token),
            None => None
        }
    };


    let order = order.unwrap_or("new".to_string());

    match access_token {
        Some(access_token) => {
            let posts = get_posts_by_order(access_token.as_str(), order.as_str());
            Ok(ft_sdk::data::json(UserData {is_logged_in: true, posts })?.with_cookie(session_cookie(access_token.as_str(), host)?))
        }
        None => ft_sdk::data::json(UserData {is_logged_in: false, posts: vec![] })
    }
}


fn get_posts_by_order(access_token: &str, _order: &str) -> Vec<PostData> {
    let get_posts = call_get_posts_api(access_token);
    let mut post_data_hash: std::collections::HashMap<String, Vec<PostDataByDate>> = std::collections::HashMap::new();
    for post in get_posts.data {
        let naive_date_time = string_to_naive_date_time(post.createdon.as_str());
        let date = naive_date_time.date().to_string();
        let post_by_date = PostDataByDate {
            time: naive_date_time.time().to_string(),
            post: post.postcontent,
            media_url: post.mediaurl,
        };
        match post_data_hash.get_mut(&date) {
            Some(posts) => posts.push(post_by_date),
            None => { post_data_hash.insert(date, vec![post_by_date]); }
        }
    }
    post_data_hash.into_iter().map(|(date, post_data_by_date)| PostData {date, data: post_data_by_date }).collect()
}





#[derive(serde::Deserialize, Debug)]
struct ApiResponse {
    status: bool,
    data: Vec<ApiPost>,
}

#[derive(serde::Deserialize, Debug)]
struct ApiPost {
    postid: i32,
    userid: i32,
    postcontent: Option<String>,
    mediaurl: Option<String>,
    createdon: String,
}

fn call_get_posts_api(token: &str) -> ApiResponse {
    let authorization_header = format!("Bearer {}", token);

    let url = ft_sdk::env::var("SITE_URL".to_string()).unwrap();

    let client = http::Request::builder();
    let request = client
        .method("GET")
        .uri(format!("{url}/api/v0.1/get/posts"))
        .header("Authorization", authorization_header)
        .header("Accept", "application/json; api-version=2.0")
        .header("Content-Type", "application/json")
        .header("User-Agent", "FifthTry")
        .body(bytes::Bytes::new()).unwrap();

    let response = ft_sdk::http::send(request).unwrap(); //todo: remove unwrap()

    if response.status().is_success() {
        let api_response: ApiResponse = serde_json::from_str(String::from_utf8_lossy(response.body()).to_string().as_str()).unwrap();
        // Extract the 'value' field from the JSON response
        ft_sdk::println!("Response: {:?}", api_response);

        return api_response;
    } else {
        ft_sdk::println!("Request failed with status: {}", response.status());
        todo!()
    }
}




#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct UserData {
    is_logged_in: bool,
    posts: Vec<PostData>,
}

#[derive(serde::Serialize, Debug)]
struct PostData {
    date: String,
    data: Vec<PostDataByDate>,
}

#[derive(serde::Serialize, Debug)]
struct PostDataByDate {
    time: String,
    post: Option<String>,
    media_url: Option<String>,
}



fn session_cookie(sid: &str, host: ft_sdk::Host) -> Result<http::HeaderValue, ft_sdk::Error> {
    let cookie = cookie::Cookie::build((ft_sdk::auth::SESSION_KEY, sid))
        .domain(host.without_port())
        .path("/")
        .max_age(cookie::time::Duration::seconds(34560000))
        .same_site(cookie::SameSite::Strict)
        .build();

    Ok(http::HeaderValue::from_str(cookie.to_string().as_str())?)
}


fn string_to_naive_date_time(date_time_str: &str) -> chrono::NaiveDateTime  {
    let format = "%Y-%m-%d %H:%M:%S";
    chrono::NaiveDateTime::parse_from_str(date_time_str, format).expect("Failed to parse date and time")
}