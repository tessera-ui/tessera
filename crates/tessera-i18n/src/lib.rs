//! Fluent catalogs and entry-owned language selection for Tessera.
//!
//! Configure catalogs through `tessera-build`, register `res::i18n()` with
//! `EntryPoint::package`, and resolve messages through `res::I18N`.

use std::{borrow::Cow, collections::HashMap, sync::Arc};

use tessera_ui::{EntryRegistry, Plugin, State, StateOwner, TesseraPackage, try_with_plugin};

pub use fluent_bundle::FluentValue;
pub use fluent_templates::{self, LanguageIdentifier, Loader, langid};

/// Named values supplied to a Fluent message.
pub type Arguments<'a> = HashMap<Cow<'static, str>, FluentValue<'a>>;

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

struct LocalizationPlugin {
    _owner: StateOwner,
    current: I18n,
}

impl Plugin for LocalizationPlugin {}

impl TesseraPackage for Catalog {
    fn register(self, registry: &mut EntryRegistry) {
        let owner = StateOwner::new();
        let state = owner.state_with_key((), || Localization {
            locale: self.default_locale.clone(),
            catalog: self,
        });
        registry.register_plugin(LocalizationPlugin {
            _owner: owner,
            current: I18n(state),
        });
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
        try_with_plugin::<LocalizationPlugin, _>(|plugin| plugin.current)
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
