//! Unified painter content for vector and raster imagery.
//!
//! ## Usage
//!
//! Pass one painter value into image-like and icon-like components.
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tessera_ui::AssetExt;
use thiserror::Error;

use crate::{
    image::{ImageData, ImageLoadError, TryIntoImageData},
    image_vector::{ImageVectorData, ImageVectorLoadError, TryIntoImageVectorData},
};

/// Shared visual content that can be rendered by image-like components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Painter {
    /// Vector content backed by decoded vector geometry.
    Vector(Arc<ImageVectorData>),
    /// Raster content backed by decoded image pixels.
    Raster(Arc<ImageData>),
}

impl From<ImageVectorData> for Painter {
    fn from(data: ImageVectorData) -> Self {
        Self::Vector(Arc::new(data))
    }
}

impl From<Arc<ImageVectorData>> for Painter {
    fn from(data: Arc<ImageVectorData>) -> Self {
        Self::Vector(data)
    }
}

impl From<ImageData> for Painter {
    fn from(data: ImageData) -> Self {
        Self::Raster(Arc::new(data))
    }
}

impl From<Arc<ImageData>> for Painter {
    fn from(data: Arc<ImageData>) -> Self {
        Self::Raster(data)
    }
}

impl From<crate::material_icons::Asset> for Painter {
    fn from(asset: crate::material_icons::Asset) -> Self {
        Self::Vector(asset.into())
    }
}

/// Errors that can occur while decoding a painter source.
#[derive(Debug, Error)]
pub enum PainterLoadError {
    /// Failed to read bytes from an asset handle.
    #[error("failed to read painter bytes from asset: {source}")]
    AssetRead {
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
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

/// Decodes a painter from an asset handle.
pub fn try_painter_asset<T>(asset: T) -> Result<Painter, PainterLoadError>
where
    T: AssetExt,
{
    let bytes = asset
        .read()
        .map_err(|source| PainterLoadError::AssetRead { source })?;
    try_decode_bytes(bytes.as_ref())
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

impl TryIntoPainter for Arc<ImageData> {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        Ok(Painter::from(self))
    }
}

impl TryIntoPainter for ImageVectorData {
    fn try_into_painter(self) -> Result<Painter, PainterLoadError> {
        Ok(Painter::from(self))
    }
}

impl TryIntoPainter for Arc<ImageVectorData> {
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
                Ok(data) => Ok(Painter::Raster(Arc::new(data))),
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
                Ok(data) => Ok(Painter::Raster(Arc::new(data))),
                Err(raster) => Err(PainterLoadError::Decode { vector, raster }),
            }
        }
    }
}
