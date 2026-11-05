# Tessera releases

Use Semantic Versioning and Conventional Commits: `BREAKING CHANGE:` requires a major release, `feat:` a minor release, and fixes a patch release.

Preview the release plan:

```sh
rust-script scripts/release-package.rs
```

Add `--major <package>` to request a major release. Add `--execute` to publish the reviewed plan to crates.io and push release commits and tags.
