use crate::{
    AccessibilityActionHandler, AccessibilityNode, ComputedData, Constraint, CursorEventContent,
    DrawModifierContent, DrawModifierContext, DrawModifierNode, FloatRect, FocusRequester,
    LayoutPolicy, LayoutResult, Modifier, PlacementModifierNode, PointerEventPass, PointerInput,
    PointerInputModifierNode, PressKeyEventType, Px, PxPosition, SemanticsModifierNode, State,
    accesskit::Role,
    layout::{MeasureScope, layout},
    modifier::FocusModifierExt as _,
    popup::{PopupPositionInput, PopupPositionPolicy, WindowOrigin, popup},
    remember, tessera,
    testing::{TestProbe, UiTest},
};

#[derive(Clone, PartialEq)]
struct Fixed(i32, i32);

impl LayoutPolicy for Fixed {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        let mut result = LayoutResult::new(ComputedData {
            width: Px(self.0),
            height: Px(self.1),
        });
        for child in scope.children() {
            child.measure(scope, &Constraint::NONE)?;
            result.place_child(child, PxPosition::ZERO);
        }
        Ok(result)
    }
}

/// Stacks children vertically, optionally leaving the first one unplaced.
#[derive(Clone, PartialEq)]
struct Column(bool);

impl LayoutPolicy for Column {
    fn measure(
        &self,
        scope: &mut MeasureScope<'_>,
    ) -> Result<LayoutResult, crate::MeasurementError> {
        let mut result = LayoutResult::new(ComputedData::ZERO);
        let mut y = Px(0);
        let mut width = Px(0);
        for (index, child) in scope.children().into_iter().enumerate() {
            let size = child.measure(scope, &Constraint::NONE)?.size();
            if index > 0 || self.0 {
                result.place_child(child, PxPosition::new(Px(0), y));
            }
            y += size.height;
            width = width.max(size.width);
        }
        result.size = ComputedData { width, height: y };
        Ok(result)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ClipChildren;

impl DrawModifierNode for ClipChildren {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        ctx.render_input.metadata_mut().set_clips_children(true);
        content.draw(ctx.render_input);
    }
}

#[derive(PartialEq)]
struct Offset(State<i32>);

impl PlacementModifierNode for Offset {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::new(Px(self.0.get()), Px(0))
    }
}

/// Places the popup below the anchor's start edge.
#[derive(Clone, Copy, PartialEq)]
struct Below;

impl PopupPositionPolicy for Below {
    fn position(&self, input: &PopupPositionInput) -> PxPosition {
        let anchor = input.anchor.window_bounds();
        PxPosition::from_f32_xy(anchor.x, anchor.y + anchor.height)
    }
}

type Log = TestProbe<Vec<&'static str>>;

struct Press(&'static str, Log);

impl PointerInputModifierNode for Press {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        for change in input.pointer_changes.iter_mut() {
            if !change.is_consumed() && matches!(change.content, CursorEventContent::Pressed(_)) {
                self.1.with_mut(|log| log.push(self.0));
                change.consume();
            }
        }
    }
}

struct Hover(TestProbe<usize>);

impl PointerInputModifierNode for Hover {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass == PointerEventPass::Main && input.cursor_position_rel.is_some() {
            self.0.with_mut(|count| *count += 1);
        }
    }
}

#[derive(Clone, PartialEq)]
struct Label(&'static str);

impl SemanticsModifierNode for Label {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.label = Some(self.0.to_owned());
        node.role = Some(Role::Button);
        node.focusable = true;
    }
}

#[derive(Clone, Default, PartialEq)]
struct Probe {
    log: Log,
    hover: TestProbe<usize>,
    open: TestProbe<Option<State<bool>>>,
    offset: TestProbe<Option<State<i32>>>,
    anchor_placed: TestProbe<Option<State<bool>>>,
    modal: TestProbe<Option<State<bool>>>,
    background: TestProbe<Option<FocusRequester>>,
}

fn press(ui: &mut UiTest, x: i32, y: i32) {
    ui.pointer_press(
        crate::MOUSE_POINTER_ID,
        PxPosition::new(Px(x), Px(y)),
        PressKeyEventType::Left,
    );
    ui.frame();
}

fn bounds(ui: &UiTest, name: &str) -> Option<FloatRect> {
    ui.inspect_components()
        .nodes()
        .iter()
        .find(|node| node.name() == name)
        .and_then(|node| node.bounds())
}

#[tessera]
fn panel(log: Log) {
    layout(Fixed(100, 50)).modifier(
        Modifier::new()
            .push_semantics(Label("Panel"))
            .push_pointer_input(Press("panel", log))
            .focusable(),
    );
}

#[tessera]
fn field(probe: Probe) {
    let open = remember(|| true);
    let offset = remember(|| 0);
    let modal = remember(|| false);
    probe.open.set(Some(open));
    probe.offset.set(Some(offset));
    probe.modal.set(Some(modal));
    let log = probe.log.clone();
    layout(Fixed(40, 20))
        .draw(ClipChildren)
        .modifier(
            Modifier::new()
                .push_placement(Offset(offset))
                .push_pointer_input(Press("field", probe.log.clone())),
        )
        .child(move || {
            if open.get() {
                field_menu(log.clone(), open, modal.get());
            }
        });
}

#[tessera]
fn field_menu(log: Log, open: State<bool>, modal: bool) {
    let dismissed = log.clone();
    popup(Below)
        .modal(modal)
        .on_dismiss_request(move || {
            crate::focus::defer_keyboard_until_layout();
            dismissed.with_mut(|log| log.push("dismiss"));
            open.set(false);
        })
        .child(move || panel(log.clone()));
}

#[tessera]
fn sibling(probe: Probe) {
    let background = remember(FocusRequester::new).get();
    probe.background.set(Some(background));
    layout(Fixed(200, 100)).modifier(
        Modifier::new()
            .push_semantics(Label("Sibling"))
            .push_pointer_input(Press("sibling", probe.log.clone()))
            .push_pointer_input(Hover(probe.hover.clone()))
            .focusable()
            .focus_requester(background),
    );
}

#[tessera]
fn screen(probe: Probe) {
    let anchor_placed = remember(|| true);
    probe.anchor_placed.set(Some(anchor_placed));
    layout(Column(anchor_placed.get())).child(move || {
        field(probe.clone());
        sibling(probe.clone());
    });
}

fn screen_test(probe: &Probe) -> UiTest<'static> {
    let content = probe.clone();
    let mut ui = UiTest::new()
        .viewport(300, 200)
        .content(move || screen(content.clone()));
    ui.frame();
    ui
}

#[test]
fn popup_measures_against_the_window_and_wins_hits_over_later_siblings() {
    let probe = Probe::default();
    let mut ui = screen_test(&probe);

    // The clipping anchor is 40 px wide and its later sibling starts at y = 20.
    assert_eq!(
        bounds(&ui, "panel"),
        Some(FloatRect::from_position_size(
            PxPosition::new(Px(0), Px(20)),
            crate::PxSize::new(Px(100), Px(50)),
        ))
    );
    press(&mut ui, 80, 40);
    assert_eq!(probe.log.take(), vec!["panel"]);
}

#[test]
fn popup_follows_its_anchor_and_hides_while_the_anchor_is_unplaced() {
    let probe = Probe::default();
    let mut ui = screen_test(&probe);

    let offset = probe.offset.get().expect("offset");
    ui.run(|| offset.set(30));
    ui.frame();
    assert_eq!(bounds(&ui, "panel").map(|rect| rect.x), Some(30.0));

    let anchor_placed = probe.anchor_placed.get().expect("anchor placement");
    ui.run(|| anchor_placed.set(false));
    ui.frame();
    assert_eq!(bounds(&ui, "panel"), None);
    press(&mut ui, 50, 30);
    assert_eq!(probe.log.take(), vec!["sibling"]);
}

#[test]
fn outside_press_dismisses_and_reaches_content_beneath_a_non_modal_popup() {
    let probe = Probe::default();
    let mut ui = screen_test(&probe);

    press(&mut ui, 150, 90);
    assert_eq!(probe.log.take(), vec!["dismiss", "sibling"]);
    assert_eq!(bounds(&ui, "panel"), None);
}

#[test]
fn modal_popup_consumes_outside_presses_and_hides_background_hover() {
    let probe = Probe::default();
    let mut ui = screen_test(&probe);
    let modal = probe.modal.get().expect("modality");
    ui.run(|| modal.set(true));
    ui.frame();

    ui.pointer_move(crate::MOUSE_POINTER_ID, PxPosition::new(Px(150), Px(90)));
    ui.frame();
    assert_eq!(probe.hover.get(), 0);
    press(&mut ui, 150, 90);
    assert_eq!(probe.log.take(), vec!["dismiss"]);
}

#[test]
fn non_modal_popup_hides_content_beneath_it_from_hover() {
    let probe = Probe::default();
    let mut ui = screen_test(&probe);

    ui.pointer_move(crate::MOUSE_POINTER_ID, PxPosition::new(Px(80), Px(40)));
    ui.frame();
    assert_eq!(probe.hover.get(), 0);

    ui.pointer_move(crate::MOUSE_POINTER_ID, PxPosition::new(Px(150), Px(90)));
    ui.frame();
    assert!(probe.hover.get() > 0);
}

#[test]
fn escape_and_system_back_request_dismissal() {
    let probe = Probe::default();
    let mut ui = screen_test(&probe);
    ui.press_key(winit::keyboard::NamedKey::Escape);
    ui.frame();
    assert_eq!(probe.log.take(), vec!["dismiss"]);

    let open = probe.open.get().expect("open state");
    ui.run(|| open.set(true));
    ui.frame();
    ui.dispatch_system_back(crate::system_back::SystemBackEvent::Committed);
    ui.frame();
    assert_eq!(probe.log.take(), vec!["dismiss"]);
}

#[test]
fn modal_popup_takes_focus_hides_background_semantics_and_restores_focus() {
    let probe = Probe::default();
    let mut ui = screen_test(&probe);
    let background = probe.background.get().expect("background requester");
    ui.run(|| background.request_focus());
    ui.frame();
    assert!(ui.run(|| background.is_focused()));

    let modal = probe.modal.get().expect("modality");
    ui.run(|| modal.set(true));
    ui.frame();
    assert!(!ui.run(|| background.is_focused()));
    let labels: Vec<_> = ui
        .accessibility_snapshot()
        .expect("accessibility tree")
        .nodes
        .into_iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    assert!(labels.iter().any(|label| label == "Panel"));
    assert!(!labels.iter().any(|label| label == "Sibling"));

    ui.run(|| background.request_focus());
    ui.frame();
    assert!(!ui.run(|| background.is_focused()));

    let open = probe.open.get().expect("open state");
    ui.run(|| open.set(false));
    ui.frames(2);
    assert!(ui.run(|| background.is_focused()));
}

#[derive(Clone, Default, PartialEq)]
struct StackProbe {
    log: Log,
    first_modal: TestProbe<Option<State<bool>>>,
    second_open: TestProbe<Option<State<bool>>>,
}

#[tessera]
fn layer(second: bool, log: Log, modal: bool) {
    let name = if second { "second" } else { "first" };
    popup(WindowOrigin).modal(modal).child(move || {
        layout(Fixed(50, 50))
            .modifier(Modifier::new().push_pointer_input(Press(name, log.clone())));
    });
}

#[tessera]
fn stacked(probe: StackProbe) {
    let first_modal = remember(|| false);
    let second_open = remember(|| false);
    probe.first_modal.set(Some(first_modal));
    probe.second_open.set(Some(second_open));
    layout(Fixed(100, 100)).child(move || {
        if second_open.get() {
            layer(true, probe.log.clone(), false);
        }
        layer(false, probe.log.clone(), first_modal.get());
    });
}

#[test]
fn sibling_popups_stack_by_modality_then_opening_order() {
    let probe = StackProbe::default();
    let content = probe.clone();
    let mut ui = UiTest::new()
        .viewport(100, 100)
        .content(move || stacked(content.clone()));
    ui.frame();

    // Declared earlier but opened later, so it stacks above.
    let second_open = probe.second_open.get().expect("second popup");
    ui.run(|| second_open.set(true));
    ui.frame();
    press(&mut ui, 10, 10);
    assert_eq!(probe.log.take(), vec!["second"]);

    let first_modal = probe.first_modal.get().expect("first modality");
    ui.run(|| first_modal.set(true));
    ui.frame();
    press(&mut ui, 10, 10);
    assert_eq!(probe.log.take(), vec!["first"]);
}
