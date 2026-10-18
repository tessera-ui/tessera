use std::cell::{Cell, RefCell};

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

thread_local! {
    static EDITOR: Cell<Option<State<TextInputController>>> = const { Cell::new(None) };
    static OTHER: Cell<Option<State<TextInputController>>> = const { Cell::new(None) };
    static READ_ONLY: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static ENABLED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static CHANGES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static WRITE: RefCell<Option<ClipboardWrite>> = const { RefCell::new(None) };
    static HANDLED: Cell<bool> = const { Cell::new(false) };
}

#[tessera]
fn fixture() {
    let editor = remember(|| TextInputController::new(Dp(16.0), Some(Dp(24.0))));
    let other = remember(|| TextInputController::new(Dp(16.0), Some(Dp(24.0))));
    let read_only = remember(|| false);
    let enabled = remember(|| true);
    EDITOR.set(Some(editor));
    OTHER.set(Some(other));
    READ_ONLY.set(Some(read_only));
    ENABLED.set(Some(enabled));
    material_theme().child(move || {
        column().children(move || {
            text_input()
                .controller(editor)
                .modifier(Modifier::new().width(Dp(280.0)))
                .single_line(true)
                .enabled(enabled.get())
                .read_only(read_only.get())
                .input_transform(|text| text.replace(['\r', '\n'], ""))
                .on_change(|text| {
                    CHANGES.with_borrow_mut(|changes| changes.push(text.clone()));
                    text.to_uppercase()
                });
            text_input().controller(other).single_line(true);
        });
    });
}

fn request(action: ClipboardAction) {
    HANDLED.set(false);
    WRITE.with_borrow_mut(|write| *write = None);
    request_clipboard_action(action, |response| {
        HANDLED.set(!matches!(response, ClipboardResponse::Ignored));
        if let ClipboardResponse::Write(write) = response {
            WRITE.with_borrow_mut(|pending| *pending = Some(write));
        }
    });
}

fn arrange(editor: State<TextInputController>) {
    editor.with_mut(|editor| {
        editor.set_text_and_selection("abé🙂z", TextSelection { start: 8, end: 2 })
    });
    CHANGES.with_borrow_mut(Vec::clear);
}

fn check_clipboard() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| fixture()),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 480,
            height: 240,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(4);
    let editor = EDITOR.get().expect("editor");
    editor.with(|editor| editor.focus_handler().request_focus());
    renderer.render_frames(4);
    arrange(editor);
    request(ClipboardAction::Copy);
    renderer.render_frames(3);
    assert!(HANDLED.get());
    let write = WRITE.with_borrow_mut(Option::take).expect("copy write");
    assert_eq!(write.text(), "é🙂");
    write.complete(true);
    assert_eq!(editor.with(|editor| editor.text()), "abé🙂z");
    assert!(CHANGES.with_borrow(Vec::is_empty));

    request(ClipboardAction::Cut);
    renderer.render_frames(3);
    WRITE
        .with_borrow_mut(Option::take)
        .expect("denied cut")
        .complete(false);
    assert_eq!(editor.with(|editor| editor.text()), "abé🙂z");
    assert!(CHANGES.with_borrow(Vec::is_empty));

    request(ClipboardAction::Cut);
    renderer.render_frames(3);
    WRITE
        .with_borrow_mut(Option::take)
        .expect("accepted cut")
        .complete(true);
    assert_eq!(editor.with(|editor| editor.text()), "ABZ");
    assert_eq!(CHANGES.with_borrow(Clone::clone), ["abz"]);

    arrange(editor);
    request(ClipboardAction::Paste("xy\r\n".into()));
    renderer.render_frames(3);
    assert_eq!(editor.with(|editor| editor.text()), "ABXYZ");
    assert_eq!(CHANGES.with_borrow(Clone::clone), ["abxyz"]);
    assert!(editor.with(|editor| editor.selection().is_collapsed()));
    assert_eq!(OTHER.get().expect("other").with(|other| other.text()), "");

    arrange(editor);
    request(ClipboardAction::Cut);
    renderer.render_frames(3);
    let stale = WRITE.with_borrow_mut(Option::take).expect("pending cut");
    request(ClipboardAction::Paste("new".into()));
    renderer.render_frames(3);
    stale.complete(true);
    assert_eq!(editor.with(|editor| editor.text()), "ABNEWZ");
    assert_eq!(CHANGES.with_borrow(Clone::clone), ["abnewz"]);

    READ_ONLY.get().expect("read-only state").set(true);
    renderer.render_frames(3);
    arrange(editor);
    request(ClipboardAction::Paste("replace".into()));
    renderer.render_frames(3);
    assert!(HANDLED.get());
    assert_eq!(editor.with(|editor| editor.text()), "abé🙂z");
    request(ClipboardAction::Cut);
    renderer.render_frames(3);
    assert!(WRITE.with_borrow(Option::is_none));
    request(ClipboardAction::Copy);
    renderer.render_frames(3);
    assert_eq!(
        WRITE
            .with_borrow_mut(Option::take)
            .expect("read-only copy")
            .text(),
        "é🙂"
    );

    ENABLED.get().expect("enabled state").set(false);
    renderer.render_frames(3);
    request(ClipboardAction::Paste("disabled".into()));
    renderer.render_frames(3);
    assert!(!HANDLED.get());
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
