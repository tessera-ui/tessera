//! Perceptual color transitions for Material controls.
//!
//! ## Usage
//!
//! Preserve color timing and velocity across changing visual states.

use tessera_ui::Color;

use super::material_motion::{Scalar, Spec};

fn color_vector(color: Color) -> [f64; 4] {
    let (r, g, b) = (f64::from(color.r), f64::from(color.g), f64::from(color.b));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        f64::from(color.a),
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

fn vector_color(vector: [f64; 4]) -> Color {
    let [alpha, lightness, a, b] = vector;
    let lightness = lightness.clamp(0.0, 1.0);
    let a = a.clamp(-0.5, 0.5);
    let b = b.clamp(-0.5, 0.5);
    let l = (lightness + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (lightness - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (lightness - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    Color::new(
        (4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s).clamp(0.0, 1.0) as f32,
        (-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s).clamp(0.0, 1.0) as f32,
        (-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s).clamp(0.0, 1.0) as f32,
        alpha.clamp(0.0, 1.0) as f32,
    )
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ColorMotion {
    channels: [Scalar; 4],
    target: Color,
}

impl ColorMotion {
    pub(crate) fn new(color: Color) -> Self {
        Self {
            channels: color_vector(color).map(Scalar::new),
            target: color,
        }
    }

    pub(crate) fn retarget(&mut self, color: Color, spec: Spec, frame: u64, snap: bool) {
        if snap {
            *self = Self::new(color);
        } else if self.target != color {
            for (channel, target) in self.channels.iter_mut().zip(color_vector(color)) {
                channel.start_to(target, spec, frame);
            }
            let duration = self
                .channels
                .iter()
                .map(|channel| channel.duration_millis)
                .max()
                .unwrap_or(0);
            for channel in &mut self.channels {
                channel.duration_millis = duration;
            }
            self.target = color;
        }
    }

    pub(crate) fn advance(&mut self, frame: u64) {
        for channel in &mut self.channels {
            channel.advance(frame);
        }
    }

    pub(crate) fn is_animating(&self) -> bool {
        self.channels.iter().any(Scalar::is_animating)
    }

    pub(crate) fn value(&self) -> Color {
        if self.is_animating() {
            vector_color(self.channels.map(|channel| channel.value))
        } else {
            self.target
        }
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::Color;

    use super::{color_vector, vector_color};

    #[test]
    fn oklab_conversions_match_independent_primary_and_neutral_values() {
        let red = color_vector(Color::RED);
        for (actual, expected) in red
            .into_iter()
            .zip([1.0, 0.62795536, 0.22486306, 0.12584630])
        {
            assert!((actual - expected).abs() < 0.000001);
        }
        let neutral = vector_color([0.25, 0.5, 0.0, 0.0]);
        for channel in [neutral.r, neutral.g, neutral.b] {
            assert!((channel - 0.125).abs() < 0.000001);
        }
        assert_eq!(neutral.a, 0.25);
    }
}
