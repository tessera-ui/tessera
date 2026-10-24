//! Accessible menu selection and labeling figures.
//!
//! ## Usage
//!
//! Select a state or inspect a file action's accessible name.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    icon::icon,
    material_icons::filled,
    menus::{
        MenuAnchor, MenuColorStyle, MenuController, MenuItemSelectionMode, menu_item, menu_provider,
    },
    modifier::ModifierExt as _,
    shape_def::Shape,
    surface::surface,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Dp, Modifier, Px, PxPosition, PxSize,
    accesskit::{HasPopup, Role},
    modifier::ModifierCapabilityExt as _,
    remember, tessera, use_context,
};

use super::{gap, prose, type_style};

#[derive(Clone)]
struct Disclosure(bool);

impl tessera_ui::SemanticsModifierNode for Disclosure {
    fn apply(
        &self,
        node: &mut tessera_ui::AccessibilityNode,
        _handler: &mut Option<tessera_ui::AccessibilityActionHandler>,
    ) {
        node.expanded = Some(self.0);
        node.has_popup = Some(HasPopup::Menu);
    }
}

#[tessera]
pub(super) fn state_selection() {
    let selected = remember(|| "AK");
    let controller = remember(|| {
        let mut controller = MenuController::new();
        controller.open();
        controller
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(360.0))
                .background_with_shape(
                    scheme.secondary_container,
                    Shape::rounded_rectangle(Dp(16.0)),
                ),
        )
        .children(move || {
            menu_provider()
                .controller(controller)
                .modifier(Modifier::new().width(Dp(232.0)))
                .anchor(MenuAnchor::new(
                    PxPosition::from_f32_xy(0.0, 0.0),
                    PxSize::new(Px::from(Dp(232.0)), Px::from(Dp(56.0))),
                ))
                .offset([Dp::ZERO, Dp::ZERO])
                .max_height(Px::from(Dp(148.0)))
                .container_color(scheme.surface_container_low)
                .main_content(move || {
                    boxed()
                        .modifier(Modifier::new().size(Dp(232.0), Dp(204.0)))
                        .children(move || {
                            surface()
                                .style(scheme.surface_container_highest.into())
                                .shape(Shape::RECTANGLE)
                                .accessibility_role(Role::Button)
                                .accessibility_label(format!("State: {}", selected.get()))
                                .modifier(Modifier::new().size(Dp(232.0), Dp(56.0)).push_semantics(
                                    Disclosure(controller.with(MenuController::is_open)),
                                ))
                                .on_click(move || controller.with_mut(MenuController::toggle))
                                .child(move || {
                                    boxed().modifier(Modifier::new().fill_max_size()).children(
                                        move || {
                                            column()
                                                .modifier(
                                                    Modifier::new()
                                                        .padding_symmetric(Dp(16.0), Dp(8.0)),
                                                )
                                                .children(move || {
                                                    text()
                                                        .content("State")
                                                        .style(type_style(12.0, 16.0))
                                                        .color(scheme.on_surface_variant);
                                                    text()
                                                        .content(selected.get())
                                                        .style(type_style(16.0, 24.0))
                                                        .color(scheme.on_surface);
                                                });
                                            boxed()
                                                .alignment(Alignment::CenterEnd)
                                                .modifier(
                                                    Modifier::new()
                                                        .fill_max_size()
                                                        .padding_symmetric(Dp(16.0), Dp::ZERO),
                                                )
                                                .children(move || {
                                                    icon()
                                                        .vector(filled::ARROW_DROP_DOWN_SVG)
                                                        .size(Dp(20.0))
                                                        .rotation(
                                                            if controller
                                                                .with(MenuController::is_open)
                                                            {
                                                                180.0
                                                            } else {
                                                                0.0
                                                            },
                                                        );
                                                });
                                            boxed()
                                                .alignment(Alignment::BottomStart)
                                                .modifier(Modifier::new().fill_max_size())
                                                .children(move || {
                                                    boxed().modifier(
                                                        Modifier::new()
                                                            .fill_max_width()
                                                            .height(Dp(1.0))
                                                            .background(scheme.on_surface_variant),
                                                    );
                                                });
                                        },
                                    );
                                });
                        });
                })
                .menu_content(move || {
                    for code in ["AL", "AK", "AZ", "AR", "CA", "CO"] {
                        menu_item()
                            .label(code)
                            .selected(selected.get() == code)
                            .selection_mode(MenuItemSelectionMode::Radio)
                            .color_style(MenuColorStyle::Standard)
                            .height(Dp(48.0))
                            .on_click(move || selected.set(code));
                    }
                });
        });
    gap(Dp(16.0));
    prose("Use multiple visual cues like color, shape, and icons to show that an item is selected");
}

#[tessera]
pub(super) fn labeling() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let notice = remember(|| "No action selected");
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(390.0))
                .clip_to_bounds()
                .background_with_shape(
                    scheme.secondary_container,
                    Shape::rounded_rectangle(Dp(16.0)),
                ),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(-64.0), Dp(56.0))
                        .size(Dp(384.0), Dp(360.0))
                        .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(24.0)))
                        .border_with_shape(
                            Dp(8.0),
                            scheme.outline_variant,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    column()
                        .modifier(Modifier::new().padding_symmetric(Dp(64.0), Dp(24.0)))
                        .children(move || {
                            text().content("My files").style(type_style(22.0, 28.0));
                            gap(Dp(16.0));
                            for name in ["Photos", "Recipes", "Work", "Documents", "Sketches"] {
                                boxed()
                                    .alignment(Alignment::CenterStart)
                                    .modifier(Modifier::new().height(Dp(48.0)))
                                    .children(move || {
                                        text().content(name).style(type_style(14.0, 20.0));
                                    });
                            }
                        });
                });
            surface()
                .style(scheme.surface_container_low.into())
                .accessibility_role(Role::Menu)
                .shape(Shape::rounded_rectangle(Dp(16.0)))
                .elevation(Dp(3.0))
                .modifier(
                    Modifier::new()
                        .offset(Dp(112.0), Dp(188.0))
                        .width(Dp(180.0)),
                )
                .child(move || {
                    column().children(move || {
                        for (label, glyph) in [
                            ("Preview", filled::VISIBILITY_SVG),
                            ("Share", filled::PERSON_ADD_SVG),
                            ("Get link", filled::LINK_SVG),
                        ] {
                            menu_item()
                                .label(label)
                                .leading_icon(glyph)
                                .height(Dp(48.0))
                                .on_click(move || notice.set(label));
                        }
                    });
                });
            boxed().modifier(
                Modifier::new()
                    .offset(Dp(112.0), Dp(188.0))
                    .size(Dp(180.0), Dp(48.0))
                    .border_with_shape(
                        Dp(3.0),
                        scheme.tertiary,
                        Shape::rounded_rectangle(Dp(12.0)),
                    ),
            );
            boxed().modifier(
                Modifier::new()
                    .offset(Dp(292.0), Dp(212.0))
                    .size(Dp(124.0), Dp(3.0))
                    .background(scheme.tertiary),
            );
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(416.0), Dp(156.0))
                        .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(10.0)))
                        .border_with_shape(
                            Dp(3.0),
                            scheme.tertiary,
                            Shape::rounded_rectangle(Dp(10.0)),
                        ),
                )
                .children(move || {
                    column()
                        .modifier(Modifier::new().padding_all(Dp(14.0)))
                        .children(move || {
                            text().content("Label").style(type_style(16.0, 24.0));
                            text()
                                .content("Preview")
                                .style(type_style(16.0, 24.0))
                                .color(scheme.tertiary);
                        });
                });
        });
    gap(Dp(16.0));
    prose("The menu item's accessibility label aligns with the UI text");
    gap(Dp(16.0));
    super::spec_table(
        vec![
            "Element".into(),
            "A11y label".into(),
            "Role (Web)".into(),
            "Role (Android Views)".into(),
            "Role (Jetpack Compose)".into(),
        ],
        vec![vec![
            "Menu item text".into(),
            "Preview".into(),
            "Menu item".into(),
            "Generic actionable element".into(),
            "Generic actionable element".into(),
        ]],
    );
    gap(Dp(16.0));
    prose(format!("Last file action: {}", notice.get()));
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use tessera_components::theme::material_theme;
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, accesskit::Toggled,
    };

    use super::*;

    struct Pipelines;

    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[tessera]
    fn fixture() {
        material_theme().child(state_selection);
    }

    fn click(renderer: &mut HeadlessRenderer, label: &str) {
        let snapshot = renderer.accessibility_snapshot().unwrap();
        let node = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap_or_else(|| panic!("missing {label}"));
        let bounds = node.1.bounds().unwrap();
        let position = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(20);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn state_dropdown_updates_trigger_and_retains_selected_state_after_reopening() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 650,
                height: 440,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(20);
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("AK")
                    && node.role() == Role::MenuItemRadio
                    && node.toggled() == Some(Toggled::True))
        );
        click(&mut renderer, "AZ");
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(snapshot.nodes.iter().any(
            |(_, node)| node.label() == Some("State: AZ") && node.is_expanded() == Some(false)
        ));
        assert!(
            !snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::MenuItemRadio)
        );
        click(&mut renderer, "State: AZ");
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("AZ")
                    && node.role() == Role::MenuItemRadio
                    && node.toggled() == Some(Toggled::True))
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("AK")
                    && node.role() == Role::MenuItemRadio
                    && node.toggled() == Some(Toggled::False))
        );
    }
}
