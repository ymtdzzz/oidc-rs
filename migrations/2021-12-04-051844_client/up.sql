-- Your SQL goes here
CREATE TABLE client (
  client_id VARCHAR(255) NOT NULL PRIMARY KEY,
  scope VARCHAR(255) NOT NULL,
  response_type VARCHAR(255) NOT NULL,
  redirect_uri VARCHAR(255) NOT NULL
);
