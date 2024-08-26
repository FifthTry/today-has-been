#[ft_sdk::data]
fn login(
    ft_sdk::Query(access_token): ft_sdk::Query<"access_token">,
    ft_sdk::Query(date): ft_sdk::Query<"date", Option<String>>,
    ft_sdk::Query(next): ft_sdk::Query<"next", Option<String>>,
    host: ft_sdk::Host,
) -> ft_sdk::data::Result {
    let next = next.unwrap_or_else(|| {
        let date_query = date
            .filter(|date| !date.is_empty())
            .map(|date| format!("?date={}", date))
            .unwrap_or_default();

        format!("/{}", date_query)
    });

    ft_sdk::data::browser_redirect_with_cookie(
        next,
        todayhasbeen::set_session_cookie(access_token.as_str(), &host)?,
    )
}
