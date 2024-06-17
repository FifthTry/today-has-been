#[ft_sdk::data]
fn login(
    mut conn: ft_sdk::Connection,
    ft_sdk::Form(payload): ft_sdk::Form<Payload>,
) -> ft_sdk::data::Result {
    payload.validate()?;

    if let Some(user) = payload.get_user(&mut conn)? {
        return ft_sdk::data::api_ok(user);
    }



    todo!()
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
    mobile_number: i64,
    user_name: String,
    time_zone: Option<String>,
    language: Option<String>,
    subscription_type: Option<String>,
    subscription_end_time: Option<String>,
    customer_id: Option<String>,
    access_token: Option<String>,
}

impl Payload {
    pub(crate) fn get_user(
        &self,
        conn: &mut ft_sdk::Connection,
    ) -> Result<Option<Output>, ft_sdk::Error> {
        use diesel::prelude::*;
        use todayhasbeen::schema::users;

        match users::table
            .filter(users::mobile_number.eq(self.mobile_number.parse::<i64>().unwrap()))
            .select(UserData::as_select())
            .first(conn)
        {
            Ok(v) => Ok(Some(v.into_output())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    pub(crate) fn validate(&self) -> Result<(), ft_sdk::Error> {
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

#[derive(Debug, diesel::Selectable, diesel::Queryable)]
#[diesel(table_name = todayhasbeen::schema::users)]
struct UserData {
    id: i64,
    mobile_number: i64,
    user_name: String,
    time_zone: Option<String>,
    language: Option<String>,
    subscription_type: Option<String>,
    subscription_end_time: Option<chrono::DateTime<chrono::Utc>>,
    customer_id: Option<String>,
    access_token: Option<String>,
    created_on: chrono::DateTime<chrono::Utc>,
    updated_on: chrono::DateTime<chrono::Utc>,
}

impl UserData {
    fn into_output(self) -> Output {
        Output {
            user_id: self.id,
            mobile_number: self.mobile_number,
            user_name: self.user_name,
            time_zone: self.time_zone,
            language: self.language,
            subscription_type: self.subscription_type,
            subscription_end_time: self
                .subscription_end_time
                .map(|datetime| datetime.format("%Y-%m-%d").to_string()),
            customer_id: self.customer_id,
            access_token: self.access_token,
        }
    }
}
