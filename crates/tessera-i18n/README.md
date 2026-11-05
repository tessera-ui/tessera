# tessera-i18n

Fluent messages with a language controller owned by the Tessera entry.

Configure `tessera-config.toml`:

```toml
[i18n]
fallback = "en"
```

Place feature files under `locales/en/` and `locales/zh-CN/`. Use the scaffold's
`tessera_build::build()` call and generated `res` bindings:

```rust
EntryPoint::new(app).package(res::i18n())
```

Resolve messages near their output:

```rust
text(res::I18N.text("home-title"));
text(res::I18N.format("unread-messages", [("count", 3.into())]));
```

```fluent
unread-messages = { $count ->
    [one] One unread message
   *[other] { $count } unread messages
}
```

Switch language with `I18n::current().set_locale(langid!("zh-CN"))`. Each entry
owns its language independently. Library catalogs use that language and accept
application overrides; missing messages follow the catalog's fallback policy.
