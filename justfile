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
    echo -e '{\n"version": "{{tag}}",\n"url": "https://github.com/AzurIce/ice/releases/download/{{tag}}/ice-{{tag}}-x86_64-windows.zip",\n"bin": "ice.exe"\n}' > ice.json
    cargo check
    git add CHANGELOG.md Cargo.toml Cargo.lock ice.json
    git commit -m "chore(release): prepare for {{tag}}"
    git tag -a {{tag}} -m "release v{{tag}}"
