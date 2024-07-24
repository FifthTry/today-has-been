#[ft_sdk::processor]
fn payment_link(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(customer_id): ft_sdk::Query<"customer_id">,
    ft_sdk::Host(host): ft_sdk::Host,
    ft_sdk::Mountpoint(mountpoint): ft_sdk::Mountpoint,
) -> ft_sdk::processor::Result {
    use std::str::FromStr;

    let setup_intent = {
        let client = ft_stripe::Client::new(common::STRIPE_SECRET_KEY);
        let mut setup_intent = ft_stripe::CreateSetupIntent::new();
        setup_intent.customer =
            Some(ft_stripe::CustomerId::from_str(customer_id.as_str()).unwrap());
        setup_intent.payment_method_types = Some(vec!["card".to_string()]);
        ft_stripe::SetupIntent::create(&client, setup_intent)?
    };

    let plans = common::get_subscription_plans()?;
    let user_data = common::get_user_from_customer_id(&mut conn, customer_id.as_str())?;

    let output = Output {
        return_url: format!(
            "https://{host}{mountpoint}charge/subscription/?customer_id={customer_id}"
        ),
        customer_id,
        client_secret: setup_intent.client_secret,
        stripe_public_key: common::STRIPE_PUBLIC_KEY.to_string(),
        plans,
        subscription_type: user_data.subscription_type,
    };

    ft_sdk::println!("output: {output:?}");
    ft_sdk::processor::json(output)
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
struct Output {
    customer_id: String,
    client_secret: Option<String>,
    stripe_public_key: String,
    plans: Vec<common::SubscriptionPlanUI>,
    return_url: String,
    subscription_type: Option<String>,
}