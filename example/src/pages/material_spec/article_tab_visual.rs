use tessera_components::{modifier::ModifierExt as _, shape_def::Shape};
use tessera_ui::{Asset, Color, Dp, Modifier};

use crate::res::material_spec::tab_symbols as assets;

use super::srgb;

fn background_rgb(selected: bool, hovered: bool, pressed: bool) -> [u8; 3] {
    let (base, layer) = if selected {
        ([220, 218, 245], [31, 25, 35])
    } else {
        ([248, 241, 246], [77, 66, 86])
    };
    let opacity = if pressed {
        0.12
    } else if hovered {
        0.08
    } else {
        0.0
    };
    [0, 1, 2].map(|index| {
        (base[index] as f32 * (1.0 - opacity) + layer[index] as f32 * opacity).round() as u8
    })
}

pub(super) fn background(selected: bool, hovered: bool, pressed: bool) -> Color {
    let [r, g, b] = background_rgb(selected, hovered, pressed);
    srgb(r, g, b)
}

pub(super) fn pill_color(progress: f32, hovered: bool, pressed: bool) -> Color {
    let base = background_rgb(false, hovered, pressed);
    let selected = background_rgb(true, hovered, pressed);
    if progress <= 0.0 {
        return Color::TRANSPARENT;
    }
    let [r, g, b] = [0, 1, 2].map(|index| {
        (base[index] as f32 * (1.0 - progress) + selected[index] as f32 * progress).round() as u8
    });
    let desired = srgb(r, g, b);
    let background = background(false, hovered, pressed);
    let channel =
        |desired: f32, base: f32| ((desired - base * (1.0 - progress)) / progress).clamp(0.0, 1.0);
    Color::new(
        channel(desired.r, background.r),
        channel(desired.g, background.g),
        channel(desired.b, background.b),
        progress,
    )
}

#[derive(Clone, Copy)]
pub(super) struct SelectionMotion {
    from: f32,
    value: f32,
    target: f32,
    started: u64,
}

impl SelectionMotion {
    pub(super) fn new(target: f32, now: u64) -> Self {
        Self {
            from: target,
            value: target,
            target,
            started: now,
        }
    }

    pub(super) fn retarget(&mut self, target: f32, now: u64) {
        if target != self.target {
            self.from = self.value;
            self.target = target;
            self.started = now;
        }
    }

    pub(super) fn target(self) -> f32 {
        self.target
    }
    pub(super) fn value(self) -> f32 {
        self.value
    }
    pub(super) fn is_running(self) -> bool {
        self.value != self.target
    }

    pub(super) fn advance(&mut self, now: u64) {
        let progress = (now.saturating_sub(self.started) as f32 / 200_000_000.0).clamp(0.0, 1.0);
        if progress == 1.0 {
            self.value = self.target;
            return;
        }
        let mut low = 0.0;
        let mut high = 1.0;
        for _ in 0..24 {
            let t = (low + high) * 0.5;
            let x = 0.6 * t * (1.0 - t) * (1.0 - t) + t * t * t;
            if x < progress {
                low = t;
            } else {
                high = t;
            }
        }
        let t = (low + high) * 0.5;
        let eased = t * t * (3.0 - 2.0 * t);
        self.value = (self.from + (self.target - self.from) * eased).clamp(0.0, 1.0);
    }
}

pub(super) fn foreground(selected: bool, hovered: bool, pressed: bool) -> Color {
    if selected {
        srgb(33, 24, 43)
    } else if hovered || pressed {
        srgb(28, 27, 29)
    } else {
        srgb(77, 66, 86)
    }
}

pub(super) fn font_family(selected: bool, hovered: bool, pressed: bool) -> &'static str {
    if pressed {
        "Example Sans Tabs Pressed"
    } else if selected {
        "Example Sans Tabs Selected"
    } else if hovered {
        "Example Sans Tabs Hover"
    } else {
        "Example Sans Tabs"
    }
}

pub(super) fn asset(index: usize, selected: bool, hovered: bool, pressed: bool) -> Asset {
    let assets = [
        assets::INFO_0_400_SVG,
        assets::INFO_0_600_SVG,
        assets::INFO_0_300_SVG,
        assets::INFO_1_400_SVG,
        assets::INFO_1_600_SVG,
        assets::INFO_1_300_SVG,
        assets::STYLE_0_400_SVG,
        assets::STYLE_0_600_SVG,
        assets::STYLE_0_300_SVG,
        assets::STYLE_1_400_SVG,
        assets::STYLE_1_600_SVG,
        assets::STYLE_1_300_SVG,
        assets::DESIGN_SERVICES_0_400_SVG,
        assets::DESIGN_SERVICES_0_600_SVG,
        assets::DESIGN_SERVICES_0_300_SVG,
        assets::DESIGN_SERVICES_1_400_SVG,
        assets::DESIGN_SERVICES_1_600_SVG,
        assets::DESIGN_SERVICES_1_300_SVG,
        assets::ACCESSIBILITY_NEW_0_400_SVG,
        assets::ACCESSIBILITY_NEW_0_600_SVG,
        assets::ACCESSIBILITY_NEW_0_300_SVG,
        assets::ACCESSIBILITY_NEW_1_400_SVG,
        assets::ACCESSIBILITY_NEW_1_600_SVG,
        assets::ACCESSIBILITY_NEW_1_300_SVG,
    ];
    let state = if pressed {
        2
    } else if hovered {
        1
    } else {
        0
    };
    assets[index * 6 + usize::from(selected) * 3 + state]
}

pub(super) fn focus_modifier(modifier: Modifier, visible: bool, hovered: bool) -> Modifier {
    let modifier = if !visible {
        modifier
    } else if hovered {
        modifier.border_with_shape(Dp(2.0), Color::WHITE, Shape::CAPSULE)
    } else {
        modifier
            .border_with_shape(Dp(2.0), srgb(28, 27, 29), Shape::CAPSULE)
            .border_with_shape(Dp(4.0), Color::WHITE, Shape::CAPSULE)
    };
    modifier.padding_symmetric(Dp(24.0), Dp::ZERO)
}
