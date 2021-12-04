use crate::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "auth_challenges"]
pub struct AuthChallenge {
    pub challenge: String,
}

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "session"]
pub struct Session {
    pub session_id: String,
}

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "auth_code"]
pub struct AuthCode {
    pub code: String,
}
