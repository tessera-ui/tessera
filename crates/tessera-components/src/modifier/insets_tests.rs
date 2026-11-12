use tessera_ui::{
    Dp, Insets, Modifier, State, WindowInsets, WindowInsetsController, remember, tessera,
    testing::{
        TestProbe, WorkPhase, layout_test, pump_layout_test_session, start_layout_test_session,
    },
};

use crate::{
    boxed::boxed,
    modifier::{ModifierExt as _, SemanticsArgs},
};

#[derive(Clone, Copy)]
struct Handles {
    top: State<Dp>,
    height: State<Dp>,
    insets: WindowInsetsController,
}

#[tessera]
fn fixture(observed: TestProbe<Option<Handles>>) {
    let top = remember(|| Dp(10.0));
    let height = remember(|| Dp(10.0));
    let insets = remember(|| {
        let insets = WindowInsetsController::current();
        insets.set(
            Insets::new(Dp(5.0), Dp(30.0), Dp(7.0), Dp(20.0)),
            Insets::bottom(Dp(80.0)),
        );
        insets
    })
    .get();
    observed.set(Some(Handles {
        top,
        height,
        insets,
    }));
    boxed()
        .modifier(
            Modifier::new()
                .required_size(Dp(200.0), Dp(200.0 + top.get().0))
                .window_insets_padding(WindowInsets::Fixed(Insets::top(top.get()))),
        )
        .children(move || {
            boxed()
                .modifier(Modifier::new().window_insets_padding(WindowInsets::SystemBars))
                .children(move || {
                    boxed()
                        .modifier(Modifier::new().size(Dp(10.0), height.get()).semantics(
                            SemanticsArgs {
                                test_tag: Some("leaf".into()),
                                ..Default::default()
                            },
                        ))
                        .children(|| {});
                });
            boxed()
                .modifier(Modifier::new().consume_window_insets(WindowInsets::SystemBars))
                .children(|| {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .width(Dp(200.0))
                                .window_insets_bottom_height(WindowInsets::SafeDrawing)
                                .semantics(SemanticsArgs {
                                    test_tag: Some("keyboard".into()),
                                    ..Default::default()
                                }),
                        )
                        .children(|| {});
                });
        });
}

#[test]
fn nested_insets_preserve_consumption_through_cached_layout_and_live_updates() {
    let observed = TestProbe::default();
    let mut session = start_layout_test_session(
        layout_test({
            let observed = observed.clone();
            move || {
                fixture(observed.clone());
            }
        })
        .viewport_px(300, 300),
    );
    let first = pump_layout_test_session(&mut session, 0);
    first.node("leaf").position(5, 30).size(10, 10);
    first.node("keyboard").size(200, 60);
    let handles = observed.get().unwrap();
    session.run(|| handles.top.set(Dp(20.0)));
    pump_layout_test_session(&mut session, 1)
        .node("leaf")
        .position(5, 30);
    session.run(|| handles.height.set(Dp(12.0)));
    pump_layout_test_session(&mut session, 2)
        .node("leaf")
        .position(5, 30)
        .size(10, 12);
    session.run(|| {
        handles.insets.set(
            Insets::new(Dp(5.0), Dp(50.0), Dp(7.0), Dp(20.0)),
            Insets::bottom(Dp(100.0)),
        )
    });
    let changed = pump_layout_test_session(&mut session, 3);
    changed.node("leaf").position(5, 50).size(10, 12);
    changed.node("keyboard").size(200, 80);
    assert_eq!(session.work().phase(WorkPhase::Build).count(), 0);
    pump_layout_test_session(&mut session, 4);
    assert_eq!(session.work().phase(WorkPhase::Measure).count(), 0);
}
