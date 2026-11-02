# Interaction

- Handle actions by updating state, operating a controller, or starting owned work. Let subsequent UI construction reflect the result.
- Use existing interactive controls and semantic interaction modifiers. When handling raw input, preserve pointer identity, sample position, coordinate space, and event consumption.
- Provide control names, roles, values, selection, and keyboard actions together. Publish selection and checked state according to their distinct meanings.
- Use focus requesters and managers for focus changes. Request traversal that can reveal virtualized content; define initial focus and restoration for overlays and navigation.
- Use the text editor's selection, IME, clipboard, and documented transform interfaces. Apply asynchronous editing results to the matching current content and acknowledge successful cut writes before deletion.
