#!/usr/bin/env bash
# Install the native engine selected by Cargo.lock, plus its Book toolchain.
set -euo pipefail
cd "$(dirname "$0")/.."

revision=$(awk '
  /^\[\[package\]\]/ { anchor = 0 }
  /^name = "anchor-book"$/ { anchor = 1 }
  anchor && /^source = "git\+https:\/\/github.com\/anchor-book\/anchor-book[?#]/ {
    sub(/^.*#/, ""); sub(/"$/, ""); print; exit
  }
' Cargo.lock)
if [[ ! "$revision" =~ ^[0-9a-f]{40}$ ]]; then
  echo "Cargo.lock must select an exact Anchor Book Git revision; update the dependency first." >&2
  exit 1
fi

tools_root="$PWD/target/book-tools"
export PATH="$tools_root/bin:$PATH"
if [[ "$("$tools_root/bin/anchorbook" --version 2>/dev/null || true)" != *"($revision)" ]]; then
  cargo install --git https://github.com/anchor-book/anchor-book --rev "$revision" \
    --locked --force --root "$tools_root" anchor-book
fi
cargo install mdbook --version 0.5.4 --locked --root "$tools_root"
cargo install mdbook-mermaid --version 0.17.1 --locked --root "$tools_root"
cargo install cargo-nextest --version 0.9.151 --locked --root "$tools_root"
mdbook-mermaid install .
echo 'Book tools ready. For this shell: export PATH="$PWD/target/book-tools/bin:$PATH"'
