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

fn append_draw(builder: &mut RenderGraphBuilder, paint: u32, x: i32, sample: Option<SampleRegion>) {
    let mut fragment = RenderFragment::default();
    fragment.push_draw_command(Draw { paint, sample });
    builder.append_fragment(
        &fragment,
        DrawGeometry::from_position_size(
            PxPosition::new(Px(x), Px::ZERO),
            PxSize::new(Px(10), Px(10)),
        ),
        1.0,
    );
}

fn paint(op: &RenderGraphOp) -> u32 {
    op.command.downcast_draw_ref::<Draw>().unwrap().paint
}

#[test]
fn paint_updates_emit_the_current_frame_command() {
    let frame = |paint| {
        let mut builder = RenderGraphBuilder::new();
        append_draw(&mut builder, paint, 0, None);
        builder.finish()
    };
    let previous = frame(1);
    assert!(previous.ordered_ops(None).map(paint).eq([1]));
    assert!(frame(2).ordered_ops(Some(&previous)).map(paint).eq([2]));
}

#[test]
fn frame_updates_respect_sampling_dependencies_and_clip_scope() {
    let clip = PxRect::new(Px(20), Px(0), Px(10), Px(10));
    let frame = |source_x, sample_x, clip_reader| {
        let mut builder = RenderGraphBuilder::new();
        append_draw(&mut builder, 1, source_x, None);
        if clip_reader {
            builder.push_clip_push(clip);
        }
        let sample = SampleRegion::Absolute(PxRect::new(Px(sample_x), Px(0), Px(4), Px(4)));
        append_draw(&mut builder, 2, 20, Some(sample));
        if !clip_reader {
            builder.push_clip_push(clip);
        }
        builder.push_clip_pop();
        builder.finish()
    };
    let previous = frame(0, 40, false);
    let _ = previous.ordered_ops(None).count();

    for (source_x, sample_x) in [(40, 40), (0, 0)] {
        let current = frame(source_x, sample_x, false);
        assert!(
            current
                .ordered_ops(Some(&previous))
                .take(2)
                .map(paint)
                .eq([1, 2]),
            "the source at {source_x} must draw before sampling at {sample_x}"
        );
    }

    let current = frame(0, 40, true);
    let mut ops = current.ordered_ops(Some(&previous));
    assert_eq!(paint(ops.next().unwrap()), 1);
    assert!(matches!(ops.next().unwrap().command, Command::ClipPush(rect) if rect == clip));
    assert_eq!(paint(ops.next().unwrap()), 2);
    assert!(matches!(ops.next().unwrap().command, Command::ClipPop));
    assert!(ops.next().is_none());
}
