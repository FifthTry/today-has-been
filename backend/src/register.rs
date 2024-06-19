#[ft_sdk::data]
fn register(
    mut conn: ft_sdk::Connection,
    ft_sdk::Form(payload): ft_sdk::Form<Payload>,
) -> ft_sdk::data::Result {
    payload.validate()?;

    let output = match payload.get_user(&mut conn)? {
        Some(user) => user,
        None => payload.create_user(&mut conn)?,
    };

    ft_sdk::data::api_ok(output)
}

#[derive(Debug, serde::Deserialize)]
struct Payload {
    user_name: String,
    mobile_number: String,
    secret_key: String,
}

#[derive(Debug, serde::Serialize)]
struct Output {
    user_id: i64,
    mobile_number: String,
    user_name: String,
    time_zone: Option<String>,
    language: Option<String>,
    subscription_type: Option<String>,
    subscription_end_time: Option<String>,
    customer_id: Option<String>,
    access_token: String,
}

impl Payload {
    pub(crate) fn get_user(
        &self,
        conn: &mut ft_sdk::Connection,
    ) -> Result<Option<Output>, ft_sdk::Error> {
        let user_id = match ft_sdk::auth::provider::user_data_by_identity(
            conn,
            todayhasbeen::PROVIDER_ID,
            self.mobile_number.as_str(),
        ) {
            Ok((user_id, _)) => user_id,
            Err(ft_sdk::auth::UserDataError::NoDataFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        let session_id = update_session_if_expired(conn, &user_id)?;
        Ok(Some(self.to_output(user_id.0, &session_id.0)))
    }

    pub(crate) fn create_user(
        &self,
        conn: &mut ft_sdk::Connection,
    ) -> Result<Output, ft_sdk::Error> {
        let user_id = ft_sdk::auth::provider::create_user(
            conn,
            todayhasbeen::PROVIDER_ID,
            self.to_provider_data(),
        )?;

        let session_id = create_new_session(conn, &user_id, None)?;
        Ok(self.to_output(user_id.0, session_id.0.as_str()))
    }

    fn to_output(&self, user_id: i64, access_token: &str) -> Output {
        Output {
            user_id,
            mobile_number: self.mobile_number.clone(),
            user_name: self.user_name.clone(),
            time_zone: None,
            language: None,
            subscription_type: None,
            subscription_end_time: None,
            customer_id: None,
            access_token: access_token.to_string(),
        }
    }

    fn to_provider_data(&self) -> ft_sdk::auth::ProviderData {
        // This is a hack to use mobile number as value for auth
        let mobile_to_email = ft_sdk::auth::mobile_to_email(self.mobile_number.as_str());

        ft_sdk::auth::ProviderData {
            identity: self.mobile_number.to_string(),
            username: Some(self.user_name.to_string()),
            name: None,
            emails: vec![mobile_to_email.to_string()],
            verified_emails: vec![mobile_to_email.to_string()],
            profile_picture: None,
            custom: serde_json::Value::Null,
        }
    }

    fn validate(&self) -> Result<(), ft_sdk::Error> {
        let secret_key = todayhasbeen::SECRET_KEY;
        if secret_key.ne(&self.secret_key) {
            return Err(ft_sdk::SpecialError::Single(
                "secret_key".to_string(),
                "Invalid secret key.".to_string(),
            )
            .into());
        }

        let mut errors = std::collections::HashMap::new();
        self.validate_mobile_number(&mut errors)?;

        if !errors.is_empty() {
            return Err(ft_sdk::SpecialError::Multi(errors).into());
        }
        Ok(())
    }

    fn validate_mobile_number(
        &self,
        errors: &mut std::collections::HashMap<String, String>,
    ) -> Result<(), ft_sdk::Error> {
        let is_digit = self.mobile_number.chars().all(|c| c.is_digit(10));
        if is_digit {
            errors.insert(
                "mobile_number".to_string(),
                "Mobile number can only contain digits.".to_string(),
            );
        } else if self.mobile_number.len().le(&10) && self.mobile_number.len().gt(&12) {
            errors.insert(
                "mobile_number".to_string(),
                "Mobile number must be between 10 and 12 digits long.".to_string(),
            );
        }
        Ok(())
    }
}

/// This function checks if the session associated with the given user ID is expired.
/// If it is expired, it creates a new session with a specified expiration duration.
/// If it is not expired, it returns the existing session ID.
fn update_session_if_expired(
    conn: &mut ft_sdk::Connection,
    user_id: &ft_sdk::auth::UserId,
) -> Result<ft_sdk::auth::SessionID, ft_sdk::Error> {
    // Attempt to retrieve the current session ID for the given user ID.
    let existing_session_id = ft_sdk::auth::SessionID::from_user_id(conn, user_id).ok();
    create_new_session(conn, user_id, existing_session_id)
}

/// Creates a new session with a specified expiration duration.
fn create_new_session(
    conn: &mut ft_sdk::Connection,
    user_id: &ft_sdk::auth::UserId,
    session_id: Option<ft_sdk::auth::SessionID>,
) -> Result<ft_sdk::auth::SessionID, ft_sdk::Error> {
    // Define the duration for which the session should be valid.
    let session_expiration_duration = Some(chrono::Duration::days(todayhasbeen::DURATION_TO_EXPIRE_ACCESS_TOKEN_IN_DAYS));

    // Creating a new session if needed.
    Ok(ft_sdk::auth::provider::login_with_custom_session_expiration(
        conn,
        user_id,
        session_id,
        session_expiration_duration,
    )?)
}
