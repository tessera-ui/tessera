# Tessera Build

[![English][readme-en-badge]][readme-en-url]

[readme-en-badge]: https://img.shields.io/badge/README-English-blue.svg?style=for-the-badge&logo=readme
[readme-en-url]: https://github.com/tessera-ui/tessera/blob/main/crates/tessera-build/README.md

Tessera 项目的构建期工具集。

在 `tessera-config.toml` 中为大型 TrueType 字体启用按需 Unicode 分片：

```toml
[assets.font-subsets]
"MyFont.ttf" = []
```

空数组表示全部支持的字符，也可指定闭区间。入口注册 `Fonts(res::FONT_SUBSETS)`。
Emoji 字体保留完整文件以支持跨区间的组合序列。
