table! {
    auth_challenges (challenge) {
        challenge -> Varchar,
    }
}

table! {
    auth_code (code) {
        code -> Varchar,
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
    session,
);
