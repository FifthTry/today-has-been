// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "subscription_type"))]
    pub struct SubscriptionType;
}

diesel::table! {
    posts (postid) {
        postid -> Int4,
        userid -> Int4,
        postcontent -> Nullable<Text>,
        #[max_length = 255]
        mediaurl -> Nullable<Varchar>,
        createdon -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SubscriptionType;

    users (userid) {
        userid -> Int4,
        mobilenumber -> Int4,
        #[max_length = 100]
        username -> Varchar,
        #[max_length = 100]
        timezone -> Nullable<Varchar>,
        #[max_length = 100]
        language -> Nullable<Varchar>,
        subscriptiontype -> Nullable<SubscriptionType>,
        subscriptionendtime -> Nullable<Date>,
        #[max_length = 255]
        customerid -> Varchar,
        createdon -> Timestamp,
        updatedon -> Timestamp,
    }
}

diesel::joinable!(posts -> users (userid));

diesel::allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
