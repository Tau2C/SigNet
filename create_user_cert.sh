#!/usr/bin/env nix-shell
#! nix-shell -i bash --pure
#! nix-shell -p openssl bash coreutils

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <username>"
    exit 1
fi

NAME="$1"
USER_DIR="certs/$NAME"
CA_KEY="ca/ca.key"
CA_CERT="ca/ca.crt"

mkdir -p "$USER_DIR"

# Generate user private key
openssl ecparam -name prime256v1 -genkey -noout -out "$USER_DIR/$NAME.key"

# Generate certificate signing request (CSR)
openssl req -new -key "$USER_DIR/$NAME.key" \
    -subj "/CN=$NAME/O=TaU2C/C=PL" \
    -out "$USER_DIR/$NAME.csr"

# Sign the certificate with CA
openssl x509 -req -in "$USER_DIR/$NAME.csr" \
    -CA "$CA_CERT" -CAkey "$CA_KEY" -CAcreateserial \
    -out "$USER_DIR/$NAME.crt" -days 365 -sha256

# Show fingerprint
echo "✅ Certificate created for $NAME:"
openssl x509 -in "$USER_DIR/$NAME.crt" -noout -fingerprint -sha256
