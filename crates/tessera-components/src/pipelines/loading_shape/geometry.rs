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

impl Default for Geometry {
    fn default() -> Self {
        let (words, _) = include_bytes!("../../../assets/material_loading_shapes.bin").as_chunks();
        assert_eq!(&words[0], b"MLI1");
        assert_eq!(u32::from_le_bytes(words[1]), 8);
        let (sequences, curves) = words[2..].split_at(8 * 3);
        let curves: Vec<_> = curves
            .as_chunks::<16>()
            .0
            .iter()
            .map(|words| {
                let points = words.map(f32::from_le_bytes);
                CubicPair {
                    start: std::array::from_fn(|i| [points[i * 2], points[i * 2 + 1]]),
                    end: std::array::from_fn(|i| [points[8 + i * 2], points[9 + i * 2]]),
                }
            })
            .collect();
        let sequences = std::array::from_fn(|i| {
            let [offset, count, scale] = sequences.as_chunks::<3>().0[i].map(u32::from_le_bytes);
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
                scale: f32::from_bits(scale),
                curvature,
            }
        });
        Geometry { curves, sequences }
    }
}

pub(crate) fn geometry() -> std::rc::Rc<Geometry> {
    tessera_ui::runtime_resource::<Geometry>()
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
