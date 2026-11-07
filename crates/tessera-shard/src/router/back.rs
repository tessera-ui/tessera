use std::sync::Arc;

use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Dp, FocusManager, FrameNanosControl, Modifier,
    PointerInput, PointerInputModifierNode, PxPosition, SemanticsModifierNode, State,
    current_frame_nanos, key,
    layout::layout,
    modifier::{ModifierCapabilityExt as _, PlacementModifierNode},
    provide_context, receive_frame_nanos, remember,
    system_back::{BackPreviewScope, BackSwipeEdge, SystemBackEvent, system_back_handler},
    tessera, use_context,
};

use crate::{__private::RouteRenderScope, RouterController, RouterDestination};

#[derive(Clone, Copy)]
struct Preview {
    route_id: u64,
    edge: BackSwipeEdge,
    progress: f32,
    restoring: Option<(u64, f32)>,
}

#[tessera(tessera_ui)]
pub(super) fn route_display(controller: State<RouterController>) {
    let (route_id, route) = controller
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
                preview.set(current.map(|(route_id, _)| Preview {
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
                        preview.restoring = Some((current_frame_nanos(), preview.progress));
                    }
                });
            }
            SystemBackEvent::Committed => {
                let started = preview.get().filter(|preview| preview.restoring.is_none());
                preview.set(None);
                if let Some((id, _)) = current
                    && started.is_none_or(|started| started.route_id == id)
                {
                    controller.with_mut(RouterController::pop);
                    FocusManager::current().defer_keyboard_until_layout();
                }
            }
        }
    });

    let gesture = preview.get().filter(|preview| preview.route_id == route_id);
    if gesture.is_some_and(|gesture| gesture.restoring.is_some()) {
        receive_frame_nanos(move |now| {
            preview.with_mut(|preview| {
                if let Some(active) = preview
                    && let Some((start, progress)) = active.restoring
                {
                    let remaining = 1.0 - now.saturating_sub(start) as f32 / 160_000_000.0;
                    if remaining > 0.0 {
                        active.progress = progress * remaining;
                        return FrameNanosControl::Continue;
                    }
                }
                *preview = None;
                FrameNanosControl::Stop
            })
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
    layout().child(move || {
        if let Some((id, previous)) = &previous {
            key(*id, || {
                route_layer(controller, *id, previous.clone())
                    .preview(true)
                    .modifier(
                        Modifier::new()
                            .push_semantics(HiddenRoute)
                            .push_pointer_preview_input(HiddenRoute),
                    );
            });
        }
        key(route_id, || {
            route_layer(controller, route_id, route.clone()).modifier(foreground.clone());
        });
    });
}

#[tessera(tessera_ui)]
fn route_layer(
    controller: State<RouterController>,
    route_id: u64,
    #[prop(skip_eq)] route: Arc<dyn RouterDestination>,
    modifier: Option<Modifier>,
    preview: Option<bool>,
) {
    let destination = route.exec_component_handle();
    let scope = RouteRenderScope {
        controller,
        route_id,
    };
    let preview = preview.unwrap_or(false)
        || use_context::<BackPreviewScope>().is_some_and(|scope| scope.get().0);
    layout()
        .modifier(modifier.unwrap_or_default())
        .child(move || {
            provide_context(
                || scope,
                || {
                    provide_context(|| BackPreviewScope(preview), || destination(controller));
                },
            );
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
    use tessera_ui::testing::{
        TestProbe, layout_test, pump_layout_test_session, start_layout_test_session,
    };

    use super::*;
    use crate::{shard, shard_home};

    #[shard(state = usize, shard_crate_path = crate)]
    fn preview_page(label: Option<u8>, observed: Option<TestProbe<Vec<(u8, usize)>>>) {
        if let (Some(label), Some(observed)) = (label, observed) {
            observed.with_mut(|pages| pages.push((label, state.get())));
        }
        layout().child(|| {});
    }

    #[tessera(tessera_ui)]
    fn fixture(
        observed: TestProbe<Vec<(u8, usize)>>,
        stack: TestProbe<Option<State<RouterController>>>,
    ) {
        let router = remember(|| {
            let mut router = RouterController::with_root(PreviewPageDestination {
                label: Some(1),
                observed: Some(observed.clone()),
            });
            router.init_or_get::<usize, _, _>(PreviewPageDestination::destination_id(), |state| {
                state.set(10);
            });
            router.push(PreviewPageDestination {
                label: Some(2),
                observed: Some(observed.clone()),
            });
            router.init_or_get::<usize, _, _>(PreviewPageDestination::destination_id(), |state| {
                state.set(20);
            });
            router
        });
        stack.set(Some(router));
        shard_home().controller(router);
    }

    #[test]
    fn predictive_back_previews_previous_route_state_and_only_pops_on_commit() {
        let observed = TestProbe::default();
        let stack = TestProbe::default();
        let probe = observed.clone();
        let controller = stack.clone();
        let mut session = start_layout_test_session(layout_test(move || {
            fixture(probe.clone(), controller.clone());
        }));
        pump_layout_test_session(&mut session, 0);
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
        pump_layout_test_session(&mut session, 16_000_000);
        assert!(observed.take().contains(&(1, 10)));
        assert_eq!(session.run(|| controller.with(RouterController::len)), 2);
        session.dispatch_system_back(SystemBackEvent::Cancelled);
        pump_layout_test_session(&mut session, 32_000_000);
        pump_layout_test_session(&mut session, 224_000_000);
        assert_eq!(session.run(|| controller.with(RouterController::len)), 2);

        session.dispatch_system_back(SystemBackEvent::Committed);
        pump_layout_test_session(&mut session, 240_000_000);
        assert_eq!(session.run(|| controller.with(RouterController::len)), 1);
        session.dispatch_system_back(SystemBackEvent::Committed);
        assert_eq!(session.run(|| controller.with(RouterController::len)), 1);
    }
}
