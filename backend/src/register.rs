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
    #[serde(rename = "userid")]
    user_id: i64,
    #[serde(rename = "mobilenumber")]
    mobile_number: String,
    #[serde(rename = "username")]
    user_name: String,
    timezone: Option<String>,
    language: Option<String>,
    #[serde(rename = "subscriptiontype")]
    subscription_type: Option<String>,
    #[serde(rename = "subscriptionendtime")]
    subscription_end_time: Option<String>,
    #[serde(rename = "customerid")]
    customer_id: Option<String>,
    access_token: String,
}

impl Output {
    fn from_user_data(user_data: common::UserData) -> Output {
        Output {
            user_id: user_data.id,
            mobile_number: user_data.mobile_number.to_string(),
            user_name: user_data.user_name,
            timezone: user_data.time_zone,
            language: user_data.language,
            subscription_type: user_data.subscription_type,
            subscription_end_time: user_data.subscription_end_time,
            customer_id: user_data.customer_id,
            access_token: user_data.access_token,
        }
    }
}

impl Payload {
    pub(crate) fn get_user(
        &self,
        conn: &mut ft_sdk::Connection,
    ) -> Result<Option<Output>, ft_sdk::Error> {
        use common::schema::users;
        use diesel::prelude::*;

        match users::table
            .filter(users::mobile_number.eq(self.mobile_number.parse::<i64>().unwrap()))
            .select(common::UserData::as_select())
            .first(conn)
        {
            Ok(mut user_data) => {
                update_token_if_expired(conn, &mut user_data)?;
                Ok(Some(Output::from_user_data(user_data)))
            }
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub(crate) fn create_user(
        &self,
        conn: &mut ft_sdk::Connection,
    ) -> Result<Output, ft_sdk::Error> {
        use common::schema::users;
        use diesel::prelude::*;

        let now = ft_sdk::env::now();
        let access_token = generate_access_token();

        let new_user = NewUserData {
            mobile_number: self.mobile_number.parse::<i64>().unwrap(),
            user_name: self.user_name.to_string(),
            time_zone: None,
            language: None,
            subscription_type: None,
            subscription_end_time: None,
            customer_id: None,
            access_token,
            created_on: now,
            updated_on: now,
        };

        let user_id = diesel::insert_into(users::table)
            .values(new_user.clone())
            .returning(users::id)
            .get_result::<i64>(conn)?;

        Ok(new_user.into_output(user_id))
    }

    pub(crate) fn validate(&self) -> Result<(), ft_sdk::Error> {
        let secret_key = common::SECRET_KEY;
        if secret_key.ne(&self.secret_key) {
            return Err(ft_sdk::SpecialError::Single(
                "secret_key".to_string(),
                "Invalid secret key.".to_string(),
            )
            .into());
        }

        let mut errors = std::collections::HashMap::new();
        self.validate_mobile_number(&mut errors)?;
        self.validate_user_name(&mut errors)?;

        if !errors.is_empty() {
            return Err(ft_sdk::SpecialError::Multi(errors).into());
        }
        Ok(())
    }

    fn validate_mobile_number(
        &self,
        errors: &mut std::collections::HashMap<String, String>,
    ) -> Result<(), ft_sdk::Error> {
        if self.mobile_number.is_empty() {
            errors.insert(
                "mobile_number".to_string(),
                "Mobile number cannot be empty.".to_string(),
            );
        }
        let is_digit = self.mobile_number.chars().all(|c| c.is_digit(10));
        if !is_digit {
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

    fn validate_user_name(
        &self,
        errors: &mut std::collections::HashMap<String, String>,
    ) -> Result<(), ft_sdk::Error> {
        if self.user_name.is_empty() {
            errors.insert(
                "user_name".to_string(),
                "Username cannot be empty.".to_string(),
            );
        }
        Ok(())
    }
}

#[derive(diesel::Insertable, Clone)]
#[diesel(treat_none_as_default_value = false)]
#[diesel(table_name = common::schema::users)]
struct NewUserData {
    mobile_number: i64,
    user_name: String,
    time_zone: Option<String>,
    language: Option<String>,
    subscription_type: Option<String>,
    subscription_end_time: Option<String>,
    customer_id: Option<String>,
    access_token: String,
    created_on: chrono::DateTime<chrono::Utc>,
    updated_on: chrono::DateTime<chrono::Utc>,
}

impl NewUserData {
    fn into_output(self, user_id: i64) -> Output {
        Output {
            user_id,
            mobile_number: self.mobile_number.to_string(),
            user_name: self.user_name,
            timezone: self.time_zone,
            language: self.language,
            subscription_type: self.subscription_type,
            subscription_end_time: self.subscription_end_time,
            customer_id: self.customer_id,
            access_token: self.access_token,
        }
    }
}

fn generate_access_token() -> String {
    use rand_core::RngCore;

    let mut rand_buf: [u8; 16] = Default::default();
    ft_sdk::Rng::fill_bytes(&mut ft_sdk::Rng {}, &mut rand_buf);
    uuid::Uuid::new_v8(rand_buf).to_string()
}

fn update_token_if_expired(
    conn: &mut ft_sdk::Connection,
    user: &mut common::UserData,
) -> Result<(), ft_sdk::Error> {
    use common::schema::users;
    use diesel::prelude::*;

    if !user.is_access_token_expired() {
        return Ok(());
    }

    let new_access_token = generate_access_token();

    let now = ft_sdk::env::now();
    diesel::update(users::table)
        .set((
            users::updated_on.eq(now),
            users::access_token.eq(&new_access_token),
        ))
        .filter(users::id.eq(user.id))
        .execute(conn)?;

    user.access_token = new_access_token;
    Ok(())
}
