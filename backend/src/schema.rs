diesel::table! {
    posts (id) {
        id -> BigInt,
        user_id -> BigInt,
        post_content -> Nullable<Text>,
        media_url -> Nullable<Text>,
        created_on -> Timestamptz,
    }
}

diesel::joinable!(posts -> ft_sdk::auth::fastn_user (user_id));
