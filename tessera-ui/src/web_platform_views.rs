//! DOM elements composed between WebGPU canvas layers.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlElement};

use crate::render_graph::PlatformViewPlacement;

/// Creates and updates a DOM element hosted in Tessera's Web scene.
pub trait WebPlatformViewFactory: 'static {
    /// Creates the element inserted into the renderer's scene.
    fn create(&self) -> HtmlElement;

    /// Refreshes element content when the renderer updates its placement.
    fn update(&self, _element: &HtmlElement) {}
}

struct Entry {
    factory: Rc<dyn WebPlatformViewFactory>,
    element: HtmlElement,
}

/// DOM host and ordered canvas layers for a Web renderer.
pub(crate) struct WebPlatformViews {
    root: HtmlElement,
    entries: HashMap<u64, Entry>,
    slices: Vec<HtmlCanvasElement>,
}

impl WebPlatformViews {
    pub(crate) fn new(canvas: HtmlCanvasElement) -> Result<Rc<RefCell<Self>>, String> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| "Web document is unavailable".to_owned())?;
        let root = document
            .create_element("div")
            .map_err(|error| format!("Failed to create DOM scene root: {error:?}"))?
            .dyn_into::<HtmlElement>()
            .map_err(|_| "DOM scene root is not an HTML element".to_owned())?;
        root.set_attribute("data-tessera-scene", "")
            .map_err(|error| format!("Failed to mark DOM scene root: {error:?}"))?;
        let style = root.style();
        style
            .set_property("position", "relative")
            .map_err(js_error)?;
        style.set_property("overflow", "hidden").map_err(js_error)?;
        style
            .set_property("isolation", "isolate")
            .map_err(js_error)?;
        style.set_property("display", "block").map_err(js_error)?;
        let parent = canvas
            .parent_node()
            .ok_or_else(|| "Web canvas has no parent element".to_owned())?;
        parent
            .insert_before(&root, Some(&canvas))
            .map_err(|error| format!("Failed to mount DOM scene root: {error:?}"))?;
        root.append_child(&canvas)
            .map_err(|error| format!("Failed to move Web canvas into scene root: {error:?}"))?;
        let canvas_style = canvas.style();
        canvas_style
            .set_property("position", "relative")
            .map_err(js_error)?;
        canvas_style
            .set_property("display", "block")
            .map_err(js_error)?;
        canvas_style
            .set_property("z-index", "0")
            .map_err(js_error)?;
        canvas_style.set_property("margin", "0").map_err(js_error)?;
        Ok(Rc::new(RefCell::new(Self {
            root,
            entries: HashMap::new(),
            slices: Vec::new(),
        })))
    }

    pub(crate) fn register(
        &mut self,
        id: u64,
        factory: Rc<dyn WebPlatformViewFactory>,
    ) -> Result<(), String> {
        if self.entries.contains_key(&id) {
            return Ok(());
        }
        let element = factory.create();
        set_layer_style(&element, 1)?;
        self.root
            .append_child(&element)
            .map_err(|error| format!("Failed to mount platform view: {error:?}"))?;
        self.entries.insert(id, Entry { factory, element });
        Ok(())
    }

    pub(crate) fn unregister(&mut self, id: u64) {
        if let Some(entry) = self.entries.remove(&id) {
            entry.element.remove();
        }
    }

    pub(crate) fn create_slice_canvas(
        &mut self,
        size: [u32; 2],
    ) -> Result<HtmlCanvasElement, String> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| "Web document is unavailable".to_owned())?;
        let canvas = document
            .create_element("canvas")
            .map_err(js_error)?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| "Scene slice is not a canvas".to_owned())?;
        set_canvas_size(&canvas, size)?;
        set_layer_style(&canvas, 1)?;
        canvas
            .style()
            .set_property("width", "100%")
            .map_err(js_error)?;
        canvas
            .style()
            .set_property("height", "100%")
            .map_err(js_error)?;
        canvas
            .style()
            .set_property("pointer-events", "none")
            .map_err(js_error)?;
        self.root.append_child(&canvas).map_err(js_error)?;
        self.slices.push(canvas.clone());
        Ok(canvas)
    }

    pub(crate) fn discard_last_slice(&mut self) {
        if let Some(canvas) = self.slices.pop() {
            canvas.remove();
        }
    }

    pub(crate) fn apply_composition(
        &mut self,
        placements: &[PlatformViewPlacement],
        overlays_after: &[usize],
        viewport: [u32; 2],
        scale: f64,
    ) -> Result<(), String> {
        let mut layer = 1_i32;
        let visible = placements
            .iter()
            .map(|placement| placement.id)
            .collect::<std::collections::HashSet<_>>();
        for (view_index, placement) in placements.iter().enumerate() {
            if let Some(entry) = self.entries.get(&placement.id) {
                let element = &entry.element;
                if placement.opacity <= 0.0 || placement.unsupported_shape_clip {
                    element
                        .style()
                        .set_property("visibility", "hidden")
                        .map_err(js_error)?;
                } else {
                    let bounds = placement.geometry.window_bounds();
                    let scale_css = if scale.is_finite() && scale > 0.0 {
                        scale
                    } else {
                        1.0
                    };
                    let style = element.style();
                    style
                        .set_property("visibility", "visible")
                        .map_err(js_error)?;
                    style
                        .set_property("left", &format!("{}px", bounds.x / scale_css as f32))
                        .map_err(js_error)?;
                    style
                        .set_property("top", &format!("{}px", bounds.y / scale_css as f32))
                        .map_err(js_error)?;
                    style
                        .set_property("width", &format!("{}px", bounds.width / scale_css as f32))
                        .map_err(js_error)?;
                    style
                        .set_property("height", &format!("{}px", bounds.height / scale_css as f32))
                        .map_err(js_error)?;
                    style
                        .set_property("opacity", &placement.opacity.to_string())
                        .map_err(js_error)?;
                    if let Some(clip) = placement.clip_bounds {
                        let left = (clip.x.0 as f32 - bounds.x).max(0.0) / scale_css as f32;
                        let top = (clip.y.0 as f32 - bounds.y).max(0.0) / scale_css as f32;
                        let right = (bounds.x + bounds.width
                            - clip.x.0.saturating_add(clip.width.0) as f32)
                            .max(0.0)
                            / scale_css as f32;
                        let bottom = (bounds.y + bounds.height
                            - clip.y.0.saturating_add(clip.height.0) as f32)
                            .max(0.0)
                            / scale_css as f32;
                        style
                            .set_property(
                                "clip-path",
                                &format!("inset({top}px {right}px {bottom}px {left}px)"),
                            )
                            .map_err(js_error)?;
                    } else {
                        style.set_property("clip-path", "none").map_err(js_error)?;
                    }
                    entry.factory.update(element);
                }
                element
                    .style()
                    .set_property("z-index", &layer.to_string())
                    .map_err(js_error)?;
            }
            layer += 1;
            if let Some(slice_index) = overlays_after.iter().position(|index| *index == view_index)
            {
                if let Some(canvas) = self.slices.get(slice_index) {
                    set_canvas_size(canvas, viewport)?;
                    canvas
                        .style()
                        .set_property("z-index", &layer.to_string())
                        .map_err(js_error)?;
                }
                layer += 1;
            }
        }
        for (&id, entry) in &self.entries {
            if !visible.contains(&id) {
                entry
                    .element
                    .style()
                    .set_property("visibility", "hidden")
                    .map_err(js_error)?;
            }
        }
        for (index, canvas) in self.slices.iter().enumerate() {
            set_canvas_size(canvas, viewport)?;
            canvas
                .style()
                .set_property(
                    "visibility",
                    if index < overlays_after.len() {
                        "visible"
                    } else {
                        "hidden"
                    },
                )
                .map_err(js_error)?;
        }
        Ok(())
    }
}

impl Drop for WebPlatformViews {
    fn drop(&mut self) {
        self.root.remove();
    }
}

fn set_layer_style(element: &HtmlElement, z_index: i32) -> Result<(), String> {
    let style = element.style();
    style
        .set_property("position", "absolute")
        .map_err(js_error)?;
    style.set_property("inset", "0").map_err(js_error)?;
    style
        .set_property("z-index", &z_index.to_string())
        .map_err(js_error)?;
    style.set_property("margin", "0").map_err(js_error)?;
    style
        .set_property("box-sizing", "border-box")
        .map_err(js_error)?;
    style
        .set_property("pointer-events", "auto")
        .map_err(js_error)?;
    Ok(())
}

fn set_canvas_size(canvas: &HtmlCanvasElement, size: [u32; 2]) -> Result<(), String> {
    let width = size[0].max(1);
    let height = size[1].max(1);
    if canvas.width() != width {
        canvas.set_width(width);
    }
    if canvas.height() != height {
        canvas.set_height(height);
    }
    Ok(())
}

fn js_error(error: wasm_bindgen::JsValue) -> String {
    format!("DOM operation failed: {error:?}")
}
