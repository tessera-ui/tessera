//! Fluent catalogs and entry-owned language selection for Tessera.
//!
//! Configure catalogs through `tessera-build`, register `res::i18n()` with
//! `EntryPoint::package`, and resolve messages through `res::I18N`.

use std::{
    borrow::Cow,
    cmp::Reverse,
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use fluent_bundle::{FluentArgs, FluentResource, concurrent::FluentBundle};
use tessera_ui::{EntryRegistry, ModelStore, State, TesseraPackage, local_runtime_resource};

pub use fluent_bundle::FluentValue;
pub use fluent_templates::{self, LanguageIdentifier, Loader, langid};

/// Named values supplied to a Fluent message.
pub type Arguments<'a> = HashMap<Cow<'static, str>, FluentValue<'a>>;

/// Fluent catalogs embedded in the binary and parsed per locale on first lookup.
///
/// Lookups try the best matching locales before the fallback locale.
pub struct EmbeddedLoader {
    locales: Vec<EmbeddedLocale>,
    fallback: LanguageIdentifier,
}

struct EmbeddedLocale {
    id: LanguageIdentifier,
    sources: &'static [&'static str],
    bundle: OnceLock<FluentBundle<FluentResource>>,
}

impl EmbeddedLoader {
    /// Creates a loader from Fluent sources grouped by locale identifier.
    ///
    /// Panics when an identifier is invalid, or later when a locale's sources
    /// fail to parse or define a message twice.
    pub fn new(
        locales: &'static [(&'static str, &'static [&'static str])],
        fallback: LanguageIdentifier,
    ) -> Self {
        Self {
            locales: locales
                .iter()
                .map(|&(id, sources)| EmbeddedLocale {
                    id: id.parse().expect("invalid locale identifier"),
                    sources,
                    bundle: OnceLock::new(),
                })
                .collect(),
            fallback,
        }
    }

    fn negotiate(&self, requested: &LanguageIdentifier) -> Vec<&EmbeddedLocale> {
        let mut available = self.locales.iter().collect::<Vec<_>>();
        let mut matches = Vec::new();
        for available_as_range in [false, true] {
            available.retain(|locale| {
                let matched = locale.id.matches(requested, available_as_range, false);
                if matched {
                    matches.push(*locale);
                }
                !matched
            });
        }
        matches.sort_by_key(|locale| {
            Reverse(
                usize::from(locale.id.script.is_some())
                    + usize::from(locale.id.region.is_some())
                    + locale.id.variants().len(),
            )
        });
        matches
    }

    fn format(locale: &EmbeddedLocale, id: &str, args: Option<&Arguments<'_>>) -> Option<String> {
        let bundle = locale.bundle.get_or_init(|| {
            let mut bundle = FluentBundle::new_concurrent(vec![locale.id.clone()]);
            for source in locale.sources {
                let resource = FluentResource::try_new((*source).to_owned())
                    .unwrap_or_else(|_| panic!("invalid Fluent source for {}", locale.id));
                bundle
                    .add_resource(resource)
                    .unwrap_or_else(|_| panic!("duplicate Fluent message for {}", locale.id));
            }
            bundle
        });
        let (message, attribute) = match id.split_once('.') {
            Some((message, attribute)) => (message, Some(attribute)),
            None => (id, None),
        };
        let message = bundle.get_message(message)?;
        let pattern = match attribute {
            Some(attribute) => message
                .attributes()
                .find(|candidate| candidate.id() == attribute)?
                .value(),
            None => message.value()?,
        };
        let args = args.map(|args| {
            args.iter()
                .map(|(key, value)| (key.as_ref(), value.clone()))
                .collect::<FluentArgs<'_>>()
        });
        let mut errors = Vec::new();
        let value = bundle.format_pattern(pattern, args.as_ref(), &mut errors);
        errors.is_empty().then(|| value.into_owned())
    }
}

impl Loader for EmbeddedLoader {
    fn lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&Arguments<'_>>,
    ) -> String {
        self.try_lookup_complete(lang, text_id, args)
            .unwrap_or_else(|| format!("Unknown localization key: {text_id:?}"))
    }

    fn try_lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&Arguments<'_>>,
    ) -> Option<String> {
        self.negotiate(lang)
            .into_iter()
            .find_map(|locale| Self::format(locale, text_id, args))
            .or_else(|| {
                self.locales
                    .iter()
                    .find(|locale| *lang != self.fallback && locale.id == self.fallback)
                    .and_then(|locale| Self::format(locale, text_id, args))
            })
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        Box::new(self.locales.iter().map(|locale| &locale.id))
    }
}

/// A shareable message catalog that can be registered as an entry package.
#[derive(Clone)]
pub struct Catalog {
    loader: Arc<dyn Loader + Send + Sync>,
    default_locale: LanguageIdentifier,
}

impl Catalog {
    /// Uses the loader's language negotiation and fallback policy.
    pub fn new(
        loader: impl Loader + Send + Sync + 'static,
        default_locale: LanguageIdentifier,
    ) -> Self {
        Self {
            loader: Arc::new(loader),
            default_locale,
        }
    }

    /// Sets the default locale for this catalog and its entry package.
    pub fn locale(mut self, locale: LanguageIdentifier) -> Self {
        self.default_locale = locale;
        self
    }

    /// Formats a message or attribute in the entry's current language.
    ///
    /// Application messages override library messages. Without an i18n package,
    /// this catalog uses its default locale. Missing messages display their IDs.
    pub fn text(&self, id: impl AsRef<str>) -> String {
        let id = id.as_ref();
        self.lookup(id, None).unwrap_or_else(|| id.to_owned())
    }

    /// Formats a complete message with named Fluent arguments.
    pub fn format<'a>(
        &self,
        id: &str,
        args: impl IntoIterator<Item = (&'static str, FluentValue<'a>)>,
    ) -> String {
        let args = args
            .into_iter()
            .map(|(key, value)| (Cow::Borrowed(key), value))
            .collect();
        self.lookup(id, Some(&args))
            .unwrap_or_else(|| id.to_owned())
    }

    fn lookup(&self, id: &str, args: Option<&Arguments<'_>>) -> Option<String> {
        if let Some(i18n) = I18n::try_current() {
            let (application, locale) = i18n.snapshot();
            if !Arc::ptr_eq(&application, &self.loader) {
                if let Some(value) = application.try_lookup_complete(&locale, id, args) {
                    return Some(value);
                }
            }
            self.loader.try_lookup_complete(&locale, id, args)
        } else {
            self.loader
                .try_lookup_complete(&self.default_locale, id, args)
        }
    }
}

struct Localization {
    catalog: Catalog,
    locale: LanguageIdentifier,
}

#[derive(Default)]
// The store owns the localization state for the session.
struct LocalizationModel(std::cell::OnceCell<(ModelStore, State<Localization>)>);

impl TesseraPackage for Catalog {
    fn register(self, _registry: &mut EntryRegistry) {
        let localization = local_runtime_resource::<LocalizationModel>();
        // The first registered catalog stays in effect for the session.
        if localization.0.get().is_none() {
            let store = ModelStore::new();
            let state = store
                .model(|| {
                    State::new(Localization {
                        locale: self.default_locale.clone(),
                        catalog: self,
                    })
                })
                .with(|state| *state);
            let _ = localization.0.set((store, state));
        }
    }
}

/// A copyable controller for the language owned by an application entry.
#[derive(Clone, Copy, PartialEq)]
pub struct I18n(State<Localization>);

impl I18n {
    /// Returns the current entry's localization controller.
    ///
    /// Panics when no localization catalog was registered with the entry.
    pub fn current() -> Self {
        Self::try_current().expect("register a localization catalog with EntryPoint::package")
    }

    /// Returns a controller when the current entry has a localization catalog.
    pub fn try_current() -> Option<Self> {
        local_runtime_resource::<LocalizationModel>()
            .0
            .get()
            .map(|(_, state)| I18n(*state))
    }

    /// Returns the requested locale, which may use catalog fallback.
    pub fn locale(self) -> LanguageIdentifier {
        self.0.with(|localization| localization.locale.clone())
    }

    /// Changes the locale and invalidates readers when it changes.
    pub fn set_locale(self, locale: LanguageIdentifier) {
        if self.locale() != locale {
            self.0.with_mut(|localization| localization.locale = locale);
        }
    }

    /// Lists available application catalog locales.
    pub fn locales(self) -> Vec<LanguageIdentifier> {
        let (loader, _) = self.snapshot();
        loader.locales().cloned().collect()
    }

    fn snapshot(self) -> (Arc<dyn Loader + Send + Sync>, LanguageIdentifier) {
        self.0.with(|localization| {
            (
                Arc::clone(&localization.catalog.loader),
                localization.locale.clone(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static LOCALES: &[(&str, &[&str])] = &[
        (
            "en",
            &[
                "greeting = Hello\n    .title = Welcome\n",
                "only-en = Fallback\n",
            ],
        ),
        ("zh-CN", &["greeting = 你好\n"]),
    ];

    #[test]
    fn embedded_locales_parse_on_first_lookup_and_fall_back_per_message() {
        let loader = EmbeddedLoader::new(LOCALES, langid!("en"));
        assert_eq!(
            loader
                .try_lookup(&langid!("en"), "greeting.title")
                .as_deref(),
            Some("Welcome")
        );
        assert!(loader.locales[1].bundle.get().is_none());
        assert_eq!(
            loader.try_lookup(&langid!("zh-CN"), "greeting").as_deref(),
            Some("你好")
        );
        assert_eq!(
            loader.try_lookup(&langid!("zh-CN"), "only-en").as_deref(),
            Some("Fallback")
        );
        assert_eq!(
            loader.try_lookup(&langid!("fr"), "greeting").as_deref(),
            Some("Hello")
        );
        assert_eq!(loader.try_lookup(&langid!("en"), "missing"), None);
    }
}
