//! Opt-in diagnostics for Material theme generation and Web rendering.
//!
//! ## Usage
//!
//! Inspect raw theme roles and isolated native color patches outside production
//! pages.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use tessera_components::{
    boxed::boxed,
    md3::{
        navigation_rail::{navigation_rail, navigation_rail_item},
        surface::surface,
        theme::{MaterialTheme, material_theme},
    },
    modifier::ModifierExt as _,
    shape_def::Shape,
    spacer::spacer,
};
use tessera_ui::{Color, Dp, Modifier, PxRect, modifier::FocusModifierExt as _, tessera};
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Default)]
pub(crate) struct DiagnosticBounds(Mutex<BTreeMap<&'static str, [f32; 4]>>);

struct DiagnosticBoundsOwner(std::sync::Arc<DiagnosticBounds>);

impl Drop for DiagnosticBoundsOwner {
    fn drop(&mut self) {
        self.0.0.lock().unwrap().clear();
    }
}

const DIAGNOSTIC_SEED: Color = Color::from_rgb(0.1356333, 0.08021982, 0.3712377);

#[wasm_bindgen(
    inline_js = "export function md3_drawer_fixture_count() { const value = new URL(location.href).searchParams.get('drawer_figures'); return value === '1' || value === '2' ? Number(value) : 0; }"
)]
extern "C" {
    fn md3_drawer_fixture_count() -> usize;
}

/// A diagnostic application's observations and UI session.
#[wasm_bindgen]
pub struct Md3Diagnostics {
    session: tessera_ui::WeakRuntimeSession,
    bounds: Arc<DiagnosticBounds>,
    drawers: Arc<crate::pages::DrawerObservations>,
}

#[wasm_bindgen]
impl Md3Diagnostics {
    /// Returns measured bounds while this diagnostic application remains live.
    pub fn bounds(&self) -> Option<String> {
        self.session.run(|| bounds_json(&self.bounds))
    }

    /// Returns drawer observations while this diagnostic application remains live.
    pub fn drawer_observations(&self) -> Option<String> {
        self.session
            .run(|| crate::pages::diagnostic_drawer_observations(&self.drawers))
    }
}

#[derive(Default)]
struct DiagnosticRegistration(
    Mutex<
        Option<(
            std::sync::Weak<DiagnosticBounds>,
            std::sync::Weak<crate::pages::DrawerObservations>,
        )>,
    >,
);

#[tessera_ui::entry]
pub(crate) fn entry() -> tessera_ui::EntryPoint {
    let bounds = Arc::new(DiagnosticBounds::default());
    let drawers = Arc::new(crate::pages::DrawerObservations::default());
    let entry = tessera_ui::EntryPoint::new({
        let bounds = Arc::clone(&bounds);
        let drawers = Arc::clone(&drawers);
        move || {
            diagnostic_app(Arc::clone(&bounds), Arc::clone(&drawers));
        }
    });
    *entry
        .runtime_session()
        .resource::<DiagnosticRegistration>()
        .0
        .lock()
        .unwrap() = Some((Arc::downgrade(&bounds), Arc::downgrade(&drawers)));
    entry
}

impl Md3Diagnostics {
    pub(crate) fn bind(entry: &tessera_ui::EntryPoint) -> Self {
        let registration = entry.runtime_session().resource::<DiagnosticRegistration>();
        let probes = registration.0.lock().unwrap();
        let (bounds, drawers) = probes.as_ref().expect("registered diagnostic root");
        Self {
            session: entry.runtime_session().downgrade(),
            bounds: bounds.upgrade().expect("live diagnostic bounds owner"),
            drawers: drawers.upgrade().expect("live drawer observation owner"),
        }
    }
}

fn observe_bounds(name: &'static str, observations: Arc<DiagnosticBounds>) -> Modifier {
    Modifier::new().on_globally_positioned(move |bounds: PxRect| {
        observations.0.lock().unwrap().insert(
            name,
            [
                bounds.x.to_f32(),
                bounds.y.to_f32(),
                bounds.width.to_f32(),
                bounds.height.to_f32(),
            ],
        );
    })
}

/// Renders isolated, opaque color patches and an unmodified collapsed rail.
#[tessera]
pub(crate) fn diagnostic_app(
    #[prop(skip_eq)] bounds: Arc<DiagnosticBounds>,
    #[prop(skip_eq)] drawers: Arc<crate::pages::DrawerObservations>,
) {
    let _observations = tessera_ui::remember(|| DiagnosticBoundsOwner(Arc::clone(&bounds)));
    let count = md3_drawer_fixture_count();
    if count > 0 {
        material_theme().child(move || {
            crate::pages::diagnostic_drawer_figures(count, Arc::clone(&drawers));
        });
        return;
    }
    let theme = MaterialTheme::from_seed(DIAGNOSTIC_SEED, true);
    let surface_color = theme.color_scheme.surface;
    let secondary_container = theme.color_scheme.secondary_container;
    material_theme()
        .theme(move || theme.clone())
        .child(move || {
            let bounds = Arc::clone(&bounds);
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background(Color::WHITE)
                        .then(observe_bounds("viewport", Arc::clone(&bounds))),
                )
                .children(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(16.0), Dp(16.0))
                                .size(Dp(64.0), Dp(64.0))
                                .background(surface_color)
                                .then(observe_bounds("surface_shape_patch", Arc::clone(&bounds))),
                        )
                        .children(|| {});
                    surface()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(16.0), Dp(112.0))
                                .size(Dp(64.0), Dp(64.0))
                                .then(observe_bounds("surface_native_patch", Arc::clone(&bounds))),
                        )
                        .shape(Shape::rounded_rectangle(Dp::ZERO))
                        .style(surface_color.into())
                        .child(|| {});
                    diagnostic_rail(Arc::clone(&bounds));
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(16.0), Dp(208.0))
                                .size(Dp(64.0), Dp(64.0))
                                .background(secondary_container)
                                .then(observe_bounds(
                                    "secondary_container_patch",
                                    Arc::clone(&bounds),
                                )),
                        )
                        .children(|| {});
                    for (name, y, channel) in [
                        ("linear_one_over_255_patch", 304.0, 1.0 / 255.0),
                        ("linear_two_over_255_patch", 400.0, 2.0 / 255.0),
                    ] {
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .offset(Dp(16.0), Dp(y))
                                    .size(Dp(64.0), Dp(64.0))
                                    .background(Color::from_rgb(channel, channel, channel))
                                    .then(observe_bounds(name, Arc::clone(&bounds))),
                            )
                            .children(|| {});
                    }
                });
        });
}

/// Returns the last observed component bounds in window pixels as JSON.
///
/// Each rectangle is `[x, y, width, height]`. Values are geometry observations,
/// not pixel samples or an indication of GPU completion.
/// Gray patches use linear channels of 1/255 and 2/255, not sRGB byte values.
fn bounds_json(observations: &DiagnosticBounds) -> String {
    let observations = observations.0.lock().unwrap();
    let mut json = String::from(
        r#"{"units":"window_px","rect_encoding":"x,y,width,height","rail_boundary":"wrapper, not internal surface","bounds":{"#,
    );
    for (index, (name, bounds)) in observations.iter().enumerate() {
        if index != 0 {
            json.push(',');
        }
        json.push_str(&format!(
            "\"{name}\":[{},{},{},{}]",
            bounds[0], bounds[1], bounds[2], bounds[3]
        ));
    }
    json.push_str("}}");
    json
}

/// Returns raw linear-sRGB theme roles as a JSON string.
///
/// Enable the `md3-diagnostics` feature in a WebAssembly build to export this
/// function. Seed channels are linear RGB values, not sRGB byte values. The
/// result describes theme generation only, not draw uniforms, scene readback,
/// or the presented canvas. Non-finite input or output produces an error
/// object.
#[wasm_bindgen]
pub fn md3_theme_roles_linear(seed_r: f32, seed_g: f32, seed_b: f32, is_dark: bool) -> String {
    if ![seed_r, seed_g, seed_b].into_iter().all(f32::is_finite) {
        return r#"{"error":"seed channels must be finite"}"#.to_owned();
    }

    let theme = MaterialTheme::from_seed(Color::from_rgb(seed_r, seed_g, seed_b), is_dark);
    let scheme = theme.color_scheme;
    let roles = [
        ("surface", scheme.surface),
        ("surface_container_lowest", scheme.surface_container_lowest),
        ("surface_container_low", scheme.surface_container_low),
        ("surface_container", scheme.surface_container),
        ("surface_container_high", scheme.surface_container_high),
        (
            "surface_container_highest",
            scheme.surface_container_highest,
        ),
        ("secondary_container", scheme.secondary_container),
        ("on_secondary_container", scheme.on_secondary_container),
        ("secondary", scheme.secondary),
    ];
    for (name, color) in &roles {
        if ![color.r, color.g, color.b, color.a]
            .into_iter()
            .all(f32::is_finite)
        {
            return format!(
                "{{\"error\":\"theme returned non-finite channels\",\"role\":\"{name}\"}}"
            );
        }
    }

    let mut json = format!(
        "{{\"source\":\"MaterialTheme::from_seed\",\"color_space\":\"linear-srgb\",\"is_dark\":{is_dark},\"seed\":[{seed_r},{seed_g},{seed_b},1],\"roles\":{{"
    );
    for (index, (name, color)) in roles.into_iter().enumerate() {
        if index != 0 {
            json.push(',');
        }
        json.push_str(&format!(
            "\"{name}\":[{},{},{},{}]",
            color.r, color.g, color.b, color.a
        ));
    }
    json.push_str("}}");
    json
}

#[tessera]
fn diagnostic_rail(#[prop(skip_eq)] bounds: Arc<DiagnosticBounds>) {
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(112.0), Dp(16.0))
                .size(Dp(96.0), Dp(560.0))
                .then(observe_bounds("rail_wrapper", Arc::clone(&bounds))),
        )
        .children(move || {
            let bounds = Arc::clone(&bounds);
            navigation_rail()
                .expanded(false)
                .content(move |rail_scope| {
                    let bounds = Arc::clone(&bounds);
                    navigation_rail_item(rail_scope)
                        .label("Home")
                        .selected(true)
                        .icon(move || {
                            spacer().modifier(
                                Modifier::new()
                                    .size(Dp(24.0), Dp(24.0))
                                    .then(observe_bounds("rail_home_icon", Arc::clone(&bounds))),
                            );
                        });
                });
        });
}
