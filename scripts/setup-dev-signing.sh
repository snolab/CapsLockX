#!/bin/bash
# Create the local code-signing identity that build.sh uses.
#
# Why this exists
# ---------------
# Ad-hoc signing (`codesign -s -`) produces a Designated Requirement pinned to
# the binary's cdhash:
#
#     designated => cdhash H"59e022566e37792996229f0de89f79c07cac8bb2"
#
# TCC stores Accessibility / Screen Recording grants against that requirement,
# so *every* rebuild changes the cdhash and silently drops the grant — CLX
# comes back up in "limited mode" with no CGEventTap and no hotkeys. Passing
# `--identifier` does not help: the identifier never makes it into the DR.
#
# Signing with a self-signed certificate instead yields:
#
#     designated => identifier "com.snomiao.capslockx" and certificate leaf = H"..."
#
# which is independent of the binary's contents, so the grant survives rebuilds
# and only has to be given once.
#
# The certificate does NOT need to be added to the trust store — codesign
# accepts an untrusted self-signed leaf, and TCC only evaluates the DR. That
# keeps this script free of GUI password prompts. Gatekeeper (`spctl`) rejects
# the result, which is irrelevant for locally built binaries: they carry no
# quarantine attribute. Ad-hoc signing is rejected by spctl too.
#
# Idempotent — safe to re-run. Re-run it and you get a *new* leaf, which costs
# you one more Accessibility re-grant, so it exits early when the cert exists.
set -e

CN="${CLX_SIGN_IDENTITY:-CapsLockX Dev Signing}"
KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"

if security find-certificate -c "$CN" >/dev/null 2>&1; then
    echo "[signing] '$CN' already in the login keychain — nothing to do."
    echo "[signing] Delete it in Keychain Access first if you really want a fresh one."
    exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# LibreSSL (the system openssl) defaults to a PKCS#12 MAC that the Security
# framework refuses to import, so the legacy SHA-1/3DES algorithms are pinned
# explicitly. The passphrase is throwaway — it only guards the temp file.
openssl req -x509 -newkey rsa:2048 -nodes -days 7300 \
    -keyout "$WORK/key.pem" -out "$WORK/cert.pem" \
    -subj "/CN=$CN" \
    -addext "basicConstraints=critical,CA:false" \
    -addext "keyUsage=critical,digitalSignature" \
    -addext "extendedKeyUsage=critical,codeSigning" 2>/dev/null

openssl pkcs12 -export -out "$WORK/cert.p12" \
    -inkey "$WORK/key.pem" -in "$WORK/cert.pem" \
    -passout pass:clxdev -macalg sha1 \
    -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES

# -A plus the explicit -T entries let codesign use the private key without
# throwing a "wants to access your keychain" dialog on every build.
security import "$WORK/cert.p12" -k "$KEYCHAIN" -P clxdev \
    -T /usr/bin/codesign -T /usr/bin/security -A

echo "[signing] created '$CN' in the login keychain."
echo "[signing] leaf SHA-1: $(openssl x509 -in "$WORK/cert.pem" -noout -fingerprint -sha1 | cut -d= -f2)"
echo "[signing] Run ./build.sh, then grant Accessibility once — it sticks from now on."
