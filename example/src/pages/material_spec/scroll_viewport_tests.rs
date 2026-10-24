//! Stable specification viewport and real scrollbar geometry.
//!
//! ## Usage
//!
//! Reserve a visible track before long specification content is measured.

use std::{cell::RefCell, env, process::Command, time::Duration};

use tessera_components::{
    boxed::boxed, modifier::ModifierExt as _, scrollable::ScrollableController,
    theme::material_theme,
};
use tessera_ui::{
    Dp, HeadlessConfig, HeadlessRenderer, LayoutCoordinates, Modifier, accesskit::Role,
    modifier::FocusModifierExt as _, remember, tessera,
};

use super::spec_scroll_viewport;

thread_local! {
    static GEOMETRY: RefCell<Vec<LayoutCoordinates>> = const { RefCell::new(Vec::new()) };
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        viewport_fixture();
    });
}

#[tessera]
fn viewport_fixture() {
    let controller = remember(ScrollableController::new);
    spec_scroll_viewport(controller)
        .viewport_modifier(Modifier::new().on_layout_coordinates(|coordinates| {
            GEOMETRY.with_borrow_mut(|values| values.push(coordinates));
        }))
        .children(|| {
            boxed().modifier(Modifier::new().fill_max_width().height(Dp(1200.0)));
        });
}

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[test]
#[ignore = "requires a native GPU adapter for first-frame viewport and track geometry"]
fn gallery_track_reserves_exact_width_on_first_frame_and_after_resize() {
    let name = "pages::material_spec::scroll_viewport::tests::gallery_track_reserves_exact_width_on_first_frame_and_after_resize";
    if env::var("TESSERA_SPEC_SCROLL_TEST").as_deref() != Ok(name) {
        let output = Command::new(env::current_exe().unwrap())
            .args([
                "--exact",
                name,
                "--include-ignored",
                "--test-threads=1",
                "--nocapture",
            ])
            .env("TESSERA_SPEC_SCROLL_TEST", name)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("test result: ok. 1 passed;"));
        return;
    }
    for (density, initial_width, resized_width, thickness) in
        [(1.0, 892, 585, 15.0), (2.0, 877, 570, 30.0)]
    {
        GEOMETRY.with_borrow_mut(Vec::clear);
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 907,
                height: 320,
                scale_factor: density,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(1);
        let first = GEOMETRY.with_borrow(|values| values[0]);
        assert_eq!(first.local_size.width.0, initial_width);
        renderer.render_frames(8);
        assert!(GEOMETRY.with_borrow(|values| {
            values
                .iter()
                .all(|value| value.local_size.width.0 == initial_width)
        }));
        let track = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.role() == Role::ScrollBar)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert_eq!(
            (track.x0, track.width(), track.height()),
            (f64::from(initial_width), thickness, 320.0)
        );
        GEOMETRY.with_borrow_mut(Vec::clear);
        renderer.resize(600, 240);
        renderer.render_frames(1);
        assert_eq!(
            GEOMETRY.with_borrow(|values| values[0].local_size.width.0),
            resized_width
        );
        renderer.render_frames(8);
        assert!(GEOMETRY.with_borrow(|values| {
            values
                .iter()
                .all(|value| value.local_size.width.0 == resized_width)
        }));
        let track = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.role() == Role::ScrollBar)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert_eq!(
            (track.x0, track.width(), track.height()),
            (f64::from(resized_width), thickness, 240.0)
        );
    }
}
