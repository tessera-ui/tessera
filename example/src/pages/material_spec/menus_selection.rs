//! Native single-selection and multiple-selection menu figures.
//!
//! ## Usage
//!
//! Compare exclusive and independent choices using live dietary preferences.

use std::fmt::Write as _;

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    icon::icon,
    material_icons::filled,
    menus::{MenuItemSelectionMode, menu_group, menu_item},
    modifier::{InteractionVisualState, ModifierExt as _, SemanticsArgs},
    text_field::TextFieldBuilder,
    text_input::TextInputController,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, LayoutCoordinates, Modifier, accesskit::Role, modifier::FocusModifierExt as _,
    remember, scaled_box, tessera, use_context,
};

use super::super::annotations;

const CHOICES: [&str; 5] = ["Gluten-free", "Kosher", "Nut-free", "Vegan", "Vegetarian"];

fn selected_text(selected: &[bool; 5]) -> String {
    CHOICES
        .iter()
        .enumerate()
        .filter_map(|(index, label)| selected[index].then_some(*label))
        .collect::<Vec<_>>()
        .join(", ")
}

#[tessera]
pub(super) fn selection_figure() {
    let width = remember(|| Dp(676.0));
    let scale = (width.get().0 / 676.0) as f32;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if width.get() != next {
                    width.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box(scale).child(|| {
                let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(
                        Modifier::new()
                            .required_size(Dp(676.0), Dp(418.0))
                            .background(scheme.surface)
                            .semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some("Diet selection comparison".into()),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(|| {
                        background();
                        selection_column(false);
                        selection_column(true);
                    });
            });
        });
}

#[tessera]
fn background() {
    let path = remember(|| {
        let mut path = String::new();
        for y in (12..418).step_by(8) {
            for x in (10..676).step_by(8) {
                write!(&mut path, "M{x} {y}h1v1h-1z").unwrap();
            }
        }
        path
    });
    annotations::lines(
        Dp(676.0),
        Dp(418.0),
        format!(
            "<path d='{}' fill='#79747e' fill-opacity='.5'/>",
            path.get()
        ),
    );
}

#[tessera]
fn selection_column(multiple: bool) {
    let theme = use_context::<MaterialTheme>().unwrap();
    let selected = remember(move || [false, false, true, multiple, false]);
    let initial_text = selected.with(selected_text);
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        controller.set_text(&initial_text);
        controller
    });
    let mode = if multiple { "Multiple" } else { "Single" };
    column()
        .modifier(
            Modifier::new()
                .offset(Dp(if multiple { 370.0 } else { 106.0 }), Dp(54.0))
                .width(Dp(200.0)),
        )
        .children(move || {
            boxed()
                .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!("{mode} selection Diet field")),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .children(move || {
                    TextFieldBuilder::outlined()
                        .controller(controller)
                        .label("Diet")
                        .accessibility_label(format!("{mode} selection Diet"))
                        .read_only(true)
                        .min_width(Dp::ZERO)
                        .modifier(Modifier::new().size(Dp(200.0), Dp(56.0)))
                        .presentation_state(InteractionVisualState::Focus)
                        .background_color(Color::TRANSPARENT)
                        .focus_background_color(Color::TRANSPARENT)
                        .cursor_color(Color::TRANSPARENT)
                        .focus_border_width(Dp(3.0))
                        .trailing_icon(|| {
                            icon().vector(filled::ARROW_DROP_UP_SVG).size(Dp(24.0));
                        });
                });
            boxed()
                .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                    role: Some(Role::Menu),
                    label: Some(format!("{mode} selection menu")),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .children(move || {
                    material_theme()
                        .theme(move || {
                            let mut theme = theme.get();
                            theme.typography.body_large.font_family = Some("Example Sans Display");
                            theme.typography.body_large.font_size = Dp(14.0);
                            theme.typography.body_large.line_height = Some(Dp(20.0));
                            theme.typography.body_large.letter_spacing = Dp::ZERO;
                            theme.typography.body_large.font_weight = 475;
                            theme
                        })
                        .child(move || {
                            menu_group().children(move || {
                                for (index, choice) in CHOICES.iter().enumerate() {
                                    menu_item()
                                        .label(*choice)
                                        .selected(selected.get()[index])
                                        .selection_mode(if multiple {
                                            MenuItemSelectionMode::Checkbox
                                        } else {
                                            MenuItemSelectionMode::Radio
                                        })
                                        .close_on_click(false)
                                        .on_click(move || {
                                            selected.with_mut(|selected| {
                                                if multiple {
                                                    selected[index] = !selected[index];
                                                } else {
                                                    selected.fill(false);
                                                    selected[index] = true;
                                                }
                                            });
                                            let text = selected.with(selected_text);
                                            controller
                                                .with_mut(|controller| controller.set_text(&text));
                                        });
                                }
                            });
                        });
                });
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        accesskit::{Action, ActionRequest, Node, NodeId, Toggled, TreeId},
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
        super::super::super::media::load_fonts();
        tessera_components::theme::material_theme().child(selection_figure);
    }

    fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.label() == Some(label) && node.role() == role)
            .unwrap_or_else(|| panic!("missing {label} {role:?}"))
    }

    fn click(renderer: &mut HeadlessRenderer, label: &str, role: Role) {
        let bounds = node(renderer, label, role).1.bounds().unwrap();
        let point = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(6);
    }

    fn value(renderer: &HeadlessRenderer, label: &str) -> String {
        node(renderer, label, Role::TextInput)
            .1
            .value()
            .unwrap_or_default()
            .to_owned()
    }

    fn items_in_reading_order(renderer: &HeadlessRenderer, label: &str, role: Role) -> Vec<Node> {
        let tree = renderer.accessibility_snapshot().unwrap();
        let root = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Menu && node.label() == Some(label))
            .expect("native selection menu")
            .0;
        let mut pending = vec![root];
        let mut items = Vec::new();
        while let Some(id) = pending.pop() {
            let node = &tree
                .nodes
                .iter()
                .find(|(node_id, _)| *node_id == id)
                .expect("published semantic child")
                .1;
            if node.role() == role {
                items.push(node.clone());
            }
            pending.extend(node.children().iter().rev().copied());
        }
        items
    }

    fn save(renderer: &HeadlessRenderer, name: &str) {
        if let Some(directory) = std::env::var_os("TESSERA_MENU_QA_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            let size = renderer.physical_size();
            tessera_ui::headless::write_png_file(
                directory.join(format!("{name}.png")),
                size.width,
                size.height,
                &renderer.read_pixels().unwrap(),
            )
            .unwrap();
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter for menu selection and scaled geometry"]
    fn diet_choices_keep_radio_and_checkbox_state_independent_and_scale_as_one_figure() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 676,
                height: 418,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        save(&renderer, "menu-diet-initial-canonical");
        for (role, count, label) in [
            (Role::MenuItemRadio, 1, "Single selection menu"),
            (Role::MenuItemCheckBox, 2, "Multiple selection menu"),
        ] {
            let items = items_in_reading_order(&renderer, label, role);
            assert_eq!(items.len(), 5);
            assert_eq!(
                items
                    .iter()
                    .filter(|node| node.toggled() == Some(Toggled::True))
                    .count(),
                count
            );
            assert_eq!(
                items
                    .iter()
                    .map(|node| node.label().unwrap())
                    .collect::<Vec<_>>(),
                CHOICES
            );
            assert!(
                items
                    .iter()
                    .all(|node| node.bounds().unwrap().height() == 48.0)
            );
        }
        assert_eq!(value(&renderer, "Single selection Diet"), "Nut-free");
        assert_eq!(
            value(&renderer, "Multiple selection Diet"),
            "Nut-free, Vegan"
        );
        click(&mut renderer, "Vegan", Role::MenuItemRadio);
        assert_eq!(value(&renderer, "Single selection Diet"), "Vegan");
        assert_eq!(
            node(&renderer, "Nut-free", Role::MenuItemRadio).1.toggled(),
            Some(Toggled::False)
        );
        assert_eq!(
            value(&renderer, "Multiple selection Diet"),
            "Nut-free, Vegan"
        );
        click(&mut renderer, "Nut-free", Role::MenuItemCheckBox);
        click(&mut renderer, "Gluten-free", Role::MenuItemCheckBox);
        assert_eq!(
            value(&renderer, "Multiple selection Diet"),
            "Gluten-free, Vegan"
        );
        assert_eq!(value(&renderer, "Single selection Diet"), "Vegan");
        let kosher = node(&renderer, "Kosher", Role::MenuItemCheckBox).0;
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: kosher,
            data: None,
        }));
        renderer.render_frames(4);
        assert_eq!(
            value(&renderer, "Multiple selection Diet"),
            "Gluten-free, Vegan"
        );
        renderer.resize(338, 418);
        renderer.render_frames(8);
        let field = node(&renderer, "Single selection Diet field", Role::Group)
            .1
            .bounds()
            .unwrap();
        assert_eq!(
            (field.x0, field.y0, field.width(), field.height()),
            (53.0, 27.0, 100.0, 28.0)
        );
        let menu = node(&renderer, "Single selection menu", Role::Menu)
            .1
            .bounds()
            .unwrap();
        assert_eq!(
            (menu.x0, menu.y0, menu.width(), menu.height()),
            (53.0, 55.0, 100.0, 122.0)
        );
        click(&mut renderer, "Vegetarian", Role::MenuItemRadio);
        assert_eq!(value(&renderer, "Single selection Diet"), "Vegetarian");
        assert_eq!(
            value(&renderer, "Multiple selection Diet"),
            "Gluten-free, Vegan"
        );
        click(&mut renderer, "Gluten-free", Role::MenuItemCheckBox);
        click(&mut renderer, "Vegan", Role::MenuItemCheckBox);
        assert_eq!(value(&renderer, "Multiple selection Diet"), "");
        assert_eq!(value(&renderer, "Single selection Diet"), "Vegetarian");
        let tree = renderer.accessibility_snapshot().unwrap();
        assert!(
            tree.nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::MenuItemCheckBox)
                .all(|(_, node)| node.toggled() == Some(Toggled::False))
        );
        save(&renderer, "menu-diet-narrow-after-selection");
    }
}
