use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, Modifier,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, Toggled, TreeId},
    remember, tessera,
};

use crate::{modifier::ModifierExt as _, surface::surface, theme::material_theme};

use super::{MenuController, MenuItemSelectionMode, menu_group, menu_item, menu_provider};

#[tessera]
fn fixture() {
    let controller = remember(|| {
        let mut controller = MenuController::new();
        controller.open();
        controller
    });
    let choices = remember(|| [true, false]);
    let radio = remember(|| 0_usize);
    material_theme().child(move || {
        menu_provider()
            .controller(controller)
            .main_content(|| {
                surface()
                    .modifier(Modifier::new().fill_max_size())
                    .child(|| {});
            })
            .menu_content(move || {
                menu_group().children(move || {
                    menu_item()
                        .label("Visual selection action")
                        .selected(true)
                        .close_on_click(false)
                        .on_click(|| {});
                    for (index, label) in ["Nut-free", "Vegan"].into_iter().enumerate() {
                        menu_item()
                            .label(label)
                            .selection_mode(MenuItemSelectionMode::Checkbox)
                            .selected(choices.get()[index])
                            .close_on_click(false)
                            .on_click(move || {
                                choices.with_mut(|choices| choices[index] = !choices[index]);
                            });
                    }
                    for (index, label) in ["Standard", "Vibrant"].into_iter().enumerate() {
                        menu_item()
                            .label(label)
                            .selection_mode(MenuItemSelectionMode::Radio)
                            .selected(radio.get() == index)
                            .close_on_click(false)
                            .on_click(move || radio.set(index));
                    }
                });
            });
    });
}

fn item(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .expect("menu semantics")
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.role() == role)
        .unwrap_or_else(|| panic!("missing {role:?} {label}"))
}

fn action(renderer: &mut HeadlessRenderer, label: &str, role: Role, action: Action) {
    let target_node = item(renderer, label, role).0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node,
        data: None,
    }));
    renderer.render_frames(8);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn explicit_menu_selection_roles_preserve_independent_checkbox_actions() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 600,
            height: 600,
            sample_count: 1,
            ..Default::default()
        },
    )
    .expect("native menu renderer");
    renderer.render_frames(20);
    let generic = item(&renderer, "Visual selection action", Role::MenuItem).1;
    assert_eq!(generic.toggled(), None);
    assert_eq!(generic.is_selected(), None);
    for (label, expected) in [("Nut-free", Toggled::True), ("Vegan", Toggled::False)] {
        let node = item(&renderer, label, Role::MenuItemCheckBox).1;
        assert_eq!(node.toggled(), Some(expected));
        assert_eq!(node.is_selected(), None);
    }
    action(
        &mut renderer,
        "Vegan",
        Role::MenuItemCheckBox,
        Action::Focus,
    );
    assert_eq!(
        item(&renderer, "Vegan", Role::MenuItemCheckBox).1.toggled(),
        Some(Toggled::False)
    );
    action(
        &mut renderer,
        "Vegan",
        Role::MenuItemCheckBox,
        Action::Click,
    );
    for label in ["Nut-free", "Vegan"] {
        assert_eq!(
            item(&renderer, label, Role::MenuItemCheckBox).1.toggled(),
            Some(Toggled::True)
        );
    }
    action(
        &mut renderer,
        "Nut-free",
        Role::MenuItemCheckBox,
        Action::Click,
    );
    assert_eq!(
        item(&renderer, "Nut-free", Role::MenuItemCheckBox)
            .1
            .toggled(),
        Some(Toggled::False)
    );
    assert_eq!(
        item(&renderer, "Vegan", Role::MenuItemCheckBox).1.toggled(),
        Some(Toggled::True)
    );
    assert_eq!(
        item(&renderer, "Standard", Role::MenuItemRadio).1.toggled(),
        Some(Toggled::True)
    );
    action(&mut renderer, "Vibrant", Role::MenuItemRadio, Action::Click);
    assert_eq!(
        item(&renderer, "Standard", Role::MenuItemRadio).1.toggled(),
        Some(Toggled::False)
    );
    assert_eq!(
        item(&renderer, "Vibrant", Role::MenuItemRadio).1.toggled(),
        Some(Toggled::True)
    );
    assert_eq!(
        item(&renderer, "Vegan", Role::MenuItemCheckBox).1.toggled(),
        Some(Toggled::True)
    );
}
