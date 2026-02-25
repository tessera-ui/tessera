//! Asset reading abstractions for generated resource handles.
//!
//! ## Usage
//!
//! Use generated asset constants and call [`AssetExt::read`] to load raw bytes.

use std::{io, sync::Arc};

/// Trait implemented by generated asset handle types.
pub trait AssetExt: Copy {
    /// Read raw bytes for this asset.
    fn read(self) -> io::Result<Arc<[u8]>>;
}
