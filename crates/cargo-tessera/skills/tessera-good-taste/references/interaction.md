# Interaction

- Pass actions directly as ordinary closures that update state, operate a controller, or start owned work. Let subsequent UI construction reflect the result.
- Create `Callback` / `CallbackWith` handles only for independent storage, sharing, or forwarding; pass existing handles through the matching `_shared` setter.
- Use existing interactive controls and semantic interaction modifiers. When handling raw input, consume the events you handle.
- Provide control names, roles, values, selection, and keyboard actions together. Publish selection and checked state according to their distinct meanings.
- Use `FocusRequester` and the `focus` module functions for focus changes. Request traversal that can reveal virtualized content; define initial focus and restoration for overlays and navigation.
- Use the text editor's selection, IME, clipboard, and documented transform interfaces. Apply asynchronous editing results to the matching current content.
