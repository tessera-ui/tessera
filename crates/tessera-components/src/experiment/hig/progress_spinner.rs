use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, Modifier, State,
    accesskit::Role,
    launch,
    layout::{Stack, layout},
    modifier::ModifierCapabilityExt as _,
    remember, tessera, use_context, with_frame_time,
};

use crate::pipelines::perceptual_fill::PerceptualSpokesCommand;

use super::super::theme::{Appearance, HigTheme};

/// Size of an indeterminate [`progress_spinner`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SpinnerSize {
    /// 32 dp diameter.
    #[default]
    Regular,
    /// 16 dp diameter.
    Small,
    /// 10 dp diameter.
    Mini,
}

impl SpinnerSize {
    fn diameter(self) -> Dp {
        match self {
            Self::Regular => Dp(32.0),
            Self::Small => Dp(16.0),
            Self::Mini => Dp(10.0),
        }
    }

    fn spoke_profile(self) -> ([f64; 3], f32, f32) {
        match self {
            Self::Regular => ([8.0, 13.99, 1.814], 0.80, 0.597),
            Self::Small => ([4.052, 6.955, 0.717], 0.74, 0.752),
            Self::Mini => ([2.464, 4.285, 0.576], 0.67, 0.575),
        }
    }
}

const STEPS: u128 = 24;
const PERIOD: Duration = Duration::from_millis(800);

fn phase_at(elapsed: Duration) -> u8 {
    let period = PERIOD.as_nanos();
    (elapsed.as_nanos() % period * STEPS / period) as u8
}

fn spoke_weights(phase: u8, opacity: f32) -> [f32; 8] {
    // All sizes share the same light-up and fade envelope. The offsets keep
    // the peak aligned to each spoke within a discrete animation step.
    const PEAK_OFFSETS: [f32; 8] = [-0.09, 0.11, 0.03, -0.24, -0.40, -0.58, -0.40, -0.24];
    const RISE: f32 = 3.13;
    std::array::from_fn(|i| {
        let age = (f32::from(phase) - 3.0 * i as f32 - PEAK_OFFSETS[i]).rem_euclid(STEPS as f32);
        let fade = (age / (STEPS as f32 - RISE)).min((STEPS as f32 - age) / RISE);
        (1.0 - 0.888 * fade) * opacity
    })
}

#[derive(Clone, PartialEq)]
struct SpinnerPaint {
    size: SpinnerSize,
    tint: Color,
    phase: Option<State<u8>>,
}

impl DrawModifierNode for SpinnerPaint {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let mut metadata = context.render_input.metadata_mut();
        if let Some(size) = metadata.computed_data() {
            let extent = size.width.min(size.height).to_f32();
            if extent > 0.0 {
                let (profile, edge_width_px, opacity) = self.size.spoke_profile();
                let fit = (extent / self.size.diameter().to_pixels_f32()).min(1.0);
                let [inner, outer, radius] = profile.map(|value| Dp(value * f64::from(fit)));
                metadata
                    .fragment_mut()
                    .push_draw_command(PerceptualSpokesCommand {
                        tint: self.tint,
                        inner,
                        outer,
                        radius,
                        edge_width_px,
                        weights: spoke_weights(self.phase.map_or(0, |phase| phase.get()), opacity),
                    });
            }
        }
        content.draw(context.render_input);
    }
}

/// # progress_spinner
///
/// Shows that an operation with no known completion fraction is in progress.
///
/// ## Usage
///
/// Indicate activity while loading or synchronizing content.
///
/// ## Parameters
///
/// - `running` — whether the indicator animates.
/// - `displayed_when_stopped` — draw a stationary indicator when stopped; defaults to true.
/// - `size` — indicator diameter; defaults to regular.
/// - `modifier` — layout and additional semantics.
/// - `accessibility_label`, `accessibility_description` — optional spoken name and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::progress::progress_spinner;
/// use tessera_ui::{accesskit::Role, testing::UiTest};
///
/// let mut ui = UiTest::new().content(|| {
///     progress_spinner(true).accessibility_label("Syncing");
/// });
/// ui.frame();
/// let (_, node) = ui.accessibility_node("Syncing");
/// assert_eq!(node.role(), Role::ProgressIndicator);
/// assert_eq!(node.numeric_value(), None);
/// ```
#[tessera]
pub fn progress_spinner(
    running: bool,
    displayed_when_stopped: Option<bool>,
    size: Option<SpinnerSize>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let size = size.unwrap_or_default();
    let diameter = size.diameter();
    let mut modifier = modifier.unwrap_or_default().size(diameter, diameter);
    if running || displayed_when_stopped.unwrap_or(true) {
        let tint = if use_context::<HigTheme>()
            .is_some_and(|theme| theme.get().appearance == Appearance::Dark)
        {
            Color::WHITE
        } else {
            Color::BLACK
        };
        let phase = if running {
            let phase = remember(|| 0);
            launch((), async move {
                let mut origin = None;
                loop {
                    with_frame_time(|now| {
                        let start = *origin.get_or_insert(now);
                        phase.set_if_changed(phase_at(now.saturating_duration_since(start)));
                    })
                    .await;
                }
            });
            Some(phase)
        } else {
            None
        };
        modifier = modifier
            .semantics(SemanticsArgs {
                role: Some(Role::ProgressIndicator),
                label: accessibility_label,
                description: accessibility_description,
                ..Default::default()
            })
            .push_draw(SpinnerPaint { size, tint, phase });
    }
    layout(Stack).modifier(modifier);
}
