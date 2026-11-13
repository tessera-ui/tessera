use tessera_ui::{
    MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Action, Invalid, Node, Role, Toggled},
    tessera,
    testing::UiTest,
};

#[tessera]
fn fixture() {
    super::super::super::media::load_fonts();
    tessera_components::md3::theme::material_theme().child(|| {
        tessera_components::column::column().children(|| {
            super::states();
        });
    });
}

fn node(renderer: &UiTest, label: &str) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| crate::has_label(&node, label))
        .unwrap()
        .1
}

fn click(renderer: &mut UiTest, label: &str) {
    let bounds = node(renderer, label).bounds().unwrap();
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(60);
}

fn save_pixels(renderer: &UiTest, name: &str) {
    if let Some(directory) = std::env::var_os("TESSERA_MENU_QA_DIR") {
        let size = renderer.physical_size();
        tessera_ui::headless::write_png_file(
            std::path::PathBuf::from(directory).join(name),
            size.width,
            size.height,
            &renderer.read_pixels(),
        )
        .unwrap();
    }
}

#[test]
#[ignore = "requires a native GPU adapter for checkbox state illustrations"]
fn checkbox_states_preserve_sixty_native_controls_error_mixed_and_scaled_pointer_actions() {
    let mut renderer = UiTest::new()
        .viewport(1024, 1700)
        .rendered(crate::TestPipelines)
        .content(fixture);
    renderer.frames(8);
    save_pixels(&renderer, "checkbox-states-canonical-1024.png");
    let tree = renderer.accessibility_snapshot().unwrap();
    let controls = tree
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::CheckBox)
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 60);
    assert_eq!(
        controls
            .iter()
            .filter(|(_, node)| node.invalid() == Some(Invalid::True))
            .count(),
        30
    );
    assert_eq!(
        controls
            .iter()
            .filter(|(_, node)| node.toggled() == Some(Toggled::Mixed))
            .count(),
        20
    );
    assert_eq!(
        controls
            .iter()
            .filter(|(_, node)| node.is_disabled())
            .count(),
        12
    );
    let dark = node(&renderer, "Dark checkbox states").bounds().unwrap();
    assert_eq!(
        (dark.y0, dark.width(), dark.height()),
        (757.0, 1024.0, 757.0)
    );
    let first = node(&renderer, "Light Enabled selected checkbox state")
        .bounds()
        .unwrap();
    assert_eq!(
        (first.x0, first.y0, first.width(), first.height()),
        (95.0, 159.0, 48.0, 48.0)
    );
    click(&mut renderer, "Light Enabled mixed checkbox state");
    assert_eq!(
        node(&renderer, "Light Enabled mixed checkbox state").toggled(),
        Some(Toggled::True)
    );
    assert_eq!(
        node(&renderer, "Dark Enabled mixed checkbox state").toggled(),
        Some(Toggled::Mixed)
    );
    click(&mut renderer, "Light Disabled selected checkbox state");
    assert_eq!(
        node(&renderer, "Light Disabled selected checkbox state").toggled(),
        Some(Toggled::True)
    );
    let (id, _) = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| crate::has_label(&node, "Light Focused error mixed checkbox state"))
        .unwrap();
    assert!(renderer.accessibility_action(id, Action::Focus));
    renderer.frames(4);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, id);
    assert_eq!(
        node(&renderer, "Light Focused error mixed checkbox state").toggled(),
        Some(Toggled::Mixed)
    );
    assert_eq!(
        node(&renderer, "Light Focused error mixed checkbox state").invalid(),
        Some(Invalid::True)
    );
    renderer.resize(512, 1700);
    renderer.frames(8);
    save_pixels(&renderer, "checkbox-states-narrow-512.png");
    let narrow = node(&renderer, "Light Enabled selected checkbox state")
        .bounds()
        .unwrap();
    assert_eq!(
        (narrow.x0, narrow.y0, narrow.width(), narrow.height()),
        (47.5, 79.5, 24.0, 24.0)
    );
    click(&mut renderer, "Light Enabled selected checkbox state");
    assert_eq!(
        node(&renderer, "Light Enabled selected checkbox state").toggled(),
        Some(Toggled::False)
    );
    assert_eq!(
        node(&renderer, "Dark Enabled selected checkbox state").toggled(),
        Some(Toggled::True)
    );
}
