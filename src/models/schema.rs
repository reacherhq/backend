table! {
	api_tokens (id) {
		id -> Int4,
		api_token -> Uuid,
		stripe_subscription_item -> Varchar,
		user_id -> Uuid,
	}
}

table! {
	api_usage_records (id) {
		id -> Int4,
		api_token_id -> Int4,
		method -> Varchar,
		endpoint -> Varchar,
		created_at -> Timestamp,
	}
}

table! {
	users (id) {
		id -> Uuid,
		stripe_customer -> Varchar,
	}
}

joinable!(api_tokens -> users (user_id));
joinable!(api_usage_records -> api_tokens (api_token_id));

allow_tables_to_appear_in_same_query!(api_tokens, api_usage_records, users,);
