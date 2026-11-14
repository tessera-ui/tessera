# Local layout coordinates

Native controls measure and edit their content in local physical pixels. A
uniform scale changes only how those coordinates map to the window.

## Authoring a scaled subtree

`scaled_box(scale).child(...)` applies a finite, positive uniform scale around
the top-left origin of one direct layout child. Wrap multiple controls in a
Row, Column, or another container before supplying them as its child. Omitted
content produces an empty box subject to the parent constraints.

The boundary measures its child under inverse parent constraints and reports
the scaled occupied size to its parent, rounding that occupied size upwards.
Integer constraints can have no exact inverse interval. In that case the child
uses the greatest integer size permitted by the inverse maximum, and the
outer box still satisfies its parent interval. Overflow and clipping follow
the ordinary layout rules.

Responsive specification figures should keep one canonical local scene size
and derive a single fit scale from the available width. Component sizes,
padding, text metrics, and hit geometry stay canonical inside that boundary.
Do not additionally scale their individual authoring values.

## Shared geometry

`LocalToWindow` represents a uniform scale followed by a fractional
translation. A parent transformation composes with a child's transformation once.

`LayoutCoordinates` contains the unscaled `local_size` and the authoritative
`local_to_window` transformation. Its window bounds are a `FloatRect`.
Measured layout sizes remain integer `PxSize` values; fractional window sizes
must not be written back into local measurement or native control geometry.

Rounded shape masks transform their fractional origins, extents, and radii
together. Pointer shape checks use that same geometry. Integer scissors round
outwards after the transformation.

## Rendering

Draw commands carry their local dimensions and an explicit transformation.
Pipelines map local geometry into the render target once. Fragment-space shape
calculations must use the inverse transformation where appropriate; changing
only the vertex positions does not scale line widths or fragment-space shapes.
Text rendering scales glyph placement and glyph rasterization together.
Physical caches, such as atlases, must account for scale.

Local drawing and sampling padding scale with the command. Explicit absolute
sampling regions and global sampling already describe window coordinates.
Sampling regions remain independent of clipping and draw scissors.

## Input, focus, and accessibility

Canonical pointer samples, captured pointer positions, and gesture histories
remain in window coordinates. A stationary pointer must not acquire movement
when an ancestor moves or scales. Handlers receive an inverse-mapped local
cursor position. Native drag consumers convert observed displacements into
their local controller units.

Scroll handlers consume local displacement and return their remainder to window
units before ancestor delivery. Nested scroll connections also map requests,
consumption, and fling velocity through each provider's own coordinate space.

Focus reveal converts precise window bounds into the reveal container's local
units before changing its scroll offsets. IME anchors map through the same
transformation, and platform area sizes receive an integer enclosing extent.
Accessibility publishes transformed bounds while preserving semantic identity,
actions, selection, and focus behavior.

## Geometry observers

- `on_globally_positioned` reports outward integer bounds.
- `on_globally_positioned_precise` reports exact fractional window bounds.
- `on_local_to_window` reports changes to the transformation.
- `on_layout_coordinates` reports local dimensions and their transformation.

New observers receive the current value; others run only on change. Local-size
consumers must use local dimensions rather than a window rectangle's dimensions.
