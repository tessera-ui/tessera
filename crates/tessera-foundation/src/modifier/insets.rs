use tessera_ui::{
    AxisConstraint, LayoutModifierChild, LayoutModifierInput, LayoutModifierNode, LayoutResult,
    MeasurementError, WindowInsets,
};

use super::layout::{ConstraintModifierNode, PaddingModifierNode};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct InsetPadding {
    pub insets: WindowInsets,
    pub consume_only: bool,
}

impl LayoutModifierNode for InsetPadding {
    fn measure_content(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        let requested = self.insets.get();
        let padding = if self.consume_only {
            Default::default()
        } else {
            requested.exclude(input.layout_input.consumed_window_insets())
        };
        input
            .layout_input
            .with_consumed_window_insets(requested, || {
                PaddingModifierNode { padding }.measure_content(input, child)
            })
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct InsetSize {
    pub insets: WindowInsets,
    pub edge: usize,
}

impl LayoutModifierNode for InsetSize {
    fn measure_content(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        let insets = self
            .insets
            .get()
            .exclude(input.layout_input.consumed_window_insets());
        let edge = [insets.left, insets.top, insets.right, insets.bottom][self.edge].to_px();
        let axis = Some(AxisConstraint::exact(edge));
        ConstraintModifierNode {
            width_override: if self.edge % 2 == 0 { axis } else { None },
            height_override: if self.edge % 2 == 1 { axis } else { None },
            fill_width: false,
            fill_height: false,
            enforce_incoming: true,
        }
        .measure_content(input, child)
    }
}
