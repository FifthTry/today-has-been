#[ft_sdk::processor]
fn login(
    ft_sdk::Form(payload): ft_sdk::Form<Payload>,
) -> ft_sdk::processor::Result {

    todo!()
}




#[derive(Debug, serde::Deserialize)]
struct Payload {
    #[serde(rename = "userName")]
    user_name: String,
    #[serde(rename = "mobileNumber")]
    mobile_number: String
}