## [v0.1.0] - 2026-11-16

### Added

- First release. Provides design-system-independent building blocks for
  Tessera components: alignment and shape definitions, scroll physics, and
  layout, window inset, semantics, and visual modifiers.
- Complete generic interaction modifiers (`clickable`, `combined_clickable`,
  `toggleable`, `selectable`, and `draggable`) and `InteractionState`, including
  keyboard, focus, and accessibility behavior without design-system visuals.
- Asynchronous tap, long-press, drag, and scroll detectors in `pointer_gestures`,
  with shared movement thresholds, sampled events, and cancellation callbacks.
