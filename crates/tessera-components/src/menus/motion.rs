//! Container shape motion for vertical menus.
//!
//! ## Usage
//!
//! Distinguish the current action group while moving between related menus.

use tessera_ui::{
    Dp, FrameNanosControl, MOUSE_POINTER_ID, PointerInput, PointerInputModifierNode, State,
};

use crate::{
    animation::Animatable,
    shape_def::{RoundedCorner, Shape},
};

use super::{MenuController, MenuHost, MenuPosition};

pub(super) fn resting_corners(position: MenuPosition) -> [f32; 4] {
    match position {
        MenuPosition::Standalone => [16.0; 4],
        MenuPosition::First => [16.0, 16.0, 8.0, 8.0],
        MenuPosition::Middle => [8.0; 4],
        MenuPosition::Last => [8.0, 8.0, 16.0, 16.0],
    }
}

pub(super) struct MenuShapeMotion {
    corners: [Animatable; 4],
    target: [f32; 4],
    frame_nanos: u64,
    hovered: bool,
    mouse: bool,
    focused: bool,
    activated: bool,
}

impl MenuShapeMotion {
    pub fn new(position: MenuPosition, frame_nanos: u64) -> Self {
        let target = resting_corners(position);
        Self {
            corners: target.map(|value| Animatable::spring(value, 800.0_f32.sqrt(), 0.6)),
            target,
            frame_nanos,
            hovered: false,
            mouse: true,
            focused: false,
            activated: false,
        }
    }

    fn target(&self, position: MenuPosition, focused: bool) -> [f32; 4] {
        if self.hovered || focused || !self.activated {
            resting_corners(position)
        } else {
            [8.0; 4]
        }
    }

    pub fn needs_update(&self, position: MenuPosition, focused: bool) -> bool {
        self.focused != focused
            || self.target != self.target(position, focused)
            || (!self.activated && (self.hovered || focused))
    }

    pub fn update(&mut self, position: MenuPosition, focused: bool, frame_nanos: u64) {
        self.advance(frame_nanos);
        self.focused = focused;
        self.activated |= self.hovered || focused;
        self.target = self.target(position, focused);
        for (corner, target) in self.corners.iter_mut().zip(self.target) {
            corner.set_target(target);
        }
    }

    pub fn advance(&mut self, frame_nanos: u64) {
        if frame_nanos > self.frame_nanos {
            let dt = (frame_nanos - self.frame_nanos) as f32 / 1_000_000_000.0;
            for corner in &mut self.corners {
                corner.tick(dt);
            }
            self.frame_nanos = frame_nanos;
        }
    }

    pub fn is_idle(&self) -> bool {
        self.corners.iter().all(Animatable::is_idle)
    }

    pub fn shape(&self) -> Shape {
        let [top_left, top_right, bottom_right, bottom_left] = self
            .corners
            .map(|corner| RoundedCorner::manual(Dp(corner.value().max(0.0) as f64), 2.0));
        Shape::RoundedRectangle {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }
}

pub(super) fn advance(motion: State<MenuShapeMotion>, frame_nanos: u64) -> FrameNanosControl {
    motion.with_mut(|motion| motion.advance(frame_nanos));
    if motion.with(MenuShapeMotion::is_idle) {
        FrameNanosControl::Stop
    } else {
        FrameNanosControl::Continue
    }
}

pub(super) struct MenuShapeHover {
    pub motion: State<MenuShapeMotion>,
    pub host: Option<MenuHost>,
    pub controller: Option<State<MenuController>>,
}

impl PointerInputModifierNode for MenuShapeHover {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        let mouse = input
            .pointer_changes
            .last()
            .map(|change| change.pointer_id == MOUSE_POINTER_ID)
            .unwrap_or_else(|| self.motion.with(|motion| motion.mouse));
        let hovered = mouse
            && input.cursor_position_rel.is_some_and(|point| {
                point.x >= 0.0
                    && point.y >= 0.0
                    && point.x < input.computed_data.width.to_f32()
                    && point.y < input.computed_data.height.to_f32()
            })
            && self
                .host
                .as_ref()
                .zip(self.controller)
                .is_none_or(|(host, controller)| {
                    host.allows_hover(controller, input.cursor_position_abs())
                });
        if self
            .motion
            .with(|motion| motion.hovered != hovered || motion.mouse != mouse)
        {
            self.motion.with_mut(|motion| {
                motion.mouse = mouse;
                motion.hovered = hovered;
                motion.activated |= hovered;
            });
        }
    }
}
