# Tessera Release Rule

[![简体中文][release-zh-badge]][release-zh-url]

[release-zh-badge]: https://img.shields.io/badge/RELEASE%20RULE-简体中文-blue.svg?style=for-the-badge&logo=release
[release-zh-url]: RELEASE_RULE_zh-CN.md

## Versioning Rule

The version number consists of three parts: `major.minor.patch`, e.g., `1.0.0`.

- **Major**: Incremented for breaking changes (`BREAKING CHANGE:`) or when the roadmap is fully completed (manually via `--major`).
- **Minor**: Incremented for any feature updates (`feat:`).
- **Patch**: Incremented for any bug fixes or minor improvements.

## Release Process

You must use the `scripts/release-package.rs` script to publish. It will automatically handle version number updates, packaging, generating changelogs, and pushing based on Conventional Commits. The release process is limited to the following three packages: `tessera-ui`, `tessera-components`, and `tessera-macros`.

### How it Works

1. **Automatic Version Analysis**: The script analyzes Git history since the last tag for all publishable packages (`tessera-ui`, `tessera-components`, `tessera-macros`).
   - A commit with `BREAKING CHANGE` in its body will result in a **major** version bump.
   - A `feat:` commit will result in a **minor** version bump.
   - Any other commit type (`fix:`, `refactor:`, etc.) will result in a **patch** version bump.
2. **Dependency Propagation**: If a base package is updated, any publishable packages that depend on it will also receive a **patch** version bump to ensure consistency. The release order is determined by a topological sort of the dependency graph.
3. **Manual Major Bump**: To perform a major version bump, you must use the `--major <package-name>` flag. This is reserved for roadmap milestones.

### Usage

1. It is recommended to dry run the release script first to see if it meets expectations. The script runs in dry-run mode by default.

   Example: Trigger a release analysis based on changes in `tessera-ui`. If `tessera-ui` has `feat` commits, it will get a minor bump, and its dependents will get a patch bump.

   ```bash
   rust-script scripts/release-package.rs
   ```

   Example: Force a major version bump for `tessera-ui`. This will also trigger patch bumps for its dependents.

   ```bash
   rust-script scripts/release-package.rs --major tessera-ui
   ```

   The script will output a clear "Release Plan" table showing which packages will be released and how their versions will be bumped.

2. If the dry run is fine, you can perform the actual release by adding the `--execute` flag.

   ```bash
   # Execute the release based on automatic analysis
   rust-script scripts/release-package.rs --execute

   # Execute a release with a major bump for tessera-ui
   rust-script scripts/release-package.rs --major tessera-ui --execute
   ```

   This will perform the actual release operations, including updating version numbers, generating changelogs, publishing to crates.io, and pushing to the remote repository.
