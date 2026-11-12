//! Unified painter content for vector and raster imagery.
//!
//! ## Usage
//!
//! Pass one painter value into image-like and icon-like components.
use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use tessera_ui::{Asset, LoadState, Resource, Source};

use crate::{
    image::{ImageData, decode_raster},
    image_vector::{ImageVectorData, decode_vector},
};

/// Shared visual content that can be rendered by image-like components.
///
/// Encoded SVG or raster sources are decoded off the UI thread and show
/// nothing until they are ready.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Painter {
    /// Content decoded from a source.
    Source(PainterSource),
    /// Vector content backed by decoded vector geometry.
    Vector(ImageVectorData),
    /// Raster content backed by decoded image pixels.
    Raster(ImageData),
}

/// A painter source that is decoded on demand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PainterSource(Resource<Decoded>);

#[derive(Clone)]
enum Decoded {
    Vector(ImageVectorData),
    Raster(ImageData),
}

impl From<ImageVectorData> for Painter {
    fn from(data: ImageVectorData) -> Self {
        Self::Vector(data)
    }
}

impl From<ImageData> for Painter {
    fn from(data: ImageData) -> Self {
        Self::Raster(data)
    }
}

macro_rules! painter_from_source {
    ($($ty:ty),* $(,)?) => {$(
        impl From<$ty> for Painter {
            fn from(source: $ty) -> Self {
                Self::Source(PainterSource(Resource::load(source, decode)))
            }
        }
    )*};
}

painter_from_source!(
    Source,
    Asset,
    &str,
    String,
    &Path,
    PathBuf,
    &[u8],
    Vec<u8>,
    Arc<[u8]>
);

impl Painter {
    pub(crate) fn resolve(self) -> Option<Self> {
        match self {
            Self::Source(PainterSource(resource)) => match resource.get() {
                LoadState::Ready(Decoded::Vector(data)) => Some(Self::Vector(data)),
                LoadState::Ready(Decoded::Raster(data)) => Some(Self::Raster(data)),
                LoadState::Loading | LoadState::Failed(_) => None,
            },
            painter => Some(painter),
        }
    }
}

fn decode(bytes: Arc<[u8]>) -> io::Result<Decoded> {
    decode_vector(&bytes)
        .map(Decoded::Vector)
        .or_else(|vector| {
            decode_raster(&bytes)
                .map(Decoded::Raster)
                .map_err(|raster| {
                    io::Error::other(format!("not SVG ({vector}) or raster ({raster})"))
                })
        })
}
