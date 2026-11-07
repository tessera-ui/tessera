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
    Source(Resource<Painter>),
    /// Vector content backed by decoded vector geometry.
    Vector(ImageVectorData),
    /// Raster content backed by decoded image pixels.
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
                Self::Source(Resource::load(source, decode))
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
            Self::Source(resource) => match resource.get() {
                LoadState::Ready(painter) => Some(painter),
                LoadState::Loading | LoadState::Failed(_) => None,
            },
            painter => Some(painter),
        }
    }
}

fn decode(bytes: Arc<[u8]>) -> io::Result<Painter> {
    decode_vector(&bytes)
        .map(Painter::Vector)
        .or_else(|vector| {
            decode_raster(&bytes)
                .map(Painter::Raster)
                .map_err(|raster| {
                    io::Error::other(format!("not SVG ({vector}) or raster ({raster})"))
                })
        })
}
