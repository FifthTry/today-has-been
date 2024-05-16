mod schema;

extern crate self as backend;

#[no_mangle]
pub extern "C" fn main_ft() {
    let req = ft_sdk::http::current_request();
    let resp = backend::route(req);
    ft_sdk::http::send_response(resp);
}


pub fn route(r: http::Request<bytes::Bytes>) -> http::Response<bytes::Bytes> {
    use backend::schema::posts;
    use diesel::prelude::*;

    ft_sdk::println!("r.uri.path:: {}", r.uri().path());

    let mut conn = ft_sdk::default_pg().unwrap();
    let in_ = ft_sdk::In::from_request(r).unwrap();

    diesel::insert_into(posts::table).values(&Post {
        userid: 1,
        postcontent: Some("Hello world".to_string()),
        mediaurl: None,
        createdon: in_.now,
    }).execute(&mut conn).unwrap();

    ft_sdk::json_response(serde_json::json!({"success": true}))
}


#[derive(diesel::Insertable)]
#[diesel(table_name = backend::schema::posts)]
pub struct Post {
    pub userid: i32,
    pub postcontent: Option<String>,
    pub mediaurl: Option<String>,
    pub createdon: chrono::DateTime<chrono::Utc>
}
