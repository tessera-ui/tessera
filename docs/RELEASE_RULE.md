# Tessera releases

Use Semantic Versioning and Conventional Commits: `BREAKING CHANGE:` requires a major release, `feat:` a minor release, and fixes a patch release.

Preview the release plan:

```sh
rust-script scripts/release-package.rs
```

Run `rust-script scripts/release-package.rs --check` to verify packaged sources, scaffolds and installed resources without publishing. Publish versioned fork dependencies before releasing the workspace.

Use `--major <package>` to request a major bump. Use `--prepare` to write versions, dependency requirements and changelogs for review. Use `--execute` from a clean checkout to verify packages, publish them, then push the release commit and tags.
