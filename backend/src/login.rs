#[ft_sdk::processor]
fn login(
    ft_sdk::Form(payload): ft_sdk::Form<Payload>,
) -> ft_sdk::processor::Result {

    ft_sdk::processor::json(serde_json::json!({"user_name": payload.user_name}))
}



#[derive(Debug, serde::Deserialize)]
struct Payload {
    #[serde(rename = "userName")]
    user_name: String,
    #[serde(rename = "mobileNumber")]
    mobile_number: String
}