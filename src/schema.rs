table! {
    auth_challenges (challenge) {
        challenge -> Varchar,
        client_id -> Varchar,
        scope -> Varchar,
        response_type -> Varchar,
        redirect_uri -> Varchar,
    }
}

table! {
    auth_code (code) {
        code -> Varchar,
    }
}

table! {
    client (client_id) {
        client_id -> Varchar,
        scope -> Varchar,
        response_type -> Varchar,
        redirect_uri -> Varchar,
    }
}

table! {
    session (session_id) {
        session_id -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    auth_challenges,
    auth_code,
    client,
    session,
);
