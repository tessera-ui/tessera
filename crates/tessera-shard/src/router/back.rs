use std::rc::Rc;

use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Dp, ModelStore, Modifier, PointerInput,
    PointerInputModifierNode, PxPosition, SemanticsModifierNode, State, focus, key, launch_once,
    layout::{Stack, layout},
    modifier::{ModifierCapabilityExt as _, PlacementModifierNode},
    provide_context, remember,
    system_back::{BackPreviewScope, BackSwipeEdge, SystemBackEvent, system_back_handler},
    tessera,
    time::Instant,
    use_context, with_frame_time,
};

use crate::{RouterController, RouterDestination};

#[derive(Clone, Copy)]
struct Preview {
    route_id: u64,
    edge: BackSwipeEdge,
    progress: f32,
    // The restoration starts on the first frame that observes it.
    restoring: Option<(Option<Instant>, f32)>,
}

#[tessera]
pub(super) fn route_display(controller: State<RouterController>) {
    let (route_id, route, models) = controller
        .with(|router| router.route_at_depth(0))
        .expect("Router stack should not be empty");
    let can_pop = controller.with(|router| router.len() > 1);
    let preview = remember(|| None::<Preview>);
    system_back_handler(can_pop).on_back(move |event| {
        let current = controller.with(|router| {
            (router.len() > 1)
                .then(|| router.route_at_depth(0))
                .flatten()
        });
        match event {
            SystemBackEvent::Started(edge) => {
                preview.set(current.map(|(route_id, _, _)| Preview {
                    route_id,
                    edge,
                    progress: 0.0,
                    restoring: None,
                }));
            }
            SystemBackEvent::Progress { progress, .. } => {
                preview.with_mut(|preview| {
                    if let Some(preview) = preview {
                        preview.progress = progress;
                    }
                });
            }
            SystemBackEvent::Cancelled => {
                preview.with_mut(|preview| {
                    if let Some(preview) = preview {
                        preview.restoring = Some((None, preview.progress));
                    }
                });
            }
            SystemBackEvent::Committed => {
                let started = preview.get().filter(|preview| preview.restoring.is_none());
                preview.set(None);
                if let Some((id, _, _)) = current
                    && started.is_none_or(|started| started.route_id == id)
                {
                    controller.with_mut(RouterController::pop);
                    focus::defer_keyboard_until_layout();
                }
            }
        }
    });

    let gesture = preview.get().filter(|preview| preview.route_id == route_id);
    if gesture.is_some_and(|gesture| gesture.restoring.is_some()) {
        launch_once(async move {
            loop {
                let restoring = with_frame_time(|now| {
                    preview.with_mut(|preview| {
                        if let Some(active) = preview
                            && let Some((start, progress)) = &mut active.restoring
                        {
                            let start = *start.get_or_insert(now);
                            let progress = *progress;
                            let remaining =
                                1.0 - now.saturating_duration_since(start).as_secs_f32() / 0.16;
                            if remaining > 0.0 {
                                active.progress = progress * remaining;
                                return true;
                            }
                        }
                        *preview = None;
                        false
                    })
                })
                .await;
                if !restoring {
                    break;
                }
            }
        });
    }
    let gesture = gesture.filter(|gesture| gesture.progress > 0.0);
    let previous = gesture.and_then(|_| controller.with(|router| router.route_at_depth(1)));
    let mut foreground = Modifier::new();
    if let Some(gesture) = gesture {
        let direction = match gesture.edge {
            BackSwipeEdge::Left => 1.0,
            BackSwipeEdge::Right => -1.0,
        };
        let offset = Dp(72.0 * f64::from(gesture.progress) * direction);
        foreground = foreground.push_placement(RouteOffset(offset));
    }
    layout(Stack).child(move || {
        if let Some((id, previous, previous_models)) = &previous {
            key(*id, || {
                route_layer(previous_models.clone(), previous.clone())
                    .preview(true)
                    .modifier(
                        Modifier::new()
                            .push_semantics(HiddenRoute)
                            .push_pointer_preview_input(HiddenRoute),
                    );
            });
        }
        key(route_id, || {
            route_layer(models.clone(), route.clone()).modifier(foreground.clone());
        });
    });
}

#[tessera]
fn route_layer(
    models: Rc<ModelStore>,
    #[prop(skip_eq)] route: Rc<dyn RouterDestination>,
    modifier: Option<Modifier>,
    preview: Option<bool>,
) {
    let preview = preview.unwrap_or(false)
        || use_context::<BackPreviewScope>().is_some_and(|scope| scope.get().0);
    layout(Stack)
        .modifier(modifier.unwrap_or_default())
        .child(move || {
            provide_context(models.scope(), || {
                provide_context(BackPreviewScope(preview), || route.render());
            });
        });
}

struct HiddenRoute;

impl SemanticsModifierNode for HiddenRoute {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        _handler: &mut Option<AccessibilityActionHandler>,
    ) {
        node.hidden = true;
        node.merge_descendants = false;
    }
}

impl PointerInputModifierNode for HiddenRoute {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        input.block_cursor();
    }
}

#[derive(Clone, Copy, PartialEq)]
struct RouteOffset(Dp);

impl PlacementModifierNode for RouteOffset {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        PxPosition::from_f32_xy(position.x + self.0.to_pixels_f64() as f32, position.y)
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::testing::{TestProbe, UiTest};

    use super::*;
    use crate::{shard, shard_home};

    struct PreviewModel {
        value: tessera_ui::State<usize>,
    }

    impl tessera_ui::ModelInit for PreviewModel {
        fn init() -> Self {
            Self {
                value: tessera_ui::State::new(0),
            }
        }
    }

    #[shard]
    fn preview_page(
        model: tessera_ui::Model<PreviewModel>,
        label: Option<u8>,
        observed: Option<TestProbe<Vec<(u8, usize)>>>,
    ) {
        if let (Some(label), Some(observed)) = (label, observed) {
            observed.with_mut(|pages| pages.push((label, model.with(|model| model.value.get()))));
        }
        layout(Stack).child(|| {});
    }

    #[tessera]
    fn fixture(
        observed: TestProbe<Vec<(u8, usize)>>,
        stack: TestProbe<Option<State<RouterController>>>,
    ) {
        let router = remember(|| {
            let mut router = RouterController::with_root(PreviewPageDestination {
                label: Some(1),
                observed: Some(observed.clone()),
            });
            router.push(PreviewPageDestination {
                label: Some(2),
                observed: Some(observed.clone()),
            });
            for (route, value) in [(0, 10), (1, 20)] {
                router
                    .route_models(route)
                    .expect("route exists")
                    .model(<PreviewModel as tessera_ui::ModelInit>::init)
                    .with(|model| model.value.set(value));
            }
            router
        });
        stack.set(Some(router));
        shard_home().controller(router);
    }

    #[test]
    fn predictive_back_previews_previous_route_state_and_only_pops_on_commit() {
        let observed = TestProbe::default();
        let stack = TestProbe::default();
        let controller = stack.clone();
        let mut session = UiTest::new().content(|| fixture(observed.clone(), controller.clone()));
        session.frame();
        let controller = stack.get().expect("mounted controller");
        assert_eq!(observed.take(), vec![(2, 20)]);

        for event in [
            SystemBackEvent::Started(BackSwipeEdge::Left),
            SystemBackEvent::Progress {
                progress: 0.5,
                edge: BackSwipeEdge::Left,
            },
        ] {
            session.dispatch_system_back(event);
        }
        session.advance(std::time::Duration::from_millis(16));
        assert!(observed.take().contains(&(1, 10)));
        assert_eq!(session.run(|| controller.with(RouterController::len)), 2);
        session.dispatch_system_back(SystemBackEvent::Cancelled);
        session.advance(std::time::Duration::from_millis(16));
        session.advance(std::time::Duration::from_millis(192));
        assert_eq!(session.run(|| controller.with(RouterController::len)), 2);

        session.dispatch_system_back(SystemBackEvent::Committed);
        session.advance(std::time::Duration::from_millis(16));
        assert_eq!(session.run(|| controller.with(RouterController::len)), 1);
        session.dispatch_system_back(SystemBackEvent::Committed);
        assert_eq!(session.run(|| controller.with(RouterController::len)), 1);
    }
}
