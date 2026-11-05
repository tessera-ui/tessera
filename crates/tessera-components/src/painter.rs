//! Unified painter content for vector and raster imagery.
//!
//! ## Usage
//!
//! Pass one painter value into image-like and icon-like components.
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tessera_ui::{Asset, LoadState, Resource};
use thiserror::Error;

use crate::{
    image::{ImageData, ImageLoadError, TryIntoImageData},
    image_vector::{ImageVectorData, ImageVectorLoadError, TryIntoImageVectorData},
};

/// Shared visual content that can be rendered by image-like components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Painter {
    /// Asset content loaded and decoded on demand.
    Asset(Resource<Painter>),
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

impl From<Asset> for Painter {
    fn from(asset: Asset) -> Self {
        Self::Asset(asset.load_with(decode_asset))
    }
}

impl Painter {
    pub(crate) fn resolve(self) -> Option<Self> {
        match self {
            Self::Asset(resource) => match resource.get() {
                LoadState::Ready(painter) => Some(painter),
                LoadState::Loading | LoadState::Failed(_) => None,
            },
            painter => Some(painter),
        }
    }
}

fn decode_asset(bytes: Arc<[u8]>) -> std::io::Result<Painter> {
    try_decode_bytes(&bytes).map_err(std::io::Error::other)
}

/// Errors that can occur while decoding a painter source.
#[derive(Debug, Error)]
pub enum PainterLoadError {
    /// Failed to decode bytes or a path as either vector or raster content.
    #[error("failed to decode painter as vector or raster content")]
    Decode {
        /// Error from the vector decode attempt.
        vector: ImageVectorLoadError,
        /// Error from the raster decode attempt.
        raster: ImageLoadError,
    },
}

/// Converts a source into unified painter content.
pub trait TryIntoPainter {
    /// Convert this source into painter content.
    fn try_into_painter(self) -> Result<Painter, PainterLoadError>;
}

/// Decodes a painter from any supported source.
pub fn try_painter<T>(source: T) -> Result<Painter, PainterLoadError>
where
    T: TryIntoPainter,
{
    source.try_into_painter()
}

impl TryIntoPainter for Painter {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        Ok(self)
    }
}

impl TryIntoPainter for ImageData {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        Ok(Painter::from(self))
    }
}

impl TryIntoPainter for ImageVectorData {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        Ok(Painter::from(self))
    }
}

impl TryIntoPainter for Vec<u8> {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        try_decode_bytes(self.as_slice())
    }
}

impl TryIntoPainter for &[u8] {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        try_decode_bytes(self)
    }
}

impl TryIntoPainter for String {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        try_decode_path(Path::new(&self))
    }
}

impl TryIntoPainter for &str {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        try_decode_path(Path::new(self))
    }
}

impl TryIntoPainter for PathBuf {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        try_decode_path(self.as_path())
    }
}

impl TryIntoPainter for &Path {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        try_decode_path(self)
    }
}

fn try_decode_bytes(bytes: &[u8]) -> Result<Painter, PainterLoadError> {
    let vector = bytes.try_into_image_vector_data();
    match vector {
        Ok(data) => Ok(Painter::Vector(data)),
        Err(vector) => {
            let raster = bytes.try_into_image_data();
            match raster {
                Ok(data) => Ok(Painter::Raster(data)),
                Err(raster) => Err(PainterLoadError::Decode { vector, raster }),
            }
        }
    }
}

fn try_decode_path(path: &Path) -> Result<Painter, PainterLoadError> {
    let vector = path.try_into_image_vector_data();
    match vector {
        Ok(data) => Ok(Painter::Vector(data)),
        Err(vector) => {
            let raster = path.try_into_image_data();
            match raster {
                Ok(data) => Ok(Painter::Raster(data)),
                Err(raster) => Err(PainterLoadError::Decode { vector, raster }),
            }
        }
    }
}
