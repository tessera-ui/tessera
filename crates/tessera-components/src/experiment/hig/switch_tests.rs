use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, FocusRequester, KeyboardEvent, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, Role, Toggled},
    layout::{Stack, layout},
    remember, scaled_box, tessera,
    testing::{TestProbe, UiTest},
    winit::{event::ElementState, keyboard::NamedKey},
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    image::{ImageData, image},
    modifier::ModifierExt as _,
};

use super::{
    super::{
        button::{button, button_scope},
        theme::{Appearance, HigTheme, hig_theme},
    },
    switch,
};

#[derive(Clone, Copy)]
struct Controls {
    value: State<bool>,
    enabled: State<bool>,
    accept: State<bool>,
    requests: State<Vec<bool>>,
    focus: FocusRequester,
    other_focus: FocusRequester,
    submits: State<u32>,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let value = remember(|| false);
    let enabled = remember(|| true);
    let accept = remember(|| true);
    let requests = remember(Vec::new);
    let focus = remember(FocusRequester::new).get();
    let other_focus = remember(FocusRequester::new).get();
    let submits = remember(|| 0);
    probe.set(Some(Controls {
        value,
        enabled,
        accept,
        requests,
        focus,
        other_focus,
        submits,
    }));
    button_scope().child(move |scope| {
        column()
            .modifier(Modifier::new().background(Color::WHITE))
            .children(move || {
                switch(value.get())
                    .enabled(enabled.get())
                    .focus_requester(focus)
                    .accessibility_label("Notifications")
                    .on_value_change(move |next| {
                        requests.with_mut(|requests| requests.push(next));
                        if accept.get() {
                            value.set(next);
                        }
                    })
                    .modifier(
                        Modifier::new()
                            .size(Dp(54.0), Dp(24.0))
                            .semantics(SemanticsArgs {
                                test_tag: Some("setting".into()),
                                ..Default::default()
                            }),
                    );
                button("Save")
                    .scope(scope)
                    .default_action(true)
                    .focus_requester(other_focus)
                    .on_click(move || submits.set(submits.get() + 1));
            });
    });
}

#[test]
fn pointer_release_uses_the_current_controlled_value_and_disabled_presses_cancel() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 120)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    ui.node("setting").click();
    assert!(controls.value.get());
    controls.value.set(false);
    controls.accept.set(false);
    ui.frames(1);
    ui.node("setting").click();
    ui.node("setting").click();
    assert!(!controls.value.get());
    assert_eq!(controls.requests.get(), [true, true, true]);
    controls.value.set(true);
    ui.frames(1);
    ui.node("setting").click();
    assert_eq!(controls.requests.get(), [true, true, true, false]);
    let bounds = ui.frames(1).node("setting").bounds();
    let inside = PxPosition::from_f32_xy(bounds.x.to_f32() + 10.0, bounds.y.to_f32() + 10.0);
    let outside = PxPosition::from_f32_xy(300.0, 110.0);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    ui.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    ui.frames(1);
    assert_eq!(controls.requests.get().len(), 4);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    controls.enabled.set(false);
    ui.frames(1);
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    ui.node("setting").click();
    assert_eq!(controls.requests.get().len(), 4);
    let (_, node) = ui.accessibility_node("Notifications");
    assert_eq!(node.role(), Role::Switch);
    assert_eq!(node.toggled(), Some(Toggled::True));
    assert!(node.is_disabled());
    assert!(!node.supports_action(Action::Click));
}

fn key_up(ui: &mut UiTest<'_>, key: NamedKey) {
    ui.inject_key_event(KeyboardEvent::named(key, ElementState::Released));
    ui.frames(1);
}

fn check_keyboard_and_pixels() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 120)
        .scale_factor(1.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(3);
    let controls = probe.get().unwrap();
    let off = ui.read_pixels();
    let (id, _) = ui.accessibility_node("Notifications");
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frames(2);
    assert!(controls.requests.get().is_empty());
    assert!(ui.run(|| controls.focus.is_focused()));
    assert_ne!(off, ui.read_pixels());
    let focused = ui.read_pixels();
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    assert!(controls.requests.get().is_empty());
    assert_ne!(focused, ui.read_pixels());
    let mut repeated = KeyboardEvent::named(NamedKey::Space, ElementState::Pressed);
    repeated.repeat = true;
    ui.inject_key_event(repeated);
    ui.frames(1);
    assert!(controls.requests.get().is_empty());
    key_up(&mut ui, NamedKey::Space);
    assert_eq!(controls.requests.get(), [true]);
    assert_eq!(
        ui.accessibility_node("Notifications").1.toggled(),
        Some(Toggled::True)
    );
    let frame = ui.frames(1);
    let bounds = frame.node("setting").bounds();
    let thumb = frame.node("switch_thumb").bounds();
    assert_eq!((bounds.width.0, bounds.height.0), (54, 24));
    assert_eq!(
        (
            thumb.x.0 - bounds.x.0,
            thumb.y.0 - bounds.y.0,
            thumb.width.0,
            thumb.height.0
        ),
        (20, 2, 32, 20)
    );
    let on = ui.read_pixels();
    assert_ne!(off, on);
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frames(2);
    assert_eq!(controls.requests.get(), [true, false]);
    assert_eq!(
        ui.accessibility_node("Notifications").1.toggled(),
        Some(Toggled::False)
    );
    ui.press_key(NamedKey::Enter);
    ui.frames(1);
    assert_eq!(controls.submits.get(), 1);
    key_up(&mut ui, NamedKey::Enter);
    assert_eq!(controls.requests.get(), [true, false]);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    ui.run(|| controls.other_focus.request_focus());
    ui.frames(1);
    key_up(&mut ui, NamedKey::Space);
    assert_eq!(controls.requests.get(), [true, false]);
    ui.run(|| controls.focus.request_focus());
    ui.frames(1);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    controls.enabled.set(false);
    ui.frames(1);
    key_up(&mut ui, NamedKey::Space);
    assert_eq!(controls.requests.get(), [true, false]);
    assert!(!ui.run(|| controls.focus.is_focused()));
    assert_ne!(focused, ui.read_pixels());
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch and pixels"]
fn keyboard_feedback_focus_cancellation_and_accessibility_use_the_switch() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::switch::tests::keyboard_feedback_focus_cancellation_and_accessibility_use_the_switch",
        check_keyboard_and_pixels,
    );
}

#[tessera]
fn disabled_track_fixture(probe: TestProbe<Option<State<bool>>>) {
    let active = remember(|| true);
    let background = remember(|| {
        let mut pixels = Vec::with_capacity(120 * 80 * 4);
        for y in 0..80 {
            for x in 0..120 {
                let gray = if (x / 8 + y / 8) % 2 == 0 { 51 } else { 204 };
                pixels.extend_from_slice(&[gray, gray, gray, 255]);
            }
        }
        ImageData::from_rgba(pixels, 120, 80)
    });
    probe.set(Some(active));
    boxed().alignment(Alignment::TopStart).children(move || {
        image()
            .painter(background.get())
            .modifier(Modifier::new().size(Dp(120.0), Dp(80.0)));
        switch(true)
            .enabled(false)
            .window_active(active.get())
            .accessibility_label("Disabled setting")
            .modifier(Modifier::new().offset(Dp(20.0), Dp(20.0)));
    });
}

fn check_disabled_track() {
    for samples in [1, 4] {
        let probe = TestProbe::default();
        let mut ui = UiTest::new()
            .viewport(120, 80)
            .scale_factor(1.0)
            .samples(samples)
            .rendered(crate::TesseraComponents)
            .content(|| disabled_track_fixture(probe.clone()));
        ui.frames(2);
        let active = probe.get().unwrap();
        let (_, node) = ui.accessibility_node("Disabled setting");
        assert!(node.is_disabled());
        assert_eq!(node.toggled(), Some(Toggled::True));
        assert!(!node.supports_action(Action::Click));
        let colored = ui.read_pixels();
        for (x, red) in [(26, 61), (34, 15)] {
            let pixel = ui.pixel(x, 32);
            assert!(
                pixel[0].abs_diff(red) <= 2,
                "disabled track must retain backdrop contrast: {pixel:?}"
            );
            assert!(
                pixel[2] > pixel[0].saturating_add(100),
                "disabled on-track loses its tint: {pixel:?}"
            );
        }
        active.set(false);
        ui.frame();
        for x in [26, 34] {
            let pixel = ui.pixel(x, 32);
            assert_eq!(&pixel[..3], &[pixel[0]; 3]);
        }
        assert_ne!(colored, ui.read_pixels());
        active.set(true);
        ui.frame();
        assert_eq!(colored, ui.read_pixels());
        ui.advance(Duration::from_secs(1));
        assert_eq!(colored, ui.read_pixels());
    }
}

#[test]
#[ignore = "requires a native GPU adapter for disabled track pixels"]
fn disabled_on_track_keeps_active_tint_over_checkers_and_restores_after_deactivation() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::switch::tests::disabled_on_track_keeps_active_tint_over_checkers_and_restores_after_deactivation",
        check_disabled_track,
    );
}

#[derive(Clone, Copy)]
struct LensControls {
    enabled: State<bool>,
    visible: State<bool>,
    value: State<bool>,
    background: State<bool>,
    requests: State<usize>,
}

fn stripe_image(width: u32, height: u32) -> ImageData {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..height {
        for x in 0..width {
            let gray = if x / 4 % 2 == 0 { 31 } else { 230 };
            pixels.extend_from_slice(&[gray, gray, gray, 255]);
        }
    }
    ImageData::from_rgba(pixels, width, height)
}

#[tessera]
fn lens_solid_backdrop() {
    layout(Stack).modifier(
        Modifier::new()
            .size(Dp(160.0), Dp(80.0))
            .background(Color::from_rgb(0.1, 0.6, 0.2)),
    );
}

#[tessera]
fn lens_fixture(probe: TestProbe<Option<LensControls>>) {
    let enabled = remember(|| true);
    let visible = remember(|| true);
    let value = remember(|| false);
    let background = remember(|| false);
    let requests = remember(|| 0);
    let stripes = remember(|| stripe_image(320, 160));
    probe.set(Some(LensControls {
        enabled,
        visible,
        value,
        background,
        requests,
    }));
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(Modifier::new().size(Dp(160.0), Dp(80.0)))
        .children(move || {
            if background.get() {
                lens_solid_backdrop();
            } else {
                image()
                    .painter(stripes.get())
                    .modifier(Modifier::new().size(Dp(160.0), Dp(80.0)));
            }
            if visible.get() {
                switch(value.get())
                    .enabled(enabled.get())
                    .on_value_change(move |_| requests.set(requests.get() + 1))
                    .accessibility_label("Lens setting")
                    .modifier(Modifier::new().offset(Dp(40.0), Dp(28.0)).semantics(
                        SemanticsArgs {
                            test_tag: Some("lens".into()),
                            ..Default::default()
                        },
                    ));
            }
        });
}

fn check_held_lens() {
    for samples in [1, 4] {
        let probe = TestProbe::default();
        let mut ui = UiTest::new()
            .viewport(320, 160)
            .scale_factor(2.0)
            .samples(samples)
            .rendered(crate::TesseraComponents)
            .content(|| lens_fixture(probe.clone()));
        ui.frames(3);
        let controls = probe.get().unwrap();
        let rest = ui.read_pixels();
        let inside = PxPosition::from_f32_xy(112.0, 80.0);
        ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
        ui.frames(18);
        let thumb = ui.frame().node("switch_thumb").bounds();
        assert_eq!(
            (thumb.x.0, thumb.y.0, thumb.width.0, thumb.height.0),
            (66, 49, 100, 62)
        );
        let held = ui.read_pixels();
        assert_ne!(rest, held);
        assert_eq!(controls.requests.get(), 0);
        let mut chromatic = false;
        for y in 49..62 {
            for x in 100..132 {
                let pixel = ui.pixel(x, y);
                chromatic |= pixel[0].abs_diff(pixel[2]) > 40;
            }
        }
        assert!(
            chromatic,
            "the gray backdrop must disperse into different channel samples at the lens rim"
        );
        for (x, y, expected) in [
            (112, 52, [47_u8, 86, 185]),
            (116, 52, [224, 187, 88]),
            (116, 56, [224, 204, 151]),
            (69, 80, [215, 210, 216]),
            (70, 80, [220, 219, 218]),
            (91, 49, [35, 25, 22]),
            (92, 49, [163, 148, 129]),
        ] {
            let actual = ui.pixel(x, y);
            for channel in 0..3 {
                assert!(
                    actual[channel].abs_diff(expected[channel]) <= 16,
                    "contour coverage, sequential refraction, and dispersion must preserve flat and curved rim samples: ({x}, {y}) {actual:?} vs {expected:?}"
                );
            }
        }
        assert!(
            ui.pixel(116, 80)[0] > ui.pixel(112, 80)[0] + 80,
            "the lens interior must transmit stripe contrast rather than an opaque thumb"
        );
        controls.background.set(true);
        ui.frame();
        assert_ne!(held, ui.read_pixels());
        controls.background.set(false);
        ui.frames(2);
        assert_eq!(held, ui.read_pixels());
        controls.value.set(true);
        ui.frame();
        assert_eq!(ui.frame().node("switch_thumb").bounds().x.0, 102);
        let held_on = ui.read_pixels();
        let outside = PxPosition::from_f32_xy(inside.x, 8.0);
        ui.pointer_move(MOUSE_POINTER_ID, outside);
        ui.frames(2);
        assert_eq!(held_on, ui.read_pixels());
        assert_eq!(controls.requests.get(), 0);
        ui.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
        ui.frame();
        assert_eq!(ui.frame().node("switch_thumb").bounds().width.0, 64);
        assert_eq!(controls.requests.get(), 0);
        ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
        ui.frames(18);
        controls.enabled.set(false);
        ui.frame();
        assert_eq!(ui.frame().node("switch_thumb").bounds().width.0, 64);
        ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
        ui.frame();
        assert_eq!(controls.requests.get(), 0);
        controls.enabled.set(true);
        controls.value.set(false);
        ui.frame();
        ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
        ui.frames(18);
        controls.visible.set(false);
        ui.frame();
        ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
        ui.frame();
        controls.visible.set(true);
        ui.frames(2);
        assert_eq!(controls.requests.get(), 0);
        assert_eq!(rest, ui.read_pixels());
    }
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 160)
        .scale_factor(1.5)
        .samples(4)
        .rendered(crate::TesseraComponents)
        .content(|| {
            let probe = probe.clone();
            scaled_box(1.25).child(move || lens_fixture(probe.clone()));
        });
    ui.frames(3);
    let controls = probe.get().unwrap();
    let bounds = ui.frame().node("lens").bounds();
    let inside = PxPosition::from_f32_xy(bounds.x.to_f32() + 16.0, bounds.y.to_f32() + 16.0);
    let rest = ui.read_pixels();
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(18);
    let lens = ui.frame().node("switch_thumb").bounds();
    assert!(lens.x < bounds.x && lens.y < bounds.y);
    assert!(lens.height > bounds.height);
    let held = ui.read_pixels();
    assert_ne!(rest, held);
    ui.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(310.0, 150.0));
    ui.frames(2);
    assert_eq!(held, ui.read_pixels());
    ui.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(310.0, 150.0),
        PressKeyEventType::Left,
    );
    ui.frames(2);
    assert_eq!(controls.requests.get(), 0);
    assert_eq!(rest, ui.read_pixels());
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(18);
    ui.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(310.0, 150.0));
    ui.frames(2);
    assert_eq!(held, ui.read_pixels());
    ui.pointer_move(MOUSE_POINTER_ID, inside);
    ui.frames(18);
    assert_eq!(held, ui.read_pixels());
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(2);
    assert_eq!(controls.requests.get(), 1);
    assert_eq!(rest, ui.read_pixels());
    ui.advance(Duration::from_secs(1));
    assert_eq!(rest, ui.read_pixels());

    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 160)
        .scale_factor(2.0)
        .samples(4)
        .rendered(crate::TesseraComponents)
        .content(|| {
            let probe = probe.clone();
            hig_theme()
                .theme(HigTheme {
                    appearance: Appearance::Dark,
                })
                .child(move || lens_fixture(probe.clone()));
        });
    ui.frames(3);
    let controls = probe.get().unwrap();
    controls.value.set(true);
    ui.frames(2);
    let rest = ui.read_pixels();
    ui.pointer_press(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(155.0, 80.0),
        PressKeyEventType::Left,
    );
    ui.frames(18);
    let pixel = ui.pixel(93, 80);
    assert!(
        pixel[..3]
            .iter()
            .zip([30_u8, 138, 254])
            .all(|(a, b)| a.abs_diff(b) <= 4),
        "the dark held-on track must match the native exposed accent: {pixel:?}"
    );
    ui.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(155.0, 8.0),
        PressKeyEventType::Left,
    );
    ui.frames(2);
    assert_eq!(controls.requests.get(), 0);
    assert_eq!(rest, ui.read_pixels());
}

#[test]
#[ignore = "requires a native GPU adapter for dispersive lens pixels"]
fn held_lens_expands_transmits_and_disperses_the_live_backdrop_and_cancels_on_disable_or_removal() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::switch::tests::held_lens_expands_transmits_and_disperses_the_live_backdrop_and_cancels_on_disable_or_removal",
        check_held_lens,
    );
}

#[tessera]
fn lens_pair_fixture(probe: TestProbe<Option<State<Dp>>>) {
    let offset = remember(|| Dp(130.0));
    let stripes = remember(|| stripe_image(440, 160));
    probe.set(Some(offset));
    layout(Stack)
        .modifier(Modifier::new().size(Dp(220.0), Dp(80.0)))
        .child(move || {
            image()
                .painter(stripes.get())
                .modifier(Modifier::new().size(Dp(220.0), Dp(80.0)));
            switch(false)
                .on_value_change(|_| {})
                .modifier(Modifier::new().offset(Dp(20.0), Dp(28.0)));
            switch(false)
                .on_value_change(|_| {})
                .modifier(Modifier::new().offset(offset.get(), Dp(28.0)));
        });
}

fn check_independent_lens_rasters() {
    let crop = |pixels: &[u8], x: usize| -> Vec<u8> {
        (16..144)
            .flat_map(|y| {
                pixels[(y * 440 + x) * 4..(y * 440 + x + 144) * 4]
                    .iter()
                    .copied()
            })
            .collect()
    };
    for samples in [1, 4] {
        let probe = TestProbe::default();
        let mut ui = UiTest::new()
            .viewport(440, 160)
            .scale_factor(2.0)
            .samples(samples)
            .rendered(crate::TesseraComponents)
            .content(|| lens_pair_fixture(probe.clone()));
        ui.frames(2);
        let rest = ui.read_pixels();
        let first = PxPosition::from_f32_xy(64.0, 80.0);
        let second = PxPosition::from_f32_xy(284.0, 80.0);
        ui.pointer_press(1, first, PressKeyEventType::Left);
        ui.frames(2);
        let first_held = crop(&ui.read_pixels(), 6);
        assert_ne!(first_held, crop(&rest, 6));
        ui.pointer_press(2, second, PressKeyEventType::Left);
        ui.frames(2);
        let both = ui.read_pixels();
        assert_eq!(first_held, crop(&both, 6));
        let second_held = crop(&both, 222);
        assert_ne!(second_held, crop(&rest, 222));
        probe.get().unwrap().set(Dp(131.0));
        ui.frame();
        assert_eq!(first_held, crop(&ui.read_pixels(), 6));
        assert_ne!(second_held, crop(&ui.read_pixels(), 222));
        probe.get().unwrap().set(Dp(130.0));
        ui.frame();
        assert_eq!(both, ui.read_pixels());
        ui.pointer_release(1, first, PressKeyEventType::Left);
        ui.frames(2);
        let after_release = crop(&ui.read_pixels(), 222);
        assert!(
            second_held == after_release,
            "releasing the other lens changed {} channels; maximum delta {}; first difference {:?}",
            second_held
                .iter()
                .zip(&after_release)
                .filter(|(a, b)| a != b)
                .count(),
            second_held
                .iter()
                .zip(&after_release)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap(),
            second_held
                .iter()
                .zip(&after_release)
                .enumerate()
                .find(|(_, (a, b))| a != b)
        );
        ui.pointer_release(2, second, PressKeyEventType::Left);
        ui.frames(2);
        assert_eq!(rest, ui.read_pixels());
    }
}

#[test]
#[ignore = "requires a native GPU adapter for simultaneous lens pixels"]
fn simultaneous_lenses_keep_independent_rasters_during_motion_and_release() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::switch::tests::simultaneous_lenses_keep_independent_rasters_during_motion_and_release",
        check_independent_lens_rasters,
    );
}
