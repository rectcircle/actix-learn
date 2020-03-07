table! {
    posts (id) {
        id -> Bigint,
        user_id -> Bigint,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}

table! {
    users (id) {
        id -> Bigint,
        name -> Text,
        hair_color -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
