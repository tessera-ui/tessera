# Releasing Tessera

[简体中文](RELEASE_RULE_zh-CN.md)

1. Ensure any required fork versions are published. Preview the plan with `rust-script scripts/release-package.rs`. Releases follow Semantic Versioning and Conventional Commits; use `--major <package>` to request a major bump.
2. Add release notes at the top of each listed package's `CHANGELOG.md`. Use the planned `## [vX.Y.Z] - YYYY-MM-DD` heading and comparison link. Summarize user-visible changes and explain how to migrate breaking changes. If only dependencies changed, one line is enough.
3. Verify packages without publishing with `rust-script scripts/release-package.rs --check`. On a branch, run the script with `--prepare` and open a pull request titled `release: prepare workspace versions`. If `main` moves before merge, incorporate the new changes, update the notes, and rerun planning and preparation.
4. From a clean checkout of the latest `main`, run `rust-script scripts/release-package.rs --execute` to verify, publish, and push tags.
