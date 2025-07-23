# Tessera Release Rule

[![简体中文][release-zh-badge]][release-zh-url]

[release-zh-badge]: https://img.shields.io/badge/RELEASE%20RULE-简体中文-blue.svg?style=for-the-badge&logo=release
[release-zh-url]: RELEASE_RULE_zh-CN.md

## Versioning Rule

The version number consists of three parts: `major.minor.patch`, e.g., `1.0.0`.

- **Major**: Incremented when the roadmap is fully completed.
- **Minor**: Incremented for any feature updates or breaking changes.
- **Patch**: Incremented for any bug fixes or minor improvements.

## Release Process

You must use the `script/release-package.rs` script to publish, which will automatically handle version number updates, packaging, generating changelogs, and pushing.

1. It is recommended to dry run the release script first to see if it meets expectations. Here is an example of a dry run:

   ```bash
   tessera on  main [!?⇡] via 🦀 v1.88.0
   ❯ rust-script scripts/release-package.rs -p tessera-ui patch
   📦 Package: tessera-ui
   📄 Path: tessera-ui\Cargo.toml
   🕓 Old version: 0.2.0
   🆕 New version: 0.2.1
   tessera-ui\CHANGELOG.md
   ╭───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮
   │ line                                                                                                                  │
   ├───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
   │ --- original                                                                                                          │
   │ +++ modified                                                                                                          │
   │ @@ -0,0 +1,6 @@                                                                                                       │
   │ +## [v0.2.1] - 2025-07-19 +08:00                                                                                      │
   │ +                                                                                                                     │
   │ +### Changes                                                                                                          │
   │ +                                                                                                                     │
   │ +[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.2.0...tessera-ui-v0.2.1) │
   │ +                                                                                                                     │
   ╰───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯
   [dry-run] git add tessera-ui\CHANGELOG.md
   tessera-ui\Cargo.toml
   ╭──────────────────────────────╮
   │ line                         │
   ├──────────────────────────────┤
   │ --- original                 │
   │ +++ modified                 │
   │ @@ -1,7 +1,7 @@              │
   │                              │
   │  [package]                   │
   │  name = "tessera-ui"         │
   │ -version = "0.2.0"           │
   │ +version = "0.2.1"           │
   │  edition.workspace = true    │
   │  license.workspace = true    │
   │  repository.workspace = true │
   ╰──────────────────────────────╯
   [dry-run] git add tessera-ui\Cargo.toml
   [dry-run] git commit -m "release(tessera-ui): v0.2.1"
   [dry-run] git tag tessera-ui-v0.2.1
   [dry-run] git push
   [dry-run] git push --tags
   example\Cargo.toml
   ╭────────────────────────────────────────────────────────────────────────────╮
   │ line                                                                       │
   ├────────────────────────────────────────────────────────────────────────────┤
   │ --- original                                                               │
   │ +++ modified                                                               │
   │ @@ -13,9 +13,9 @@                                                          │
   │  path = "src/lib.rs"                                                       │
   │                                                                            │
   │  [dependencies]                                                            │
   │ -tessera-ui = { path = "../tessera-ui" }                                   │
   │ -tessera-ui-macros = { path = "../tessera-ui-macros" }                     │
   │ -tessera-ui-basic-components = { path = "../tessera-ui-basic-components" } │
   │ +tessera-ui = { version = "0.2.1" }                                        │
   │ +tessera-ui-macros = { version = "0.1.0" }                                 │
   │ +tessera-ui-basic-components = { version = "0.0.0" }                       │
   │  rand = "0.9.1"                                                            │
   │  tokio = { version = "1.45.1", features = ["full"] }                       │
   │  log = "0.4.27"                                                            │
   ╰────────────────────────────────────────────────────────────────────────────╯
   [dry-run] git add example\Cargo.toml
   tessera-ui-logo\Cargo.toml
   ╭───────────────────────────────────────────────────────────────────────────────────────────────────╮
   │ line                                                                                              │
   ├───────────────────────────────────────────────────────────────────────────────────────────────────┤
   │ --- original                                                                                      │
   │ +++ modified                                                                                      │
   │ @@ -7,9 +7,9 @@                                                                                   │
   │  # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html │
   │                                                                                                   │
   │  [dependencies]                                                                                   │
   │ -tessera-ui = { path = "../tessera-ui" }                                                          │
   │ -tessera-ui-macros = { path = "../tessera-ui-macros" }                                            │
   │ -tessera-ui-basic-components = { path = "../tessera-ui-basic-components" }                        │
   │ +tessera-ui = { version = "0.2.1" }                                                               │
   │ +tessera-ui-macros = { version = "0.1.0" }                                                        │
   │ +tessera-ui-basic-components = { version = "0.0.0" }                                              │
   │  rand = "0.9.1"                                                                                   │
   │  rand_pcg = "0.9.0"                                                                               │
   │  tokio = { version = "1.46.1", features = ["full"] }                                              │
   ╰───────────────────────────────────────────────────────────────────────────────────────────────────╯
   [dry-run] git add tessera-ui-logo\Cargo.toml
   tessera-ui-basic-components\Cargo.toml
   ╭────────────────────────────────────────────────────────────────────╮
   │ line                                                               │
   ├────────────────────────────────────────────────────────────────────┤
   │ --- original                                                       │
   │ +++ modified                                                       │
   │ @@ -17,8 +17,8 @@                                                  │
   │  glyphon = { package = "glyphon-tessera-fork", version = "0.9.0" } │
   │  log = "0.4.27"                                                    │
   │  parking_lot = "0.12.4"                                            │
   │ -tessera-ui = { path = "../tessera-ui" }                           │
   │ -tessera-ui-macros = { path = "../tessera-ui-macros" }             │
   │ +tessera-ui = { version = "0.2.1" }                                │
   │ +tessera-ui-macros = { version = "0.1.0" }                         │
   │  unicode-segmentation = "1.12.0"                                   │
   │  encase = { version = "0.11.1", features = ["glam"] }              │
   │  glam = "0.30.4"                                                   │
   ╰────────────────────────────────────────────────────────────────────╯
   [dry-run] git add tessera-ui-basic-components\Cargo.toml
   tessera-ui-macros\Cargo.toml
   ╭──────────────────────────────────────────╮
   │ line                                     │
   ├──────────────────────────────────────────┤
   │ --- original                             │
   │ +++ modified                             │
   │ @@ -13,4 +13,4 @@                        │
   │  [dependencies]                          │
   │  quote = "1.0.40"                        │
   │  syn = "2.0.101"                         │
   │ -tessera-ui = { path = "../tessera-ui" } │
   │ +tessera-ui = { version = "0.2.1" }      │
   ╰──────────────────────────────────────────╯
   [dry-run] git add tessera-ui-macros\Cargo.toml
   [dry-run] git commit -m "chore: replace path dependencies with version for publish"
   [dry-run] cargo publish -p tessera-ui
   [dry-run] git reset --hard tessera-ui-v0.2.1
   example\Cargo.toml
   ╭────────────────────────────────────────────────────────────────────────────╮
   │ line                                                                       │
   ├────────────────────────────────────────────────────────────────────────────┤
   │ --- original                                                               │
   │ +++ modified                                                               │
   │ @@ -13,9 +13,9 @@                                                          │
   │  path = "src/lib.rs"                                                       │
   │                                                                            │
   │  [dependencies]                                                            │
   │ -tessera-ui = { path = "../tessera-ui" }                                   │
   │ -tessera-ui-macros = { path = "../tessera-ui-macros" }                     │
   │ -tessera-ui-basic-components = { path = "../tessera-ui-basic-components" } │
   │ +tessera-ui = { version = "0.2.1" }                                        │
   │ +tessera-ui-macros = { version = "0.1.0" }                                 │
   │ +tessera-ui-basic-components = { version = "0.0.0" }                       │
   │  rand = "0.9.1"                                                            │
   │  tokio = { version = "1.45.1", features = ["full"] }                       │
   │  log = "0.4.27"                                                            │
   ╰────────────────────────────────────────────────────────────────────────────╯
   tessera-ui-logo\Cargo.toml
   ╭───────────────────────────────────────────────────────────────────────────────────────────────────╮
   │ line                                                                                              │
   ├───────────────────────────────────────────────────────────────────────────────────────────────────┤
   │ --- original                                                                                      │
   │ +++ modified                                                                                      │
   │ @@ -7,9 +7,9 @@                                                                                   │
   │  # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html │
   │                                                                                                   │
   │  [dependencies]                                                                                   │
   │ -tessera-ui = { path = "../tessera-ui" }                                                          │
   │ -tessera-ui-macros = { path = "../tessera-ui-macros" }                                            │
   │ -tessera-ui-basic-components = { path = "../tessera-ui-basic-components" }                        │
   │ +tessera-ui = { version = "0.2.1" }                                                               │
   │ +tessera-ui-macros = { version = "0.1.0" }                                                        │
   │ +tessera-ui-basic-components = { version = "0.0.0" }                                              │
   │  rand = "0.9.1"                                                                                   │
   │  rand_pcg = "0.9.0"                                                                               │
   │  tokio = { version = "1.46.1", features = ["full"] }                                              │
   ╰───────────────────────────────────────────────────────────────────────────────────────────────────╯
   tessera-ui-basic-components\Cargo.toml
   ╭────────────────────────────────────────────────────────────────────╮
   │ line                                                               │
   ├────────────────────────────────────────────────────────────────────┤
   │ --- original                                                       │
   │ +++ modified                                                       │
   │ @@ -17,8 +17,8 @@                                                  │
   │  glyphon = { package = "glyphon-tessera-fork", version = "0.9.0" } │
   │  log = "0.4.27"                                                    │
   │  parking_lot = "0.12.4"                                            │
   │ -tessera-ui = { path = "../tessera-ui" }                           │
   │ -tessera-ui-macros = { path = "../tessera-ui-macros" }             │
   │ +tessera-ui = { version = "0.2.1" }                                │
   │ +tessera-ui-macros = { version = "0.1.0" }                         │
   │  unicode-segmentation = "1.12.0"                                   │
   │  encase = { version = "0.11.1", features = ["glam"] }              │
   │  glam = "0.30.4"                                                   │
   ╰────────────────────────────────────────────────────────────────────╯
   tessera-ui-macros\Cargo.toml
   ╭──────────────────────────────────────────╮
   │ line                                     │
   ├──────────────────────────────────────────┤
   │ --- original                             │
   │ +++ modified                             │
   │ @@ -13,4 +13,4 @@                        │
   │  [dependencies]                          │
   │  quote = "1.0.40"                        │
   │  syn = "2.0.101"                         │
   │ -tessera-ui = { path = "../tessera-ui" } │
   │ +tessera-ui = { version = "0.2.1" }      │
   ╰──────────────────────────────────────────╯
   ```

   As you can see, the dry run shows all operations. A responsible release should first review whether it meets expectations.

2. If the dry run is fine, you can perform the actual release. Here is an example of releasing tessera-ui:

   ```bash
   rust-script scripts/release-package.rs -p tessera-ui patch --execute
   ```

   This will perform the actual release operations, including updating the version number, generating a changelog, automatically converting path dependencies to version dependencies (so it can be published to crates.io), publishing to crates.io, and pushing to the remote repository.

   Note: The `--execute` parameter is mandatory; otherwise, the script will only perform a dry run and not execute the actual operations.
