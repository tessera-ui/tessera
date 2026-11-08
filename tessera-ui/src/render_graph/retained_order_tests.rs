use super::*;

#[derive(Clone)]
struct Draw {
    paint: u32,
    sample: Option<SampleRegion>,
}

impl DrawCommand for Draw {
    fn sample_region(&self) -> Option<SampleRegion> {
        self.sample
    }

    fn apply_opacity(&mut self, _: f32) {}
}

fn graph(paint: u32, x: i32, sample: Option<SampleRegion>) -> RenderGraph {
    let mut fragment = RenderFragment::default();
    fragment.push_draw_command(Draw { paint, sample });
    let mut builder = RenderGraphBuilder::new();
    builder.append_fragment(
        &fragment,
        DrawGeometry::from_position_size(
            PxPosition::new(Px(x), Px::ZERO),
            PxSize::new(Px(10), Px(10)),
        ),
        1.0,
    );
    builder.finish()
}

#[test]
fn paint_changes_reuse_order_but_supply_current_commands() {
    let previous = graph(1, 0, None);
    let current = graph(2, 0, None);
    let plan = previous.order_plan(None);
    let command = &current.ordered_ops(Some(&previous)).next().unwrap().command;
    assert_eq!(command.downcast_draw_ref::<Draw>().unwrap().paint, 2);
    assert!(Arc::ptr_eq(plan, current.order_plan(None)));
}

#[test]
fn changed_read_write_bounds_and_clip_transitions_rebuild_order() {
    let sample = |x| {
        Some(SampleRegion::Absolute(PxRect::new(
            Px(x),
            Px(0),
            Px(4),
            Px(4),
        )))
    };
    let previous = graph(1, 0, sample(10));
    let plan = previous.order_plan(None);
    for current in [graph(1, 2, sample(10)), graph(1, 0, sample(20))] {
        assert!(!Arc::ptr_eq(plan, current.order_plan(Some(&previous))));
    }

    let clip = PxRect::new(Px(0), Px(0), Px(10), Px(10));
    let mut nested = RenderGraphBuilder::new();
    nested.push_clip_push(clip);
    nested.push_clip_push(clip);
    nested.push_clip_pop();
    nested.push_clip_pop();
    let nested = nested.finish();
    let mut siblings = RenderGraphBuilder::new();
    for _ in 0..2 {
        siblings.push_clip_push(clip);
        siblings.push_clip_pop();
    }
    let siblings = siblings.finish();
    assert!(!Arc::ptr_eq(
        nested.order_plan(None),
        siblings.order_plan(Some(&nested)),
    ));
}
