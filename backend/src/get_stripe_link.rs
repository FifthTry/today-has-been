#[ft_sdk::data]
fn get_stripe_link(
    mut conn: ft_sdk::Connection,
    headers: http::HeaderMap,
    host: ft_sdk::Host,
) -> ft_sdk::data::Result {
    let user = todayhasbeen::get_user_from_header(&mut conn, &headers)?;
    let customer_stripe_link =
    match get_customer_stripe_link(&mut conn, host, &user) {
        Ok(customer_stripe_link) => customer_stripe_link,
        Err(_) => Response {
            status: false,
            link: "".to_string(),
            message: Some("customerId not generated".to_string()),
        }
    };
    ft_sdk::data::api_ok(customer_stripe_link)
}

#[derive(serde::Serialize)]
struct Response {
    status: bool,
    link: String,
    message: Option<String>,
}

fn get_customer_stripe_link(
    conn: &mut ft_sdk::Connection,
    host: ft_sdk::Host,
    user: &todayhasbeen::UserData,
) -> Result<Response, ft_sdk::Error> {
    if let Some(ref customer_id) = user.customer_id {
        return Ok(Response {
            status: true,
            link: format!(
                "{}/api/payment/link/?customerId={customer_id}",
                host.without_port()
            ),
            message: None,
        });
    }

    if let Some(ref subscription_type) = user.subscription_type {
        return Ok(Response {
            status: false,
            link: "".to_string(),
            message: Some(format!(
                "You already subscribed with the {subscription_type} plan"
            )),
        });
    }

    let client = ft_stripe::Client::new(todayhasbeen::STRIPE_SECRET_KEY);
    let customer = {
        let mut create_customer = ft_stripe::CreateCustomer::new();
        create_customer.name = Some(user.user_name.as_str());
        create_customer.description = Some(user.user_name.as_str());
        let mobile_number_str = user.mobile_number.to_string();
        create_customer.phone = Some(mobile_number_str.as_str());
        ft_stripe::Customer::create(&client, create_customer)?
    };

    update_user_customer_id(conn, user.id, customer.id.as_str())?;

    Ok(Response {
        status: true,
        link: format!(
            "{}/api/payment/link/?customerId={}",
            host.without_port(),
            customer.id
        ),
        message: None,
    })
}

fn update_user_customer_id(
    conn: &mut ft_sdk::Connection,
    user_id: i64,
    customer_id: &str,
) -> Result<(), ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::users;

    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set(users::customer_id.eq(customer_id))
        .execute(conn)
        .map_err(|e| e.into())
        .map(|_| ())
}
