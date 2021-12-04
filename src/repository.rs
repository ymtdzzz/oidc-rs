use diesel::QueryDsl;
use diesel::{query_dsl::RunQueryDsl, MysqlConnection, QueryResult};

use crate::models::{AuthChallenge, AuthCode, Client, Session};
use crate::schema::*;

pub fn create_client(new_client: Client, conn: &MysqlConnection) -> QueryResult<usize> {
    diesel::insert_into(client::table)
        .values(&new_client)
        .execute(conn)
}

pub fn find_client(client_id: &str, conn: &MysqlConnection) -> QueryResult<Client> {
    client::table.find(client_id).first(conn)
}

pub fn create_auth_challenge(
    new_auth_challenge: AuthChallenge,
    conn: &MysqlConnection,
) -> QueryResult<usize> {
    diesel::insert_into(auth_challenges::table)
        .values(&new_auth_challenge)
        .execute(conn)
}

pub fn find_auth_challenge(challenge: &str, conn: &MysqlConnection) -> QueryResult<AuthChallenge> {
    auth_challenges::table.find(challenge).first(conn)
}

pub fn delete_auth_challenge(challenge: String, conn: &MysqlConnection) -> QueryResult<usize> {
    diesel::delete(auth_challenges::table.find(challenge)).execute(conn)
}

pub fn create_session(new_session: Session, conn: &MysqlConnection) -> QueryResult<usize> {
    diesel::insert_into(session::table)
        .values(&new_session)
        .execute(conn)
}

pub fn find_session(session_id: &str, conn: &MysqlConnection) -> QueryResult<Session> {
    session::table.find(session_id).first(conn)
}

pub fn delete_session(session_id: String, conn: &MysqlConnection) -> QueryResult<usize> {
    diesel::delete(session::table.find(session_id)).execute(conn)
}

pub fn create_auth_code(new_auth_code: AuthCode, conn: &MysqlConnection) -> QueryResult<usize> {
    diesel::insert_into(auth_code::table)
        .values(&new_auth_code)
        .execute(conn)
}

pub fn find_auth_code(code: &str, conn: &MysqlConnection) -> QueryResult<AuthCode> {
    auth_code::table.find(code).first(conn)
}

pub fn delete_auth_code(code: String, conn: &MysqlConnection) -> QueryResult<usize> {
    diesel::delete(auth_code::table.find(code)).execute(conn)
}
