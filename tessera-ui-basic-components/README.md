# Tessera basic components

[![简体中文][readme-cn-badge]][readme-cn-url]

[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge&logo=readme
[readme-cn-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-ui-basic-components/docs/README_CN.md

`tessera-ui-basic-components` provides a set of basic UI components for building common user interfaces. This includes buttons, text, layout containers, and more.

## Available components

### Layout components

- `column`: vertical layout container
- `row`: horizontal layout container
- `boxed`: bordered container
- `spacer`: adjustable empty spacer

### Core components

- `surface`: customizable surface component; base for non-glass components
- `button`: clickable button
- `switch`: toggle switch
- `slider`: slider control
- `progress`: progress bar
- `fluid_glass`: customizable glass surface component; base for glass-style components
- `glass_button`: glass-style clickable button
- `glass_switch`: glass-style toggle switch
- `glass_slider`: glass-style slider
- `glass_progress`: glass-style progress bar
- `text`: text display, supports system fonts and colored emoji
- `image`: image display (supports AVIF, BMP, DDS, EXR, GIF, HDR, ICO, JPEG, PNG, PNM, QOI, TGA, TIFF, WebP)
- `image_vector`: vector image display (SVG support)
- `icon`: semantic wrapper over vector/raster icons with consistent sizing & tint helpers
- `checkbox`: checkbox
- `tabs`: tab component
- `text_editor`: multi-line text editor
- `bottom_nav_bar`: bottom navigation bar
- `side_bar_provider`: side popup panel, provides both glass and non-glass styles
- `dialog_provider`: dialog, provides both glass and non-glass styles
- `bottom_sheet_provider`: bottom sheet, provides both glass and non-glass styles
- `scrollable`: scrollable container supporting vertical and horizontal scrolling
