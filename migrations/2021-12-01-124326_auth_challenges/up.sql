-- Your SQL goes here
CREATE TABLE auth_challenges (
  challenge VARCHAR(255) NOT NULL PRIMARY KEY,
  client_id VARCHAR(255) NOT NULL,
  scope VARCHAR(255) NOT NULL,
  response_type VARCHAR(255) NOT NULL,
  redirect_uri VARCHAR(255) NOT NULL,
  state VARCHAR(255),
  nonce VARCHAR(255)
);

