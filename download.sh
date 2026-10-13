#!/usr/bin/env bash
set -euo pipefail

# AI generated for saving time. TODO: review

MODELS_DIR="${MODELS_DIR:-models}"
# Done-markers live outside MODELS_DIR, since the app loads every file in it.
CACHE_DIR="${CACHE_DIR:-.cache}"
TMP_DIR="$(mktemp -d)"

urls=(
  "http://graphics.stanford.edu/pub/3Dscanrep/bunny.tar.gz"
  "http://graphics.stanford.edu/pub/3Dscanrep/dragon/dragon_recon.tar.gz"
  "http://graphics.stanford.edu/pub/3Dscanrep/happy/happy_recon.tar.gz"
  "http://graphics.stanford.edu/pub/3Dscanrep/drill.tar.gz"
)

cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

bold=$'\e[1m'
green=$'\e[32m'
yellow=$'\e[33m'
reset=$'\e[0m'
log() { printf '%s==>%s %s\n' "$bold" "$reset" "$*"; }
ok() { printf '%s  ✓%s %s\n' "$green" "$reset" "$*"; }
skip() { printf '%s  ↷%s %s\n' "$yellow" "$reset" "$*"; }

fetch_one() {
  local url="$1"
  local file="${url##*/}"      # bunny.tar.gz
  local name="${file%.tar.gz}" # bunny
  local marker="$CACHE_DIR/$name.done"

  if [[ -f "$marker" ]]; then
    skip "$name already extracted"
    return 0
  fi

  log "Downloading $name"
  curl -fL --progress-bar -o "$TMP_DIR/$file" "$url"

  log "Extracting $name .ply files"
  local extract="$TMP_DIR/$name"
  mkdir -p "$extract"
  tar -xzf "$TMP_DIR/$file" -C "$extract" --wildcards '*.ply'

  # Flatten into MODELS_DIR; refuse to overwrite a file from another archive.
  local f dest
  while IFS= read -r -d '' f; do
    dest="$MODELS_DIR/$(basename "$f")"
    if [[ -e "$dest" ]]; then
      printf 'error: %s already exists\n' "$dest" >&2
      return 1
    fi
    mv "$f" "$dest"
  done < <(find "$extract" -type f -name '*.ply' -print0)

  rm -f "$TMP_DIR/$file"
  mkdir -p "$CACHE_DIR"
  touch "$marker"
  ok "$name done"
}

main() {
  mkdir -p "$MODELS_DIR"
  local i=0 total=${#urls[@]}
  for url in "${urls[@]}"; do
    i=$((i + 1))
    printf '\n%s[%d/%d]%s\n' "$bold" "$i" "$total" "$reset"
    fetch_one "$url"
  done
  printf '\n'
  ok "All models in $MODELS_DIR/"
  ls "$MODELS_DIR" | wc -l | xargs printf '%s files\n'
}

main "$@"
