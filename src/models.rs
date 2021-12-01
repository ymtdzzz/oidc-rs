use crate::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[table_name = "auth_challenges"]
pub struct AuthChallenge {
    pub challenge: String,
}
