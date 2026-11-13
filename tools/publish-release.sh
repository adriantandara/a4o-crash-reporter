#!/usr/bin/env bash
set -euo pipefail

version="${1:?folosire: tools/publish-release.sh <versiune> [--authenticode]}"
signing_flag="${2:-}"
if [ -z "${signing_flag}" ] && gh variable get SIGNPATH_ORGANIZATION_ID --repo adriantandara/a4o-crash-reporter > /dev/null 2>&1; then
  signing_flag="--authenticode"
fi
key="${A4O_RELEASE_KEY:-$HOME/.config/a4o-crash-reporter/release-primary.key}"
repo="adriantandara/a4o-crash-reporter"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

cd "$(dirname "$0")/.."

echo "Descarc draft-ul v$version"
gh release download "v$version" --repo "$repo" --dir "$work" --pattern '*.msi' --pattern '*.exe' --pattern 'SHA256SUMS.txt'

echo "Verific sumele SHA-256"
(cd "$work" && sha256sum -c SHA256SUMS.txt)

echo "Verific atestarea de build"
for file in "$work"/*.msi "$work"/*.exe; do
  gh attestation verify "$file" --repo "$repo"
done

echo "Semnez manifestul"
cargo run -q --release -p release-sign -- sign "$key" "$version" "$work" $signing_flag

gh release upload "v$version" "$work/release.json" "$work/release.json.sig" --repo "$repo" --clobber

read -r -p "Public release-ul v$version pe GitHub? Jucatorii il vad abia dupa aprobarea din panou. [da/nu] " answer
if [ "$answer" = "da" ]; then
  gh release edit "v$version" --repo "$repo" --draft=false
  echo "Publicat. Urmatorul pas: panou -> Crash-uri -> Versiuni aplicatie -> Aproba."
fi
