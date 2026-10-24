# Local layout coordinates

Native controls measure and edit their content in local physical pixels. A
uniform scale changes how those coordinates appear in the window; it does not
change process density, rebuild a control from an image, or replace its input
and accessibility behavior.

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
translation. Its constructor rejects nonfinite, nonpositive, and noninvertible
scales. A parent transformation composes with a child's transformation once.

`LayoutCoordinates` contains the unscaled `local_size` and the authoritative
`local_to_window` transformation. Its window bounds are a `FloatRect`.
Measured layout sizes remain integer `PxSize` values; fractional window sizes
must not be written back into local measurement or native control geometry.

Placement modifiers transform parent-local positions. Prepared geometry keeps
the resulting transformation at each ordered input action. Input delivery
reuses those prepared values instead of running placement modifiers again.
Cached layouts and subtree replay preserve the same transformation metadata.
Geometry journals update affected descendants and frame consumers when an
ancestor transformation changes.

Rounded shape masks transform their fractional origins, extents, and radii
together. Pointer shape checks use that same geometry. Integer scissors round
outwards after the transformation.

## Rendering

Draw commands carry their local dimensions and an explicit transformation.
Pipelines map local geometry into the render target once. Fragment-space shape
calculations must use the inverse transformation where appropriate; changing
only the vertex positions does not scale line widths or fragment-space shapes.
Text rendering scales glyph placement and glyph rasterization together.

Local texture and atlas generation use their own identity coordinate space.
Their final scene composite receives the scene transformation once. Physical
caches must account for scale or bypass an incompatible cached entry.

Local drawing and sampling padding scale with the command. Explicit absolute
sampling regions and global sampling already describe window coordinates.
Sampling regions remain independent of clipping and draw scissors.

## Input, focus, and accessibility

Canonical pointer samples, captured pointer positions, and gesture histories
remain in window coordinates. A stationary pointer must not acquire movement
when an ancestor moves or scales. Handlers receive an inverse-mapped local
cursor position. Native drag consumers convert observed displacements into
their local controller units; integer drag callbacks retain rounding remainder
in window units so repeated small movements do not drift.

Wheel lines are normalized before crossing a scale boundary. Scroll handlers
consume local displacement and return their remainder to window units before
ancestor delivery. Nested scroll connections also map requests, consumption,
and fling velocity through each provider's own coordinate space.

Focus reveal converts precise window bounds into the reveal container's local
units before changing its scroll offsets. IME anchors map through the same
transformation, and platform area sizes receive an integer enclosing extent.
Accessibility publishes transformed bounds while preserving semantic identity,
actions, selection, and focus behavior.

## Geometry observers

- `on_globally_positioned` retains its previous integer behavior at scale one
  and reports an outward integer envelope for a scaled node.
- `on_globally_positioned_precise` reports exact fractional window bounds.
- `on_local_to_window` reports changes to the transformation.
- `on_layout_coordinates` reports local dimensions and their transformation.

Every newly attached observer receives its first valid geometry even when
another observer already watched the unchanged node. Existing observers run
only when their own observed value changes. A local-size consumer, such as an
outline or a sheet drag extent, must use local dimensions rather than a window
rectangle's dimensions.

## Verification

The runtime regression at scale `0.625` and origin `(10.25, 20.5)` checks a
`240 × 56` local child occupying exactly `150 × 35` window pixels, actual
inverse-mapped input, and remaining scroll delivery to an ancestor. Existing
observer coverage checks first attachment, reattachment, unchanged frames,
and independent size versus transformation changes.

Native GPU coverage must also exercise real outlined text fields and time
pickers, including editing, focus, clipping, and accessibility. Identity-scale
checks protect existing layout and rendering behavior. Performance checks
should observe measurement and geometry work at the production runtime
boundary rather than assert wall-clock timing.
