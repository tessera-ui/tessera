use std::cell::OnceCell;

use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Event, EventTarget, HtmlElement};

use crate::{
    Dp, Insets,
    winit::{platform::web::WindowExtWebSys, window::Window},
};

struct Observer {
    probe: HtmlElement,
    callback: Closure<dyn FnMut(Event)>,
    listeners: Vec<(EventTarget, &'static str)>,
}

impl Observer {
    fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("browser window unavailable")?;
        let document = window.document().ok_or("document unavailable")?;
        let probe = document.create_element("div")?.dyn_into::<HtmlElement>()?;
        probe.set_attribute("style", "position:fixed;visibility:hidden;pointer-events:none;width:0;height:0;padding:env(safe-area-inset-top,0px) env(safe-area-inset-right,0px) env(safe-area-inset-bottom,0px) env(safe-area-inset-left,0px)")?;
        document
            .document_element()
            .ok_or("document root unavailable")?
            .append_child(&probe)?;
        let dispatcher = crate::RuntimeSession::current()
            .ok_or("UI session unavailable")?
            .run(crate::dispatcher::current);
        let callback = Closure::new(move |_event: Event| {
            dispatcher.dispatch(|| {});
        });
        let mut observer = Self {
            probe,
            callback,
            listeners: Vec::new(),
        };
        let mut targets = vec![window.clone().unchecked_into::<EventTarget>()];
        if let Some(viewport) = window.visual_viewport() {
            targets.push(viewport.unchecked_into());
        }
        for target in targets {
            for event in ["resize", "scroll"] {
                target.add_event_listener_with_callback(
                    event,
                    observer.callback.as_ref().unchecked_ref(),
                )?;
                observer.listeners.push((target.clone(), event));
            }
        }
        Ok(observer)
    }

    fn read(&self, window: &Window) -> Option<(Insets, Insets)> {
        let browser = web_sys::window()?;
        let style = browser.get_computed_style(&self.probe).ok()??;
        let edge = |name| {
            style
                .get_property_value(name)
                .ok()?
                .trim()
                .strip_suffix("px")?
                .parse::<f64>()
                .ok()
        };
        let rect = window.canvas()?.get_bounding_client_rect();
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return None;
        }
        let viewport_width = browser.inner_width().ok()?.as_f64()?;
        let viewport_height = browser.inner_height().ok()?.as_f64()?;
        let size = window.inner_size();
        let x = size.width as f64 / rect.width() / window.scale_factor();
        let y = size.height as f64 / rect.height() / window.scale_factor();
        let system = Insets::new(
            Dp((edge("padding-left")? - rect.left()).clamp(0.0, rect.width()) * x),
            Dp((edge("padding-top")? - rect.top()).clamp(0.0, rect.height()) * y),
            Dp(
                (edge("padding-right")? - (viewport_width - rect.right())).clamp(0.0, rect.width())
                    * x,
            ),
            Dp(
                (edge("padding-bottom")? - (viewport_height - rect.bottom()))
                    .clamp(0.0, rect.height())
                    * y,
            ),
        );
        let ime = browser
            .visual_viewport()
            .filter(|viewport| (viewport.scale() - 1.0).abs() < 0.01)
            .map(|viewport| {
                Insets::bottom(Dp((rect.bottom()
                    - viewport.offset_top()
                    - viewport.height())
                .clamp(0.0, rect.height())
                    * y))
            })
            .unwrap_or_default();
        Some((system, ime))
    }
}

impl Drop for Observer {
    fn drop(&mut self) {
        for (target, event) in &self.listeners {
            let _ = target
                .remove_event_listener_with_callback(event, self.callback.as_ref().unchecked_ref());
        }
        self.probe.remove();
    }
}

pub(super) fn read(window: &Window) -> Option<(Insets, Insets)> {
    let observer = crate::local_runtime_resource::<OnceCell<Option<Observer>>>();
    observer
        .get_or_init(|| {
            Observer::new()
                .map_err(|error| tracing::warn!(?error, "window insets unavailable"))
                .ok()
        })
        .as_ref()?
        .read(window)
}
