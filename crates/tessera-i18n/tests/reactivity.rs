use std::sync::LazyLock;

use tessera_i18n::{Catalog, I18n, fluent_templates, langid};
use tessera_ui::{
    EntryRegistry, State, remember, tessera,
    testing::{TestProbe, layout_test, pump_layout_test_session, start_layout_test_session},
};

fluent_templates::static_loader! {
    static LOCALES = {
        locales: "./tests/locales",
        fallback_language: "en",
    };
}

static CATALOG: LazyLock<Catalog> = LazyLock::new(|| Catalog::new(&*LOCALES, langid!("en")));

#[derive(Clone, Default, PartialEq)]
struct Observation {
    i18n: TestProbe<Option<I18n>>,
    counter: TestProbe<Option<State<usize>>>,
    output: TestProbe<String>,
    builds: TestProbe<usize>,
}

#[tessera]
fn reader(observation: Observation) {
    let counter = remember(|| 0_usize);
    observation.i18n.set(Some(I18n::current()));
    observation.counter.set(Some(counter));
    observation
        .output
        .set(CATALOG.format("value", [("count", counter.get().into())]));
    observation.builds.set(observation.builds.get() + 1);
}

#[test]
fn entry_locale_changes_recompose_readers_preserve_state_and_isolate_sessions() {
    let first = Observation::default();
    let second = Observation::default();
    let mut session = start_layout_test_session(layout_test(|| reader(first.clone())));
    session.run(|| EntryRegistry::new().register_package(CATALOG.clone()));
    pump_layout_test_session(&mut session, 0);
    let initial = first.output.get();
    let locale = first.i18n.get().unwrap();
    let counter = first.counter.get().unwrap();
    let mut other = start_layout_test_session(layout_test(|| reader(second.clone())));
    other.run(|| EntryRegistry::new().register_package(CATALOG.clone()));
    pump_layout_test_session(&mut other, 0);
    let other_initial = second.output.get();
    session.run(|| locale.set_locale(langid!("zh-CN")));
    pump_layout_test_session(&mut session, 16_666_667);
    assert_ne!(first.output.get(), initial);
    assert!(first.counter.get().unwrap() == counter);
    pump_layout_test_session(&mut other, 16_666_667);
    assert_eq!(second.output.get(), other_initial);
    assert_eq!(second.builds.get(), 1);
    let localized = first.output.get();
    let builds = first.builds.get();
    session.run(|| locale.set_locale(langid!("zh-CN")));
    pump_layout_test_session(&mut session, 33_333_334);
    assert_eq!(first.builds.get(), builds);
    session.run(|| counter.set(7));
    pump_layout_test_session(&mut session, 50_000_001);
    assert_ne!(first.output.get(), localized);
}
