#!/usr/bin/env bash
set -euo pipefail

# List of triples to compile
targets=(
    # x86_64-apple-darwin
    # aarch64-apple-darwin
    # x86_64-pc-windows-msvc
    # aarch64-pc-windows-msvc
    x86_64-unknown-linux-gnu
    x86_64-unknown-linux-musl
    aarch64-unknown-linux-gnu
    aarch64-unknown-linux-musl
)

crate=$(basename "$PWD")
normal=$(tput sgr0)
bold=$(tput bold)

# Clean dist/
rm -rf dist
mkdir dist

for tgt in "${targets[@]}"; do
    echo ""
    echo "${bold}󱌣 Building for $tgt${normal}"

    # Invoke the CLI, platform-suffix, release and without JS loader
    napi build dist \
        --release \
        --platform \
        --target "$tgt" \
        --js false
        # --zig \

    # If there is any .d.ts left (the CLI does not allow disabling –dts), we delete it
    rm -f dist/*.d.ts
done

echo ""
for file in dist/${crate}.*.node; do
  # Extract everything that comes after "dist/${crate}."
  nombre=${file#dist/${crate}.}
  mv "$file" "dist/$nombre"
done

echo "${bold}󰄬 Finished${normal}"
tree -h -C ./dist
echo ""