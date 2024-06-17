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
        access_token -> Text,
        created_on -> Timestamptz,
        updated_on -> Timestamptz,
    }
}

diesel::table! {
    posts (id) {
        id -> BigInt,
        user_id -> BigInt,
        post_content -> Nullable<Text>,
        media_url -> Nullable<Text>,
        created_on -> Timestamptz,
    }
}


diesel::joinable!(posts -> users (user_id));
