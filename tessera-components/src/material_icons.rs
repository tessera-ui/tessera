//! Material Design icon content helpers.
//!
//! ## Usage
//!
//! Use style modules (e.g., [`filled`]) and functions like `home_icon()` to get
//! an [`IconContent`] that can be passed to [`crate::icon::IconArgs`].
use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;

use crate::{
    icon::IconContent, image_vector::ImageVectorSource,
    pipelines::image_vector::command::ImageVectorData,
};

pub use generated::{filled, outlined, round, sharp, two_tone};

#[allow(missing_docs, clippy::all)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/material_icons.rs"));
}

type IconCache = HashMap<(usize, usize), Arc<ImageVectorData>>;

static ICON_CACHE: std::sync::OnceLock<RwLock<IconCache>> = std::sync::OnceLock::new();

fn cache() -> &'static RwLock<IconCache> {
    ICON_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Load vector data from the bundled blob with caching.
pub fn load_icon_bytes(bytes: &'static [u8]) -> Arc<ImageVectorData> {
    let key = (bytes.as_ptr() as usize, bytes.len());
    if let Some(cached) = cache().read().get(&key) {
        return cached.clone();
    }

    let bytes = Arc::<[u8]>::from(bytes);
    let vector =
        crate::image_vector::load_image_vector_from_source(&ImageVectorSource::Bytes(bytes))
            .map(Arc::new)
            .expect("bundled material icon svg should load");

    cache().write().insert(key, vector.clone());
    vector
}

/// Convert loaded vector data into icon content.
pub fn content_from_data(data: Arc<ImageVectorData>) -> IconContent {
    IconContent::from(data)
}
