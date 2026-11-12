# Tessera diagnostic codes

Numbers are append-only. Never delete, renumber, or reuse an assigned code;
retired codes retain their explanations. Add the next unused number and its document.

- [T0001](T0001.md): component call outside a Tessera build context.
- [T0002](T0002.md): runtime API call outside a Tessera build context.
- [T0003](T0003.md): unresolved Tessera-sensitive call.
- [T0004](T0004.md): invalid Tessera lint configuration.
- [T0005](T0005.md): private Tessera macro-support item outside generated code.
- [T0006](T0006.md): model state created outside a model initializer.

Read an explanation with `cargo tessera check --explain T0001`.
