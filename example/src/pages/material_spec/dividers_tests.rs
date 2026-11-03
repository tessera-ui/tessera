//! Native divider article list figure regressions.
//!
//! ## Usage
//!
//! Verify the painted extent of full-width, inset, and open-space list figures.

use std::time::Duration;

use tessera_ui::{HeadlessConfig, HeadlessRenderer};

use super::*;

const LIST_WIDTH: usize = 300;

#[tessera]
fn list_fixture(rules: u8) {
    material_theme().child(move || {
        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
        boxed()
            .modifier(Modifier::new().fill_max_size().background(scheme.surface))
            .children(move || {
                boxed()
                    .modifier(Modifier::new().width(Dp(LIST_WIDTH as f64)))
                    .children(move || {
                        list_screen(INBOX, rules);
                    });
            });
    });
}

fn render(rules: u8) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || list_fixture(rules)),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 400,
            height: 400,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(6);
    renderer
}

struct Frame {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl Frame {
    fn read(renderer: &HeadlessRenderer) -> Self {
        let size = renderer.physical_size();
        Self {
            pixels: renderer.read_pixels().unwrap(),
            width: size.width as usize,
            height: size.height as usize,
        }
    }

    fn at(&self, x: usize, y: usize) -> [u8; 3] {
        let index = (y * self.width + x) * 4;
        [
            self.pixels[index],
            self.pixels[index + 1],
            self.pixels[index + 2],
        ]
    }

    /// Returns rows whose longest non-background run inside the list is at
    /// least half the list width, merged across antialiased neighbor rows.
    fn line_spans(&self) -> Vec<(usize, usize)> {
        let background = self.at(self.width - 1, self.height - 1);
        let painted = |pixel: [u8; 3]| {
            pixel
                .iter()
                .zip(background)
                .any(|(value, base)| value.abs_diff(base) > 6)
        };
        let mut spans: Vec<(usize, (usize, usize))> = Vec::new();
        for y in 0..self.height {
            let mut start = None;
            let mut longest = (0, 0);
            for x in 0..=LIST_WIDTH {
                let hit = x < LIST_WIDTH && painted(self.at(x, y));
                match (hit, start) {
                    (true, None) => start = Some(x),
                    (false, Some(first)) => {
                        if x - first > longest.1 - longest.0 {
                            longest = (first, x);
                        }
                        start = None;
                    }
                    _ => {}
                }
            }
            if longest.1 - longest.0 >= LIST_WIDTH / 2 {
                match spans.last_mut() {
                    Some((last, span)) if *last + 1 == y => {
                        *last = y;
                        span.0 = span.0.min(longest.0);
                        span.1 = span.1.max(longest.1);
                    }
                    _ => spans.push((y, longest)),
                }
            }
        }
        spans.into_iter().map(|(_, span)| span).collect()
    }

    fn painted_rows(&self) -> usize {
        let background = self.at(self.width - 1, self.height - 1);
        (0..self.height)
            .filter(|&y| (0..LIST_WIDTH).any(|x| self.at(x, y) != background))
            .count()
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn list_dividers_paint_full_width_or_both_insets_and_open_space_paints_none() {
    let inset = INSET_MARGIN as usize;
    for (rules, expected) in [
        (FULL, (0, LIST_WIDTH)),
        (INSET, (inset, LIST_WIDTH - inset)),
    ] {
        let spans = Frame::read(&render(rules)).line_spans();
        assert_eq!(
            spans.len(),
            3,
            "four rows must be separated by exactly three painted dividers: {spans:?}"
        );
        for span in spans {
            assert!(
                span.0.abs_diff(expected.0) <= 1 && span.1.abs_diff(expected.1) <= 1,
                "divider kind {rules} must span {expected:?}, painted {span:?}"
            );
        }
    }

    let open = Frame::read(&render(OPEN));
    assert!(
        open.line_spans().is_empty(),
        "an open-space list must not paint divider lines"
    );
    assert!(
        open.painted_rows() > 40,
        "the open-space list must still paint its rows"
    );
}
