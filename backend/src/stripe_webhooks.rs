#[ft_sdk::processor]
fn stripe_webhooks(
    mut conn: ft_sdk::Connection,
    ft_sdk::Form(payload): ft_sdk::Form<String>,
    headers: http::HeaderMap,
    host: ft_sdk::Host
) -> ft_sdk::processor::Result {

    let stripe_signature = get_stripe_signature(&headers)?;

    let event = ft_stripe::Webhook::construct_event(
        payload.as_str(),
        stripe_signature.as_str(),
        todayhasbeen::STRIPE_WEBHOOK_SECRET_KEY
    )?;

    todo!()
}


fn get_stripe_signature(headers: &http::HeaderMap) -> Result<String, ft_sdk::Error> {
    let auth_value = headers.get("Stripe-Signature").and_then(|header| header.to_str().ok()).map(|v| v.to_string());
    auth_value.ok_or_else(|| {
        ft_sdk::SpecialError::Unauthorised("No Stripe-Signature header found.".to_string()).into()
    })
}