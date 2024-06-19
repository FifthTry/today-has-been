#[ft_sdk::processor]
fn login(
    mut conn: ft_sdk::Connection,
    ft_sdk::Query(access_token): ft_sdk::Query<"access-token">,
    ft_sdk::Query(order): ft_sdk::Query<"order", Option<String>>,
    ft_sdk::Query(next): ft_sdk::Query<"next", Option<String>>,
    host: ft_sdk::Host,
) -> ft_sdk::processor::Result {
    let next = next.unwrap_or_else(|| {
        let order_query = order
            .filter(|order| !order.is_empty())
            .map(|order| format!("?order={}", order))
            .unwrap_or_default();

        format!("/{}", order_query)
    });

    let session_id = ft_sdk::auth::SessionID(access_token);

    Ok(
        ft_sdk::processor::temporary_redirect(next)?.with_cookie(ft_sdk::auth::set_session_cookie(&mut conn, session_id, host)?),
    )
}
