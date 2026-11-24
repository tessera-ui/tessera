use tessera_foundation::{
    modifier::ModifierExt as _,
    pointer_gestures::{TapGesture, TapGestureConfig, detect_tap_gestures},
};
use tessera_ui::{
    Callback, Color, Modifier,
    layout::{Stack, layout},
    tessera,
};

use crate::modifier::ModifierExt as _;

#[tessera]
pub(super) fn modal_scrim(color: Color, on_dismiss: Option<Callback>) {
    let mut modifier = Modifier::new().fill_max_size().background(color);
    if let Some(on_dismiss) = on_dismiss {
        modifier = modifier.pointer_input(on_dismiss, move |mut scope| async move {
            detect_tap_gestures(&mut scope, TapGestureConfig::default(), |gesture| {
                if matches!(gesture, TapGesture::Tapped(..)) {
                    on_dismiss.call();
                }
            })
            .await;
        });
    }
    layout(Stack).modifier(modifier);
}
