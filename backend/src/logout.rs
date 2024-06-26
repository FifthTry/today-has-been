#[ft_sdk::processor]
fn logout(
    ft_sdk::Query(next): ft_sdk::Query<"next", Option<String>>,
    host: ft_sdk::Host,
) -> ft_sdk::processor::Result {
    let next = {
        let mut next = next.unwrap_or_default();
        if next.is_empty() {
            next = "/".to_string();
        }
        next
    };

    Ok(ft_sdk::processor::temporary_redirect(next)?
        .with_cookie(todayhasbeen::expire_session_cookie(&host)?)
        .with_cookie(todayhasbeen::set_light_mode(&host)?))
}
