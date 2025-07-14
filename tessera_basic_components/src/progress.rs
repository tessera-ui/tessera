use derive_builder::Builder;
use tessera::{Color, ComputedData, Constraint, DimensionValue, Dp, Px, PxPosition, place_node};
use tessera_macros::tessera;

use crate::surface::{SurfaceArgsBuilder, surface};

/// Arguments for the `progress` component.
#[derive(Builder, Clone, Debug)]
#[builder(pattern = "owned")]
pub struct ProgressArgs {
    /// The current value of the progress bar, ranging from 0.0 to 1.0.
    #[builder(default = "0.0")]
    pub value: f32,

    /// The width of the progress bar.
    #[builder(default = "Dp(200.0)")]
    pub width: Dp,

    /// The height of the progress bar.
    #[builder(default = "Dp(8.0)")]
    pub height: Dp,

    /// The color of the active part of the track.
    #[builder(default = "Color::new(0.2, 0.5, 0.8, 1.0)")]
    pub progress_color: Color,

    /// The color of the inactive part of the track.
    #[builder(default = "Color::new(0.8, 0.8, 0.8, 1.0)")]
    pub track_color: Color,

    /// The corner radius of the progress bar.
    #[builder(default = "4.0")]
    pub corner_radius: f32,
}

#[tessera]
pub fn progress(args: impl Into<ProgressArgs>) {
    let args: ProgressArgs = args.into();

    // Child 1: The background track. It's drawn first.
    surface(
        SurfaceArgsBuilder::default()
            .color(args.track_color)
            .corner_radius(args.corner_radius)
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None,
        || {},
    );

    // Child 2: The progress fill. It's drawn on top of the track.
    surface(
        SurfaceArgsBuilder::default()
            .color(args.progress_color)
            .corner_radius(args.corner_radius)
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None,
        || {},
    );

    measure(Box::new(move |input| {
        let self_width = args.width.to_px();
        let self_height = args.height.to_px();

        let track_id = input.children_ids[0];
        let progress_id = input.children_ids[1];

        // Measure and place the background track to take the full size of the component.
        let track_constraint = Constraint::new(
            DimensionValue::Fixed(self_width),
            DimensionValue::Fixed(self_height),
        );
        tessera::measure_node(
            track_id,
            &track_constraint,
            input.tree,
            input.metadatas,
            input.compute_resource_manager.clone(),
            input.gpu,
        )?;
        place_node(track_id, PxPosition::new(Px(0), Px(0)), input.metadatas);

        // Measure and place the progress fill based on the `value`.
        let progress_width = Px((self_width.to_f32() * args.value.clamp(0.0, 1.0)) as i32);
        let progress_constraint = Constraint::new(
            DimensionValue::Fixed(progress_width),
            DimensionValue::Fixed(self_height),
        );
        tessera::measure_node(
            progress_id,
            &progress_constraint,
            input.tree,
            input.metadatas,
            input.compute_resource_manager.clone(),
            input.gpu,
        )?;
        place_node(progress_id, PxPosition::new(Px(0), Px(0)), input.metadatas);

        // The progress component itself is a container, its size is defined by the args.
        Ok(ComputedData {
            width: self_width,
            height: self_height,
        })
    }));
}
