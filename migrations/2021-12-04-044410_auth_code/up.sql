-- Your SQL goes here
CREATE TABLE auth_code (
  code VARCHAR(255) NOT NULL PRIMARY KEY,
  client_id VARCHAR(255) NOT NULL,
  user_id VARCHAR(255) NOT NULL,
  scope VARCHAR(255) NOT NULL,
  nonce VARCHAR(255) NOT NULL
);
