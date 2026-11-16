# Tessera releases

Use Semantic Versioning and Conventional Commits: `BREAKING CHANGE:` requires a major release, `feat:` a minor release, and fixes a patch release.

Preview the release plan:

```sh
rust-script scripts/release-package.rs
```

Run `rust-script scripts/release-package.rs --check` to verify packaged sources, scaffolds and installed resources without publishing. Publish versioned fork dependencies before releasing the workspace.

Release notes are written by hand when preparing a release. For each package in the plan, read the history since its previous tag and add a section at the top of its `CHANGELOG.md`:

```md
## [v1.2.0] - 2026-11-16

### Breaking changes

- What changed and how to migrate.

### Added

- ...

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/name-v1.1.0...name-v1.2.0)
```

Use `### Breaking changes`, `### Added`, `### Changed`, `### Fixed` and `### Performance` as needed, and summarize rather than list commits. The release plan prints each package's comparison link. A package released only for dependency updates notes that in one line.

Use `--major <package>` to request a major bump. Run `--prepare` on a branch to write versions, dependency requirements and changelogs, and merge them through a pull request titled `release: prepare workspace versions`. Then run `--execute` from a clean checkout of the latest `main` to verify and publish the packages and push their tags.
