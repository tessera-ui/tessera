use tessera_ui::{
    KeyboardInput, KeyboardInputModifierNode, Modifier, PointerInput, PointerInputModifierNode,
    modifier::ModifierCapabilityExt as _,
};

struct ClosurePointerInputModifierNode<F> {
    handler: F,
}

impl<F> PointerInputModifierNode for ClosurePointerInputModifierNode<F>
where
    F: for<'a> Fn(PointerInput<'a>) + 'static,
{
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        (self.handler)(input);
    }
}

struct ClosureKeyboardInputModifierNode<F> {
    handler: F,
}

impl<F> KeyboardInputModifierNode for ClosureKeyboardInputModifierNode<F>
where
    F: for<'a> Fn(KeyboardInput<'a>) + 'static,
{
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        (self.handler)(input);
    }
}

pub(crate) fn with_pointer_input<F>(base: Modifier, handler: F) -> Modifier
where
    F: for<'a> Fn(PointerInput<'a>) + 'static,
{
    base.push_pointer_input(ClosurePointerInputModifierNode { handler })
}

pub(crate) fn with_keyboard_input<F>(base: Modifier, handler: F) -> Modifier
where
    F: for<'a> Fn(KeyboardInput<'a>) + 'static,
{
    base.push_keyboard_input(ClosureKeyboardInputModifierNode { handler })
}
