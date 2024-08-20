#[ft_sdk::data]
fn notify_free_trial_expired(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(secret_key): ft_sdk::Query<"secret_key">,
) -> ft_sdk::data::Result {
    let subscriptions = find_free_trial_expired(&mut conn?;

    todo!()
}


fn find_free_trial_expired(
    conn: &mut ft_sdk::Connection
) -> Result<Vec<common::Subscription>, ft_sdk::Error> {
    use common::schema::subscriptions;
    use diesel::prelude::*;

    let now = ft_sdk::env::now();

    todo!()
}