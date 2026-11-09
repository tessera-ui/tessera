use crate::{
    AccessibilityActionHandler, AccessibilityNode, ComputedData, Constraint, LayoutModifierNode,
    LayoutResult, MeasurementError, Modifier, PointerEventPass, PointerInput,
    PointerInputModifierNode, SemanticsModifierNode,
    layout::{MeasureScope, layout},
    scaled_box,
    testing::TestProbe,
};

use super::*;

#[derive(Clone, PartialEq)]
struct Anchor;
impl LayoutModifierNode for Anchor {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let child = scope.children()[0].measure(scope, &Constraint::exact(Px(150), Px(35)))?;
        let mut result = LayoutResult::new(ComputedData {
            width: Px(300),
            height: Px(200),
        });
        result.place_child(child, PxPosition::from_f32_xy(10.25, 20.5));
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct Content;
impl LayoutModifierNode for Content {
    fn measure(&self, _: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData {
            width: Px(240),
            height: Px(56),
        }))
    }
}

struct Pointer(TestProbe<Option<PxPosition>>);
impl PointerInputModifierNode for Pointer {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass == PointerEventPass::Main
            && input
                .pointer_changes
                .iter()
                .any(|change| matches!(change.content, CursorEventContent::Pressed(_)))
        {
            self.0.set(input.cursor_position_rel);
        }
    }
}

struct Tag;
impl SemanticsModifierNode for Tag {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.key = Some("protocol-probe".to_owned());
        node.role = Some(crate::accesskit::Role::Button);
    }
}

#[crate::tessera]
fn protocol_fixture(probe: TestProbe<Option<PxPosition>>) {
    layout().policy(Anchor).child(move || {
        let probe = probe.clone();
        scaled_box(0.625).child(move || {
            layout().policy(Content).modifier(
                Modifier::new()
                    .push_semantics(Tag)
                    .push_pointer_input(Pointer(probe.clone())),
            );
        });
    });
}

fn probe_bounds(value: &Value) -> Option<&Value> {
    if value
        .get("accessibility")
        .and_then(|value| value.get("key"))
        .and_then(Value::as_str)
        == Some("protocol-probe")
    {
        return value.get("bounds");
    }
    value
        .get("children")?
        .as_array()?
        .iter()
        .find_map(probe_bounds)
}

#[test]
#[ignore = "requires a native GPU adapter; run this exact test in its own process"]
fn gpu_worker_protocol_preserves_fractional_input_and_scaled_snapshot_bounds() {
    let probe = TestProbe::new(None);
    let mut renderer = HeadlessRenderer::new(
        Box::new({
            let probe = probe.clone();
            move || protocol_fixture(probe.clone())
        }),
        Vec::new(),
        HeadlessConfig {
            width: 300,
            height: 200,
            ..HeadlessConfig::default()
        },
    )
    .expect("Native GPU adapter");
    renderer.render_frames(2);
    let (_, response) = handle_request(&mut renderer, WorkerRequest::Snapshot { id: Some(1) })
        .expect("Snapshot request");
    let bounds = probe_bounds(&response["snapshot"]["root"]).expect("Published probe bounds");
    assert_eq!(bounds["x"].as_f64(), Some(10.25));
    assert_eq!(bounds["y"].as_f64(), Some(20.5));
    assert_eq!(bounds["width"].as_f64(), Some(150.0));
    assert_eq!(bounds["width"].as_i64(), Some(150));
    assert_eq!(bounds["height"].as_f64(), Some(35.0));
    let request: WorkerRequest =
        serde_json::from_value(json!({ "cmd": "input", "kind": "press", "x": 40.25, "y": 35.5 }))
            .expect("Fractional JSON input");
    let (_, response) = handle_request(&mut renderer, request).expect("Input request");
    assert_eq!(response["ok"], true);
    renderer.render_frames(1);
    assert_eq!(probe.get(), Some(PxPosition::from_f32_xy(48.0, 24.0)));
}

#[cfg(all(feature = "profiling", not(target_family = "wasm")))]
#[path = "headless_summary_tests.rs"]
mod summary_tests;
