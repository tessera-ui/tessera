use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, Constraint, DimensionValue, Px, PxPosition, tessera, winit};

use crate::{
    animation,
    surface::{SurfaceArgsBuilder, surface},
};

const ANIM_TIME: Duration = Duration::from_millis(300);

#[derive(Builder)]
pub struct BottomSheetProviderArgs {
    pub on_close_request: Arc<dyn Fn() + Send + Sync>,
}

#[derive(Default)]
pub struct BottomSheetProviderState {
    is_open: bool,
    timer: Option<Instant>,
}

impl BottomSheetProviderState {
    pub fn open(&mut self) {
        if !self.is_open {
            self.is_open = true;
            let mut timer = Instant::now();
            if let Some(old_timer) = self.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            self.timer = Some(timer);
        }
    }

    pub fn close(&mut self) {
        if self.is_open {
            self.is_open = false;
            let mut timer = Instant::now();
            if let Some(old_timer) = self.timer {
                let elapsed = old_timer.elapsed();
                if elapsed < ANIM_TIME {
                    timer += ANIM_TIME - elapsed;
                }
            }
            self.timer = Some(timer);
        }
    }
}

#[tessera]
pub fn bottom_sheet_provider(
    args: BottomSheetProviderArgs,
    state: Arc<RwLock<BottomSheetProviderState>>,
    main_content: impl FnOnce() + Send + Sync + 'static,
    bottom_sheet_content: impl FnOnce(f32) + Send + Sync + 'static,
) {
    main_content();

    if state.read().is_open
        || state
            .read()
            .timer
            .is_some_and(|timer| timer.elapsed() < ANIM_TIME)
    {
        let on_close_for_keyboard = args.on_close_request.clone();

        let progress = animation::easing(state.read().timer.as_ref().map_or(1.0, |timer| {
            let elapsed = timer.elapsed();
            if elapsed >= ANIM_TIME {
                1.0
            } else {
                elapsed.as_secs_f32() / ANIM_TIME.as_secs_f32()
            }
        }));

        let scrim_alpha = if state.read().is_open {
            progress * 0.5
        } else {
            0.5 * (1.0 - progress)
        };

        surface(
            SurfaceArgsBuilder::default()
                .color(Color::BLACK.with_alpha(scrim_alpha))
                .on_click(Some(args.on_close_request))
                .width(DimensionValue::Fill {
                    min: None,
                    max: None,
                })
                .height(DimensionValue::Fill {
                    min: None,
                    max: None,
                })
                .block_input(true)
                .build()
                .unwrap(),
            None,
            || {},
        );

        state_handler(Box::new(move |input| {
            let events = input.keyboard_events.drain(..).collect::<Vec<_>>();
            for event in events {
                if event.state == winit::event::ElementState::Pressed {
                    if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =
                        event.physical_key
                    {
                        (on_close_for_keyboard)();
                    }
                }
            }
        }));

        let content_alpha = if state.read().is_open {
            progress
        } else {
            1.0 - progress
        };

        bottom_sheet_content(content_alpha);

        measure(Box::new(move |input| {
            let main_content_id = input.children_ids[0];
            let main_content_size =
                input.measure_child(main_content_id, input.parent_constraint)?;
            input.place_child(main_content_id, PxPosition::new(Px(0), Px(0)));

            if input.children_ids.len() > 1 {
                let scrim_id = input.children_ids[1];
                let _ = input.measure_child(scrim_id, input.parent_constraint)?;
                input.place_child(scrim_id, PxPosition::new(Px(0), Px(0)));
            }

            if input.children_ids.len() > 2 {
                let bottom_sheet_id = input.children_ids[2];

                let child_size = input.measure_child(
                    bottom_sheet_id,
                    &Constraint::new(
                        input.parent_constraint.width,
                        DimensionValue::Wrap {
                            min: None,
                            max: None,
                        },
                    ),
                )?;

                let parent_height = input.parent_constraint.height.get_max().unwrap_or(Px(0));

                let y = if state.read().is_open {
                    parent_height.0 as f32 - (child_size.height.0 as f32 * progress)
                } else {
                    parent_height.0 as f32 - (child_size.height.0 as f32 * (1.0 - progress))
                };

                input.place_child(bottom_sheet_id, PxPosition::new(Px(0), Px(y as i32)));
            }

            Ok(main_content_size)
        }));
    }
}
