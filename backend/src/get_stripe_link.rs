#[ft_sdk::data]
fn get_stripe_link(
    mut conn: ft_sdk::Connection,
    headers: http::HeaderMap,
    host: ft_sdk::Host,
) -> ft_sdk::data::Result {
    let user = match todayhasbeen::get_user_from_header(&mut conn, &headers) {
        Ok(user) => user,
        Err(_) => {
            return ft_sdk::data::json(Output {
                status: false,
                link: "".to_string(),
                message: Some("Token expired".to_string()),
            })
        }
    };
    let customer_stripe_link = match get_customer_stripe_link(&mut conn, host, &user) {
        Ok(customer_stripe_link) => customer_stripe_link,
        Err(e) => {
            ft_sdk::println!("get_stripe_link e: {e:?}");

            Output {
                status: false,
                link: "".to_string(),
                message: Some("customerId not generated".to_string()),
            }
        }
    };
    ft_sdk::data::json(customer_stripe_link)
}

#[derive(serde::Serialize)]
struct Output {
    status: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

fn get_customer_stripe_link(
    conn: &mut ft_sdk::Connection,
    ft_sdk::Host(host): ft_sdk::Host,
    user: &todayhasbeen::UserData,
) -> Result<Output, ft_sdk::Error> {
    if let Some(ref customer_id) = user.customer_id {
        return Ok(Output {
            status: true,
            link: format!("https://{host}/payment/?customer_id={customer_id}",),
            message: None,
        });
    }

    if let Some(ref subscription_type) = user.subscription_type {
        return Ok(Output {
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

    Ok(Output {
        status: true,
        link: format!("https://{host}/payment/?customer_id={}", customer.id),
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
