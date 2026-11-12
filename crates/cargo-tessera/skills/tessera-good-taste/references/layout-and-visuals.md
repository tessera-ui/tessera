# Layout and visuals

- Express layout with existing containers, parent constraints, and ordered Modifier chains. For custom arrangement, pass a `LayoutPolicy` to `layout(policy)`; use a `LayoutModifier` for reusable wrapping measurement. Use lazy containers for large collections and finite viewports for scrolling.
- Author dimensions in `Dp`; use physical pixels for physical measurements and readback. Match each geometry consumer to its local, relative, or window coordinate space.
- Use `scaffold().window_insets(WindowInsets::SafeDrawing)` for a window-sized scaffold with a background under system areas. For custom layouts, use `window_insets_padding` at screen boundaries. Use inset size helpers for edge spacers and `consume_window_insets` for space already supplied by a parent.
- Observe local sizes for size-dependent content and relative coordinates for container annotations. Use window rectangles for window-positioned integrations.
- Use theme roles, typography, and shared shapes for fills, outlines, shadows, and clipping. Choose modifier order to express content, visual, and interaction bounds.
- Pass generated assets, paths, or bytes to `.painter(...)`; they decode off the UI thread. Load custom resources with `Resource::load(source, decode)`, retain the handle with `remember`, and observe `Resource::get` as `LoadState`. Register fonts through `Fonts` or `FontStylesheet` at the entry.
