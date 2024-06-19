#[ft_sdk::data]
fn get_stripe_link(
    mut _conn: ft_sdk::Connection,
    _headers: http::HeaderMap,
) -> ft_sdk::data::Result {
    todo!()

    // let access_token = todayhasbeen::get_access_token(&headers)?;
    // let user = ft_sdk::auth::provider::user_data_by_custom_attribute()
    // let customer_id = stripe::create_customer(&mut conn, &ft_sdk::auth::SessionID(access_token), todayhasbeen::STRIPE_SECRET_KEY)?;
    //
    // ft_sdk::data::api_ok(output)
}