//! Material Design icon assets and decode helpers.
//!
//! ## Usage
//!
//! Use style modules (for example [`filled`]) to obtain icon assets, then pass
//! them to `icon().try_vector_asset(...)` or decode with
//! [`TryIntoImageVectorData`].
use std::sync::Arc;

use tessera_ui::AssetExt;

use crate::{
    image_vector::{ImageVectorLoadError, TryIntoImageVectorData},
    pipelines::image_vector::command::ImageVectorData,
};

pub use crate::res::material_icons::{filled, outlined, round, sharp, two_tone};

/// Material icon asset handle.
pub type Asset = crate::res::Asset;

impl TryIntoImageVectorData for Asset {
    fn try_into_image_vector_data(self) -> Result<Arc<ImageVectorData>, ImageVectorLoadError> {
        let bytes = self
            .read()
            .map_err(|source| ImageVectorLoadError::AssetRead { source })?;
        bytes.as_ref().try_into_image_vector_data()
    }
}
