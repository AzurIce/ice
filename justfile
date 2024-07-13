lint:
    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo doc --no-deps --workspace --document-private-items

changelog:
    git cliff -o CHANGELOG.md

# should make sure the workspace is clean
tag tag:
    git cliff --tag {{tag}} -o CHANGELOG.md
    # replace the version in Cargo.toml
    sed -i "s/^version = .*/version = \"{{tag}}\"/" Cargo.toml
    # generate ice.json for scoop menifest
    echo '{
    "version": "{{tag}}",
    "url": "https://github.com/AzurIce/ice/releases/download/{{tag}}/ice-{{tag}}-x86_64-windows.zip",
    "bin": "ice.exe"
}' > ice.json
    git add CHANGELOG.md Cargo.toml ice.json
    git commit -m "chore(release): prepare for {{tag}}"

