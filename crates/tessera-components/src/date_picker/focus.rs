//! Shared calendar focus drawing for day and year choices.
//!
//! ## Usage
//!
//! Keep keyboard and assistive-technology focus visible in date selectors.

use tessera_ui::Color;

use crate::{shape_def::Shape, surface::record_focus_indicator};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct CalendarFocusIndicator {
    pub(super) visible: bool,
    pub(super) shape: Shape,
    pub(super) color: Color,
}

impl tessera_ui::DrawModifierNode for CalendarFocusIndicator {
    fn draw(
        &self,
        context: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        if self.visible {
            record_focus_indicator(context.render_input, self.shape, self.color);
        }
        content.draw(context.render_input);
    }
}
