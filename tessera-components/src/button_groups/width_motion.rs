//! Press motion and width exchange for standard button groups.
//!
//! ## Usage
//!
//! Expand an activated button while preserving the group's total width.

use std::{any::TypeId, sync::Arc};

use tessera_ui::{
    ParentDataModifierNode, PointerEventPass, PointerInput, PointerInputModifierNode, Px, State,
    current_frame_nanos, modifier::ParentDataMap,
};

use crate::{
    animation::material_motion::{Scalar, Spec},
    modifier::InteractionState,
};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct PressWidthMotion {
    scalar: Scalar,
    pressed: bool,
    completing: bool,
    enabled: bool,
}

impl Default for PressWidthMotion {
    fn default() -> Self {
        Self {
            scalar: Scalar::new(0.0),
            pressed: false,
            completing: false,
            enabled: false,
        }
    }
}

impl PressWidthMotion {
    pub fn update(&mut self, pressed: bool, enabled: bool, frame: u64) {
        self.advance(frame);
        if !enabled {
            *self = Self::default();
            return;
        }
        self.enabled = true;
        if pressed && !self.pressed {
            self.scalar.retarget(1.0, Spec::FastSpatial, frame);
            self.completing = true;
        }
        self.pressed = pressed;
        self.finish_press(frame);
    }

    pub fn pulse(&mut self, frame: u64) {
        self.advance(frame);
        if self.enabled && !self.pressed {
            self.scalar.retarget(1.0, Spec::FastSpatial, frame);
            self.completing = true;
        }
    }

    pub fn advance(&mut self, frame: u64) {
        self.scalar.advance(frame);
        self.finish_press(frame);
    }

    fn finish_press(&mut self, frame: u64) {
        if !self.pressed && self.completing && self.scalar.value >= 0.75 {
            self.completing = false;
            self.scalar.retarget(0.0, Spec::FastSpatial, frame);
        }
    }

    pub fn value(self) -> f64 {
        self.scalar.value
    }
    pub fn is_animating(self) -> bool {
        self.scalar.is_animating()
    }
}

pub(super) struct WidthObserver {
    pub interaction: State<InteractionState>,
    pub motion: State<PressWidthMotion>,
}

impl PointerInputModifierNode for WidthObserver {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Final {
            return;
        }
        let mut next = self.motion.get();
        next.update(
            self.interaction.with(InteractionState::is_pressed),
            next.enabled,
            current_frame_nanos(),
        );
        if next != self.motion.get() {
            self.motion.set(next);
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) struct WidthData {
    pub progress: f64,
    pub compression_limit: Px,
}

impl ParentDataModifierNode for WidthData {
    fn equivalent(&self, other: &dyn ParentDataModifierNode) -> bool {
        (other as &dyn std::any::Any)
            .downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }

    fn apply_parent_data(&self, data: &mut ParentDataMap) {
        data.insert(TypeId::of::<Self>(), Arc::new(*self));
    }
}

pub(super) fn redistribute_widths(widths: &mut [Px], data: &[WidthData], ratio: f32) {
    if widths.len() < 2 {
        return;
    }
    for index in 0..widths.len() {
        let progress = data[index].progress;
        if progress == 0.0 {
            continue;
        }
        let neighbors = usize::from(index > 0) + usize::from(index + 1 < widths.len());
        let mut limit = f64::from(ratio) * f64::from(widths[index].0) / neighbors as f64;
        if index > 0 {
            limit = limit.min(f64::from(data[index - 1].compression_limit.0));
        }
        if index + 1 < widths.len() {
            limit = limit.min(f64::from(data[index + 1].compression_limit.0));
        }
        let growth = Px((progress * limit + 0.5).floor() as i32);
        let minimum = Px(-widths[index].0 / neighbors as i32);
        let mut actual = Px::ZERO;
        if index > 0 {
            let amount = growth.max(minimum).min(widths[index - 1]);
            widths[index - 1] -= amount;
            actual += amount;
        }
        if index + 1 < widths.len() {
            let amount = growth.max(minimum).min(widths[index + 1]);
            widths[index + 1] -= amount;
            actual += amount;
        }
        widths[index] += actual;
    }
}
