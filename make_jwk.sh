#!/bin/sh
header='
{
  "kid": "12345",
  "alg": "RS256"
}'
jwk=$(pem-jwk public-key.pem)
# Add additional fields
jwk=$(echo '{"use":"sig"}' $jwk $header | jq -cs add)
# Export JWK
echo '{"keys":['$jwk']}'| jq . > jwks.json
