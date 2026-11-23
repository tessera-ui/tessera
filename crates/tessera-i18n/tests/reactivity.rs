use std::sync::LazyLock;

use tessera_i18n::{Catalog, I18n, fluent_templates, langid};
use tessera_ui::{
    EntryRegistry, State, remember, tessera,
    testing::{TestProbe, UiTest},
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
}

#[tessera]
fn reader(observation: Observation) {
    let counter = remember(|| 0_usize);
    observation.i18n.set(Some(I18n::current()));
    observation.counter.set(Some(counter));
    observation
        .output
        .set(CATALOG.format("value", [("count", counter.get().into())]));
}

#[test]
fn entry_locale_changes_translate_existing_state_without_affecting_other_entries() {
    let text = |observation: &Observation| {
        observation
            .output
            .get()
            .replace(['\u{2068}', '\u{2069}'], "")
    };
    let first = Observation::default();
    let second = Observation::default();
    let mut session = UiTest::new().content(|| reader(first.clone()));
    session.run(|| EntryRegistry::new().register_package(CATALOG.clone()));
    session.frame();
    assert_eq!(text(&first), "Value: 0");
    let locale = first.i18n.get().unwrap();
    let counter = first.counter.get().unwrap();
    let mut other = UiTest::new().content(|| reader(second.clone()));
    other.run(|| EntryRegistry::new().register_package(CATALOG.clone()));
    other.frame();
    session.run(|| counter.set(7));
    session.frames(1);
    assert_eq!(text(&first), "Value: 7");

    session.run(|| locale.set_locale(langid!("zh-CN")));
    session.frames(1);
    other.frames(1);
    assert_eq!(text(&first), "数值：7");
    assert_eq!(text(&second), "Value: 0");
}
