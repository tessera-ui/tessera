# Layout and visuals

- Express layout with existing containers, parent constraints, and ordered Modifier chains. Use lazy containers for large collections and finite viewports for scrolling.
- Author dimensions in `Dp`; use physical pixels for physical measurements and readback. Match each geometry consumer to its local, relative, or window coordinate space.
- Observe local sizes for size-dependent content and relative coordinates for container annotations. Use window rectangles for window-positioned integrations.
- Use theme roles, typography, and shared shapes for fills, outlines, shadows, and clipping. Choose modifier order to express content, visual, and interaction bounds.
- Use documented painter and generated asset interfaces. Give dynamic resources loading and error states, and reuse decoded or prepared content across rebuilds.
