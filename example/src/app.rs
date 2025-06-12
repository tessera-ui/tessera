use std::{
    sync::{
        Arc,
        atomic::{self, AtomicU32, AtomicU64},
    },
    time::Instant,
};

use parking_lot::RwLock;
use tessera::{CursorEventContent, DimensionValue};
use tessera_basic_components::{
    column::{ColumnItem, column},
    row::{RowItem, row},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
    text_editor::{TextEditorArgsBuilder, TextEditorState, text_editor},
};
use tessera_macros::tessera;

struct PerformanceMetrics {
    fps: AtomicU64,
    last_frame: RwLock<Instant>,
    last_fps_update_time: RwLock<Instant>,
    frames_since_last_update: AtomicU64,
}

pub struct AnimSpacerState {
    pub height: AtomicU32,
    pub max_height: AtomicU32,
    pub start_time: Instant,
}

struct AppData {
    click_count: AtomicU64,
    scroll_count: AtomicU64,
}

pub struct AppState {
    metrics: Arc<PerformanceMetrics>,
    anim_space_state: Arc<AnimSpacerState>,
    data: Arc<AppData>,
    editor_state: Arc<RwLock<TextEditorState>>,
    editor_state_2: Arc<RwLock<TextEditorState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(PerformanceMetrics {
                fps: AtomicU64::new(0),
                last_frame: RwLock::new(Instant::now()),
                last_fps_update_time: RwLock::new(Instant::now()),
                frames_since_last_update: AtomicU64::new(0),
            }),
            anim_space_state: Arc::new(AnimSpacerState {
                height: AtomicU32::new(0),
                max_height: AtomicU32::new(100),
                start_time: Instant::now(),
            }),
            data: Arc::new(AppData {
                click_count: AtomicU64::new(0),
                scroll_count: AtomicU64::new(0),
            }),
            editor_state: Arc::new(RwLock::new(TextEditorState::new(50.0.into(), 50.0.into()))),
            editor_state_2: Arc::new(RwLock::new(TextEditorState::new(50.0.into(), 50.0.into()))),
        }
    }
}

// Header row component with two text items
#[tessera]
fn header_row() {
    row([
        RowItem::fill(Box::new(|| text("Hello, this is tessera")), Some(1.0), None),
        RowItem::fill(
            Box::new(|| text("Hello, this is another tessera")),
            Some(1.0),
            None,
        ),
    ])
}

// Vertical text column component
#[tessera]
fn text_column() {
    column([
        ColumnItem::fill(Box::new(|| text("This is a column")), Some(1.0), None),
        ColumnItem::fill(Box::new(|| text("Another item in column")), Some(1.0), None),
    ])
}

// Content section with header and text column
#[tessera]
fn content_section() {
    surface(
        SurfaceArgsBuilder::default()
            .corner_radius(25.0)
            .padding(20.0.into())
            .color([0.8, 0.8, 0.9, 1.0]) // Light purple fill, RGBA
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        || {
            column([
                ColumnItem::wrap(Box::new(header_row)),
                ColumnItem::wrap(Box::new(text_column)),
            ]);
        },
    )
}

// Value display component
#[tessera]
fn value_display(app_data: Arc<AppData>) {
    surface(
        SurfaceArgsBuilder::default()
            .corner_radius(25.0)
            .color([0.9, 0.8, 0.8, 1.0]) // Light red fill, RGBA
            .build()
            .unwrap(),
        move || {
            text(
                app_data
                    .click_count
                    .load(atomic::Ordering::SeqCst)
                    .to_string(),
            );
        },
    )
}

// Scroll count display component
#[tessera]
fn scroll_display(app_data: Arc<AppData>) {
    surface(
        SurfaceArgsBuilder::default()
            .corner_radius(25.0)
            .color([0.8, 0.9, 0.8, 1.0]) // Light green fill, RGBA
            .build()
            .unwrap(),
        move || {
            text(format!(
                "Scrolls: {}",
                app_data.scroll_count.load(atomic::Ordering::SeqCst)
            ));
        },
    )
}

#[tessera]
fn perf_display(metrics: Arc<PerformanceMetrics>) {
    text(format!(
        "FPS: {}",
        metrics.fps.load(atomic::Ordering::SeqCst)
    ));
    state_handler(Box::new(move |_| {
        let now = Instant::now();
        let mut last_frame_guard = metrics.last_frame.write();
        *last_frame_guard = now;

        metrics
            .frames_since_last_update
            .fetch_add(1, atomic::Ordering::SeqCst);

        let mut last_fps_update_time_guard = metrics.last_fps_update_time.write();
        let elapsed_ms = now.duration_since(*last_fps_update_time_guard).as_millis();

        if elapsed_ms >= 100 {
            let frame_count = metrics
                .frames_since_last_update
                .swap(0, atomic::Ordering::SeqCst);
            let new_fps = (frame_count as f64 / (elapsed_ms as f64 / 1000.0)) as u64;
            metrics.fps.store(new_fps, atomic::Ordering::SeqCst);
            *last_fps_update_time_guard = now;
        }
    }));
}

fn ease_in_out_sine(x: f32) -> f32 {
    -(0.5 * (std::f32::consts::PI * x).cos()) + 0.5
}

#[tessera]
fn anim_spacer(state: Arc<AnimSpacerState>) {
    spacer(
        SpacerArgsBuilder::default()
            .height(DimensionValue::Fixed(
                state.height.load(atomic::Ordering::SeqCst),
            ))
            .build()
            .unwrap(),
    );

    state_handler(Box::new(move |_| {
        let now = Instant::now();
        let elapsed = now.duration_since(state.start_time).as_secs_f32();

        let max_height = state.max_height.load(atomic::Ordering::SeqCst) as f32;
        let speed = 200.0;
        let period = 2.0 * max_height / speed;
        let t = (elapsed % period) / period;
        let triangle = if t < 0.5 { 2.0 * t } else { 2.0 * (1.0 - t) };
        let eased = ease_in_out_sine(triangle);
        let new_height = (eased * max_height).round() as u32;
        state.height.store(new_height, atomic::Ordering::SeqCst);
    }));
}

#[tessera]
pub fn app(state: Arc<AppState>) {
    {
        let anim_space_state_clone = state.anim_space_state.clone();
        let app_data_clone_for_scroll_display = state.data.clone();
        let app_data_clone_for_value_display = state.data.clone();
        let metrics_clone = state.metrics.clone();
        let editor_state_clone = state.editor_state.clone();
        let editor_state_2_clone = state.editor_state_2.clone();
        surface(
            // Main background surface
            SurfaceArgsBuilder::default()
                .color([0.2, 0.2, 0.25, 1.0]) // Darker background, RGBA
                .width(DimensionValue::Fill {
                    min: None,
                    max: None,
                })
                .height(DimensionValue::Fill {
                    min: None,
                    max: None,
                })
                .build()
                .unwrap(),
            move || {
                column([
                    ColumnItem::wrap(Box::new(content_section)),
                    ColumnItem::wrap(Box::new(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(DimensionValue::Fixed(10))
                                .width(DimensionValue::Fill {
                                    min: None,
                                    max: None,
                                })
                                .build()
                                .unwrap(),
                        )
                    })),
                    // --- New Outline Surface Example ---
                    ColumnItem::wrap(Box::new(|| {
                        surface(
                            SurfaceArgsBuilder::default()
                                .color([0.3, 0.3, 0.3, 0.5]) // Semi-transparent fill color
                                .border_width(5.0)
                                .border_color(Some([1.0, 0.0, 0.0, 1.0])) // Red border, RGBA
                                .corner_radius(15.0)
                                .width(DimensionValue::Fixed(200))
                                .height(DimensionValue::Fixed(100))
                                .padding(10.0.into())
                                .shadow(Some(tessera::ShadowProps {
                                    color: [0.0, 0.0, 0.0, 0.5],
                                    offset: [3.0, 3.0],
                                    smoothness: 5.0,
                                }))
                                .build()
                                .unwrap(),
                            || {
                                text("Outlined Box");
                            },
                        )
                    })),
                    // --- Transparent Surface Example ---
                    ColumnItem::wrap(Box::new(|| {
                        surface(
                            SurfaceArgsBuilder::default()
                                .color([0.0, 0.0, 1.0, 0.3]) // Transparent blue fill
                                .corner_radius(10.0)
                                .width(DimensionValue::Fixed(150))
                                .height(DimensionValue::Fixed(70))
                                .padding(5.0.into())
                                .build()
                                .unwrap(),
                            || {
                                text("Transparent Fill");
                            },
                        )
                    })),
                    ColumnItem::wrap(Box::new(move || {
                        text_editor(
                            TextEditorArgsBuilder::default()
                                .height(Some(DimensionValue::Fixed(120))) // Fixed height to test scrolling
                                .width(Some(DimensionValue::Fill {
                                    min: None,
                                    max: None,
                                }))
                                .selection_color(Some([0.3, 0.8, 0.4, 0.5])) // Custom green selection with 50% transparency
                                .build()
                                .unwrap(),
                            editor_state_clone.clone(),
                        );
                    })),
                    ColumnItem::wrap(Box::new(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(DimensionValue::Fixed(10))
                                .width(DimensionValue::Fill {
                                    min: None,
                                    max: None,
                                })
                                .build()
                                .unwrap(),
                        )
                    })),
                    // Second text editor with default selection color
                    ColumnItem::wrap(Box::new(move || {
                        text_editor(
                            TextEditorArgsBuilder::default()
                                .height(Some(DimensionValue::Fixed(100))) // Fixed height to test scrolling
                                .width(Some(DimensionValue::Fill {
                                    min: None,
                                    max: None,
                                }))
                                .build()
                                .unwrap(),
                            editor_state_2_clone.clone(),
                        );
                    })),
                    ColumnItem::wrap(Box::new(move || {
                        anim_spacer(anim_space_state_clone.clone())
                    })),
                    ColumnItem::wrap(Box::new(move || {
                        value_display(app_data_clone_for_value_display.clone())
                    })),
                    ColumnItem::wrap(Box::new(move || {
                        scroll_display(app_data_clone_for_scroll_display.clone())
                    })),
                    ColumnItem::wrap(Box::new(move || perf_display(metrics_clone.clone()))),
                ]);
            },
        );
    }

    {
        let app_data_clone_for_handler = state.data.clone();
        state_handler(Box::new(move |input| {
            let count = input
                .cursor_events
                .iter()
                .filter(|event| match &event.content {
                    CursorEventContent::Pressed(key) => {
                        matches!(key, tessera::PressKeyEventType::Left)
                    }
                    _ => false,
                })
                .count();
            if count > 0 {
                println!("Left mouse button pressed {count} times");
                app_data_clone_for_handler
                    .click_count
                    .fetch_add(count as u64, atomic::Ordering::SeqCst);
            }
        }));
    }

    {
        let app_data_clone_for_scroll = state.data.clone();
        state_handler(Box::new(move |input| {
            let scroll_count = input
                .cursor_events
                .iter()
                .filter(|event| match &event.content {
                    CursorEventContent::Scroll(_) => true,
                    _ => false,
                })
                .count();
            if scroll_count > 0 {
                let total_delta: f32 = input
                    .cursor_events
                    .iter()
                    .filter_map(|event| match &event.content {
                        CursorEventContent::Scroll(scroll_event) => {
                            Some(scroll_event.delta_y.abs())
                        }
                        _ => None,
                    })
                    .sum();
                println!("Scrolled {scroll_count} times with total delta: {total_delta}");
                app_data_clone_for_scroll
                    .scroll_count
                    .fetch_add(scroll_count as u64, atomic::Ordering::SeqCst);
            }
        }));
    }
}
