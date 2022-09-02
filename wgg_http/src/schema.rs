// @generated automatically by Diesel CLI.

diesel::table! {
    agg_ingredients (id) {
        id -> Integer,
        name -> Text,
        created_by -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    agg_ingredients_link (id, provider_id) {
        id -> Integer,
        provider_id -> Integer,
        provider_ingr_id -> Text,
    }
}

diesel::table! {
    agg_ingredients_links (id, provider_id) {
        id -> Integer,
        provider_id -> Integer,
        provider_ingr_id -> Text,
    }
}

diesel::table! {
    providers (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        email -> Text,
        username -> Text,
        hash -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users_tokens (id) {
        id -> Integer,
        user_id -> Integer,
        token -> Text,
        created -> Timestamp,
        expires -> Timestamp,
    }
}

diesel::joinable!(agg_ingredients -> users (created_by));
diesel::joinable!(agg_ingredients_link -> agg_ingredients (id));
diesel::joinable!(agg_ingredients_link -> providers (provider_id));
diesel::joinable!(agg_ingredients_links -> agg_ingredients (id));
diesel::joinable!(agg_ingredients_links -> providers (provider_id));
diesel::joinable!(users_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    agg_ingredients,
    agg_ingredients_link,
    agg_ingredients_links,
    providers,
    users,
    users_tokens,
);
