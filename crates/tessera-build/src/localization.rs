use std::{
    fs,
    path::{Path, PathBuf},
};

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
    let locales = catalog_sources(&directory)?
        .into_iter()
        .map(|(locale, files)| {
            let locale = Literal::string(&locale);
            let files = files
                .iter()
                .map(|file| Literal::string(&file.to_string_lossy()));
            quote! { (#locale, &[#(include_str!(#files)),*]) }
        });
    let fallback = Literal::string(&config.fallback);
    Ok(quote! {
        mod __tessera_i18n {
            use tessera_i18n::{Catalog, EmbeddedLoader, langid};

            static LOCALES: &[(&str, &[&str])] = &[#(#locales),*];

            /// Embedded messages formatted in the current entry's language.
            pub static I18N: std::sync::LazyLock<Catalog> = std::sync::LazyLock::new(|| {
                Catalog::new(EmbeddedLoader::new(LOCALES, langid!(#fallback)), langid!(#fallback))
            });

            /// Registers this package's catalog and language controller with an entry.
            pub fn i18n() -> Catalog { I18N.clone() }
        }

        pub use __tessera_i18n::{I18N, i18n};
    })
}

fn catalog_sources(directory: &Path) -> Result<Vec<(String, Vec<PathBuf>)>> {
    let mut locales = Vec::<(String, Vec<PathBuf>)>::new();
    for entry in fs::read_dir(directory)
        .with_context(|| format!("Failed to read {}", directory.display()))?
    {
        let path = entry?.path();
        let (locale, files) = if path.is_dir() {
            let mut files = Vec::new();
            collect_ftl(&path, &mut files)?;
            (path.file_name(), files)
        } else if path.extension().is_some_and(|extension| extension == "ftl") {
            (path.file_stem(), vec![path.clone()])
        } else {
            continue;
        };
        let Some(locale) = locale.and_then(|name| name.to_str()) else {
            continue;
        };
        if locale.parse::<LanguageIdentifier>().is_err() {
            continue;
        }
        match locales.iter_mut().find(|(name, _)| name == locale) {
            Some((_, existing)) => existing.extend(files),
            None => locales.push((locale.to_owned(), files)),
        }
    }
    for (_, files) in &mut locales {
        files.sort();
    }
    locales.sort();
    Ok(locales)
}

fn collect_ftl(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("Failed to read {}", directory.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            collect_ftl(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "ftl") {
            files.push(path);
        }
    }
    Ok(())
}
