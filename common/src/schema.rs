diesel::table! {
    users (id) {
        id -> BigInt,
        mobile_number -> BigInt,
        user_name -> Text,
        time_zone -> Nullable<Text>,
        language -> Nullable<Text>,
        subscription_type -> Nullable<Text>,
        subscription_end_time -> Nullable<Text>,
        is_staff -> Bool,
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

diesel::table! {
    subscription_plans (id) {
        id -> BigInt,
        plan -> Text,
        price_id -> Text,
        amount -> Double,
        interval -> Text,
        trial_period_days -> Nullable<Int4>,
        discount -> Nullable<Text>,
        created_on -> Timestamptz,
    }
}

diesel::table! {
    subscriptions (id) {
        id -> BigInt,
        user_id -> BigInt,
        subscription_id -> Text,
        start_date -> Text,
        end_date -> Text,
        status -> Nullable<Text>,
        is_active -> Nullable<Text>,
        plan_type -> Nullable<Text>,
        created_on -> Timestamptz,
        updated_on -> Timestamptz,
    }
}

diesel::table! {
    stripe_logs (id) {
        id -> BigInt,
        event -> Nullable<Text>,
        response -> Nullable<Text>,
        created_on -> Timestamptz,
    }
}

diesel::joinable!(posts -> users (user_id));
diesel::joinable!(subscriptions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    posts,
    subscription_plans,
    subscriptions,
    stripe_logs,
);
