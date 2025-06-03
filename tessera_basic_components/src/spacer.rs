use derive_builder::Builder;
use tessera::ComputedData;
use tessera_macros::tessera;

/// Arguments for the Spacer component.
#[derive(Default, Clone, Copy, Builder)]
pub struct SpacerArgs {
    /// The desired width of the spacer.
    /// If not specified, it defaults to 0.
    #[builder(default = "0")]
    pub width: u32,
    /// The desired height of the spacer.
    /// If not specified, it defaults to 0.
    #[builder(default = "0")]
    pub height: u32,
}

/// A component that creates an empty space in the layout.
///
/// `Spacer` can be used to add gaps between other components. Its size can be
/// defined by `width` and `height` parameters passed via `SpacerArgs`.
#[tessera]
pub fn spacer(args: SpacerArgs) {
    measure(Box::new(move |_, _, constraint, _, _| {
        let mut w = args.width;
        let mut h = args.height;

        // Apply constraints from the parent
        // Width
        w = w.max(constraint.min_width.unwrap_or(0));
        if let Some(max_w) = constraint.max_width {
            w = w.min(max_w);
        }

        // Height
        h = h.max(constraint.min_height.unwrap_or(0));
        if let Some(max_h) = constraint.max_height {
            h = h.min(max_h);
        }

        ComputedData {
            width: w,
            height: h,
        }
    }));
    // Spacer has no children, so the children rendering part is empty.
}
