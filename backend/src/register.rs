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
        let (user_id, mut provider_data) = match ft_sdk::auth::provider::user_data_by_identity(
            conn,
            todayhasbeen::PROVIDER_ID,
            self.mobile_number.as_str(),
        ) {
            Ok((user_id, provider_data)) => (user_id, provider_data),
            Err(ft_sdk::auth::UserDataError::NoDataFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        update_token_if_expired(conn, &mut provider_data, &user_id)?;

        let custom = todayhasbeen::Custom::from_provider_data(&provider_data);
        Ok(Some(self.to_output(user_id.0, &custom.access_token)))
    }

    pub(crate) fn create_user(
        &self,
        conn: &mut ft_sdk::Connection,
    ) -> Result<Output, ft_sdk::Error> {
        let custom = todayhasbeen::Custom::new();
        let ft_sdk::auth::UserId(user_id) = ft_sdk::auth::provider::create_user(
            conn,
            todayhasbeen::PROVIDER_ID,
            self.to_provider_data(&custom),
        )?;

        Ok(self.to_output(user_id, custom.access_token.as_str()))
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

    fn to_provider_data(&self, custom: &todayhasbeen::Custom) -> ft_sdk::auth::ProviderData {
        // This is a hack to use mobile number as value for auth
        let mobile_to_email = format!("{}@mobile.fifthtry.com", self.mobile_number);

        ft_sdk::auth::ProviderData {
            identity: self.mobile_number.to_string(),
            username: Some(self.user_name.to_string()),
            name: None,
            emails: vec![mobile_to_email.to_string()],
            verified_emails: vec![mobile_to_email.to_string()],
            profile_picture: None,
            custom: serde_json::to_value(custom).expect("Cannot convert custom to serde_json."),
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

fn update_token_if_expired(
    conn: &mut ft_sdk::Connection,
    provider_data: &mut ft_sdk::auth::ProviderData,
    user_id: &ft_sdk::auth::UserId,
) -> Result<(), ft_sdk::Error> {
    let custom = todayhasbeen::Custom::from_provider_data(provider_data);
    if !custom.is_access_token_expired() {
        return Ok(());
    }
    let new_custom = todayhasbeen::Custom::new();
    provider_data.custom = serde_json::to_value(new_custom).unwrap();
    ft_sdk::auth::provider::update_user(
        conn,
        todayhasbeen::PROVIDER_ID,
        user_id,
        provider_data.clone(),
        false,
    )?;
    Ok(())
}
