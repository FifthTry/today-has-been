diesel::table! {
    users (id) {
        id -> BigInt,
        mobile_number -> BigInt,
        user_name -> Text,
        time_zone -> Nullable<Text>,
        language -> Nullable<Text>,
        subscription_type -> Nullable<Text>,
        subscription_end_time -> Nullable<Timestamptz>,
        customer_id -> Nullable<Text>,
        access_token -> Nullable<Text>,
        created_on -> Timestamptz,
        updated_on -> Timestamptz,
    }
}
