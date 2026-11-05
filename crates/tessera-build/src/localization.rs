use std::path::Path;

use anyhow::{Context, Result, ensure};
use fluent_templates::{ArcLoader, LanguageIdentifier, Loader};
use proc_macro2::{Literal, TokenStream};
use quote::quote;

use crate::I18nConfig;

pub(super) fn generate(root: &Path, config: Option<&I18nConfig>) -> Result<TokenStream> {
    let Some(config) = config else {
        return Ok(TokenStream::new());
    };
    let directory = root.join(config.dir.as_deref().unwrap_or("locales"));
    println!("cargo:rerun-if-changed={}", directory.display());
    let fallback: LanguageIdentifier = config
        .fallback
        .parse()
        .context("Invalid i18n fallback language")?;
    let loader = ArcLoader::builder(&directory, fallback.clone())
        .build()
        .with_context(|| format!("Invalid localization catalogs in {}", directory.display()))?;
    ensure!(
        loader.locales().any(|locale| *locale == fallback),
        "Localization fallback `{fallback}` has no catalog"
    );
    let directory = Literal::string(&directory.to_string_lossy());
    let fallback = Literal::string(&config.fallback);
    Ok(quote! {
        mod __tessera_i18n {
            use tessera_i18n::{Catalog, fluent_templates, langid};

            fluent_templates::static_loader! {
                static LOCALES = {
                    locales: #directory,
                    fallback_language: #fallback,
                };
            }

            /// Embedded messages formatted in the current entry's language.
            pub static I18N: std::sync::LazyLock<Catalog> = std::sync::LazyLock::new(|| {
                Catalog::new(&*LOCALES, langid!(#fallback))
            });

            /// Registers this package's catalog and language controller with an entry.
            pub fn i18n() -> Catalog { I18N.clone() }
        }

        pub use __tessera_i18n::{I18N, i18n};
    })
}
