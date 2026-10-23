//! Native time-picker figure surface regressions.
//!
//! ## Usage
//!
//! Keep the dialog background around its content padding.

use std::cell::Cell;

use tessera_components::theme::material_theme;
use tessera_ui::{HeadlessConfig, HeadlessRenderer, RenderModule, State, tessera};

thread_local! {
    pub(super) static GEOMETRY: Cell<Option<State<super::Geometry>>> = const { Cell::new(None) };
}

struct Pipelines;

impl RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        super::time_figure(
            false,
            false,
            false,
            super::FigureKind::Specs,
            "Surface".into(),
        );
    });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn annotated_time_surface_paints_its_24dp_content_inset() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 900,
            height: 800,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    let geometry = GEOMETRY.get().unwrap().get();
    let container = geometry["container"];
    let headline = geometry["headline"];
    assert_eq!(container.width.0, 328);
    assert_eq!(headline.x.0 - container.x.0, 24);
    let pixels = renderer.read_pixels().unwrap();
    let y = (container.y.0 + container.height.0 / 2) as usize;
    let pixel = |x: i32| &pixels[(y * 900 + x as usize) * 4..][..4];
    assert_ne!(
        pixel(container.x.0 + 8),
        pixel(container.x.0 - 8),
        "content padding must be inside the painted dialog surface"
    );
}
