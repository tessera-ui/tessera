use std::sync::LazyLock;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CubicPair {
    pub start: [[f32; 2]; 4],
    pub end: [[f32; 2]; 4],
}

#[derive(Clone, Copy)]
pub(crate) struct Sequence {
    pub offset: u32,
    pub count: u32,
    pub scale: f32,
    pub curvature: f32,
}

pub(crate) struct Geometry {
    pub curves: Vec<CubicPair>,
    pub sequences: [Sequence; 8],
}

static GEOMETRY: LazyLock<Geometry> = LazyLock::new(|| {
    let bytes = include_bytes!("../../../assets/material_loading_shapes.bin");
    assert_eq!(&bytes[..4], b"MLI1");
    let integer = |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
    assert_eq!(integer(4), 8);
    let curves: Vec<_> = bytes[104..]
        .chunks_exact(64)
        .map(|bytes| {
            let points: [f32; 16] = std::array::from_fn(|i| {
                f32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap())
            });
            CubicPair {
                start: std::array::from_fn(|i| [points[i * 2], points[i * 2 + 1]]),
                end: std::array::from_fn(|i| [points[8 + i * 2], points[9 + i * 2]]),
            }
        })
        .collect();
    let sequences = std::array::from_fn(|i| {
        let offset = integer(8 + i * 12);
        let count = integer(12 + i * 12);
        let scale = f32::from_bits(integer(16 + i * 12));
        let mut curvature = 0.0_f32;
        for curve in &curves[offset as usize..(offset + count) as usize] {
            for points in [curve.start, curve.end] {
                for i in 0..2 {
                    let x = points[i][0] - 2.0 * points[i + 1][0] + points[i + 2][0];
                    let y = points[i][1] - 2.0 * points[i + 1][1] + points[i + 2][1];
                    curvature = curvature.max(x.hypot(y));
                }
            }
        }
        Sequence {
            offset,
            count,
            scale,
            curvature,
        }
    });
    Geometry { curves, sequences }
});

pub(crate) fn geometry() -> &'static Geometry {
    &GEOMETRY
}

pub(crate) fn center(sequence: Sequence, progress: f32) -> [f32; 2] {
    let mut min = [f32::INFINITY; 2];
    let mut max = [f32::NEG_INFINITY; 2];
    for curve in
        &geometry().curves[sequence.offset as usize..(sequence.offset + sequence.count) as usize]
    {
        for i in 0..4 {
            for axis in 0..2 {
                let value =
                    curve.start[i][axis] + (curve.end[i][axis] - curve.start[i][axis]) * progress;
                min[axis] = min[axis].min(value);
                max[axis] = max[axis].max(value);
            }
        }
    }
    [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5]
}
