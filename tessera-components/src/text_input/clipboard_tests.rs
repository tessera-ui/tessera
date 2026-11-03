use tessera_ui::{
    Dp, HeadlessConfig, HeadlessRenderer, Modifier, State,
    clipboard::{ClipboardAction, ClipboardResponse, ClipboardWrite, request_clipboard_action},
    remember, tessera,
};

use crate::{
    column::column,
    modifier::ModifierExt as _,
    text_edit_core::TextSelection,
    text_input::{TextInputController, text_input},
    theme::material_theme,
};

#[derive(Clone, Default, PartialEq)]
struct Probe {
    editor: tessera_ui::testing::TestProbe<Option<State<TextInputController>>>,
    other: tessera_ui::testing::TestProbe<Option<State<TextInputController>>>,
    read_only: tessera_ui::testing::TestProbe<Option<State<bool>>>,
    enabled: tessera_ui::testing::TestProbe<Option<State<bool>>>,
    changes: tessera_ui::testing::TestProbe<Vec<String>>,
    write: tessera_ui::testing::TestProbe<Option<ClipboardWrite>>,
    handled: tessera_ui::testing::TestProbe<bool>,
}

#[tessera]
fn fixture(probe: Probe) {
    let editor = remember(|| TextInputController::new(Dp(16.0), Some(Dp(24.0))));
    let other = remember(|| TextInputController::new(Dp(16.0), Some(Dp(24.0))));
    let read_only = remember(|| false);
    let enabled = remember(|| true);
    probe.editor.set(Some(editor));
    probe.other.set(Some(other));
    probe.read_only.set(Some(read_only));
    probe.enabled.set(Some(enabled));
    let changes = probe.changes.clone();
    let on_change = tessera_ui::CallbackWith::new(move |text: String| {
        changes.with_mut(|changes| changes.push(text.clone()));
        text.to_uppercase()
    });
    material_theme().child(move || {
        column().children(move || {
            text_input()
                .controller(editor)
                .modifier(Modifier::new().width(Dp(280.0)))
                .single_line(true)
                .enabled(enabled.get())
                .read_only(read_only.get())
                .input_transform(|text| text.replace(['\r', '\n'], ""))
                .on_change_shared(on_change);
            text_input().controller(other).single_line(true);
        });
    });
}

fn request(probe: &Probe, action: ClipboardAction) {
    probe.handled.set(false);
    probe.write.with_mut(|write| *write = None);
    let probe = probe.clone();
    request_clipboard_action(action, move |response| {
        probe
            .handled
            .set(!matches!(response, ClipboardResponse::Ignored));
        if let ClipboardResponse::Write(write) = response {
            probe.write.with_mut(|pending| *pending = Some(write));
        }
    });
}

fn arrange(probe: &Probe, editor: State<TextInputController>) {
    editor.with_mut(|editor| {
        editor.set_text_and_selection("abé🙂z", TextSelection { start: 8, end: 2 })
    });
    probe.changes.with_mut(Vec::clear);
}

fn check_clipboard() {
    let probe = Probe::default();
    let output = probe.clone();
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || fixture(output.clone())),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 480,
            height: 240,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(4);
    let editor = probe.editor.get().expect("editor");
    editor.with(|editor| editor.focus_handler().request_focus());
    renderer.render_frames(4);
    arrange(&probe, editor);
    request(&probe, ClipboardAction::Copy);
    renderer.render_frames(3);
    assert!(probe.handled.get());
    let write = probe.write.with_mut(Option::take).expect("copy write");
    assert_eq!(write.text(), "é🙂");
    write.complete(true);
    assert_eq!(editor.with(|editor| editor.text()), "abé🙂z");
    assert!(probe.changes.with(Vec::is_empty));

    request(&probe, ClipboardAction::Cut);
    renderer.render_frames(3);
    probe
        .write
        .with_mut(Option::take)
        .expect("denied cut")
        .complete(false);
    assert_eq!(editor.with(|editor| editor.text()), "abé🙂z");
    assert!(probe.changes.with(Vec::is_empty));

    request(&probe, ClipboardAction::Cut);
    renderer.render_frames(3);
    probe
        .write
        .with_mut(Option::take)
        .expect("accepted cut")
        .complete(true);
    assert_eq!(editor.with(|editor| editor.text()), "ABZ");
    assert_eq!(probe.changes.with(Clone::clone), ["abz"]);

    arrange(&probe, editor);
    request(&probe, ClipboardAction::Paste("xy\r\n".into()));
    renderer.render_frames(3);
    assert_eq!(editor.with(|editor| editor.text()), "ABXYZ");
    assert_eq!(probe.changes.with(Clone::clone), ["abxyz"]);
    assert!(editor.with(|editor| editor.selection().is_collapsed()));
    assert_eq!(
        probe.other.get().expect("other").with(|other| other.text()),
        ""
    );

    arrange(&probe, editor);
    request(&probe, ClipboardAction::Cut);
    renderer.render_frames(3);
    let stale = probe.write.with_mut(Option::take).expect("pending cut");
    request(&probe, ClipboardAction::Paste("new".into()));
    renderer.render_frames(3);
    stale.complete(true);
    assert_eq!(editor.with(|editor| editor.text()), "ABNEWZ");
    assert_eq!(probe.changes.with(Clone::clone), ["abnewz"]);

    probe.read_only.get().expect("read-only state").set(true);
    renderer.render_frames(3);
    arrange(&probe, editor);
    request(&probe, ClipboardAction::Paste("replace".into()));
    renderer.render_frames(3);
    assert!(probe.handled.get());
    assert_eq!(editor.with(|editor| editor.text()), "abé🙂z");
    request(&probe, ClipboardAction::Cut);
    renderer.render_frames(3);
    assert!(probe.write.with(Option::is_none));
    request(&probe, ClipboardAction::Copy);
    renderer.render_frames(3);
    assert_eq!(
        probe
            .write
            .with_mut(Option::take)
            .expect("read-only copy")
            .text(),
        "é🙂"
    );

    probe.enabled.get().expect("enabled state").set(false);
    renderer.render_frames(3);
    request(&probe, ClipboardAction::Paste("disabled".into()));
    renderer.render_frames(3);
    assert!(!probe.handled.get());
    assert_eq!(editor.with(|editor| editor.text()), "abé🙂z");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn focused_clipboard_edits_use_the_pipeline_and_acknowledge_writes() {
    crate::test_utils::run_isolated_test(
        "text_input::clipboard_tests::focused_clipboard_edits_use_the_pipeline_and_acknowledge_writes",
        check_clipboard,
    );
}
