#!/usr/bin/env nix-shell
#! nix-shell -i bash --pure
#! nix-shell -p openssl bash coreutils
set -e

CA_DIR="ca"
CA_KEY="$CA_DIR/ca.key"
CA_CERT="$CA_DIR/ca.crt"

mkdir -p "$CA_DIR"

# Generate private key (ECC P-256)
openssl ecparam -name prime256v1 -genkey -noout -out "$CA_KEY"

# Create a long-lived self-signed certificate (10 years)
openssl req -x509 -new -key "$CA_KEY" -sha256 -days 3650 \
    -subj "/CN=MyRootCA/O=MyOrg/C=PL" \
    -out "$CA_CERT"

echo "✅ CA created:"
echo "  Key:  $CA_KEY"
echo "  Cert: $CA_CERT"
