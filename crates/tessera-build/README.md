# Tessera Build

[![简体中文][readme-cn-badge]][readme-cn-url]

[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge&logo=readme
[readme-cn-url]: https://github.com/tessera-ui/tessera/blob/main/crates/tessera-build/docs/README_CN.md

Build-time utilities for Tessera projects.

Split large TrueType fonts into on-demand Unicode resources in `tessera-config.toml`:

```toml
[assets.font-subsets]
"MyFont.ttf" = [] # All supported characters; or inclusive Unicode ranges.
```

Register `Fonts(res::FONT_SUBSETS)` at the application entry. Keep emoji fonts
whole to preserve sequences across Unicode ranges.
