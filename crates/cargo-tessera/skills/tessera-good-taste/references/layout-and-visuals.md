# Layout and visuals

- Express layout with existing containers, parent constraints, and ordered Modifier chains. Use lazy containers for large collections and finite viewports for scrolling.
- Author dimensions in `Dp`; use physical pixels for physical measurements and readback. Match each geometry consumer to its local, relative, or window coordinate space.
- Apply `window_insets_padding(WindowInsets::SafeDrawing)` at screen boundaries. Use inset size helpers for edge spacers and `consume_window_insets` for space already supplied by a parent.
- Observe local sizes for size-dependent content and relative coordinates for container annotations. Use window rectangles for window-positioned integrations.
- Use theme roles, typography, and shared shapes for fills, outlines, shadows, and clipping. Choose modifier order to express content, visual, and interaction bounds.
- Pass generated assets directly to `.painter(asset)`. Retain `Asset::load` or `load_with` handles with `remember` for custom resources and observe `Resource::get` as `LoadState`. Register additional fonts through `Fonts` at the entry; pass generated `res::FONT_SUBSETS` for fonts configured in `[assets.font-subsets]`. Keep fonts with cross-character emoji sequences whole.
