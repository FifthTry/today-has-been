#[ft_sdk::processor]
fn payment_link(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(customer_id): ft_sdk::Query<"customer_id">,
    host: ft_sdk::Host,
) -> ft_sdk::processor::Result {
    use std::str::FromStr;
    let setup_intent = {
        let client = ft_stripe::Client::new(todayhasbeen::STRIPE_SECRET_KEY);
        let mut setup_intent = ft_stripe::CreateSetupIntent::new();
        setup_intent.customer = Some(ft_stripe::CustomerId::from_str(customer_id.as_str()).unwrap());
        setup_intent.payment_method_types = Some(vec!["card".to_string()]);
        ft_stripe::SetupIntent::create(&client, setup_intent)?
    };


    let plans = get_subscription_plans(&mut conn)?;
    let user_data = get_user_from_customer_id(&mut conn, customer_id.as_str())?;

    ft_sdk::processor::json(Output {
        return_url: format!("{}/api/charge/subscription/?customer_id={customer_id}", host.without_port()),
        customer_id,
        client_secret: setup_intent.client_secret,
        stripe_key: todayhasbeen::STRIPE_SECRET_KEY.to_string(),
        plans,
        user_data,
    })
}


#[derive(serde::Serialize)]
struct Output {
    customer_id: String,
    client_secret: Option<String>,
    stripe_key: String,
    plans: Vec<SubscriptionPlan>,
    return_url: String,
    user_data: todayhasbeen::UserData
}


#[derive(Debug, serde::Serialize, diesel::Selectable, diesel::Queryable)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = todayhasbeen::schema::subscription_plans)]
pub struct SubscriptionPlan {
    pub id: i64,
    pub plan: String,
    pub price_id: String,
    pub amount: f64,
    pub createdon: chrono::DateTime<chrono::Utc>,
}

fn get_subscription_plans(conn: &mut ft_sdk::Connection) -> Result<Vec<SubscriptionPlan>, ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::subscription_plans;

    let subscription_plans = subscription_plans::table
        .select(SubscriptionPlan::as_select())
        .load(conn)?;
    Ok(subscription_plans)
}

pub(crate) fn get_user_from_customer_id(
    conn: &mut ft_sdk::Connection,
    customer_id: &str,
) -> Result<todayhasbeen::UserData, ft_sdk::Error> {
    use diesel::prelude::*;
    use todayhasbeen::schema::users;

    // Query user based on access_token
    let user = users::table
        .filter(users::customer_id.eq(customer_id))
        .select(todayhasbeen::UserData::as_select())
        .first(conn)?;

    // Check if token has expired
    if user.is_access_token_expired() {
        return Err(
            ft_sdk::SpecialError::Unauthorised("Access token has expired!".to_string()).into(),
        );
    }

    Ok(user)
}