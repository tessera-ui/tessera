use std::{io, path::PathBuf};

fn get_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir()
}

/// Initialize a WGPU pipeline cache from disk if available.
///
/// Returns `None` when the adapter does not support pipeline caches or when the
/// cache directory cannot be created. A missing cache file is not a failure:
/// the cache is still created from empty data with `fallback: true`.
pub fn initialize_cache(
    device: &wgpu::Device,
    adapter_info: &wgpu::AdapterInfo,
) -> Option<wgpu::PipelineCache> {
    // Adapter/feature support is distinct from later filesystem errors.
    if !device.features().contains(wgpu::Features::PIPELINE_CACHE) {
        return None;
    }
    let cache_filename = wgpu::util::pipeline_cache_key(adapter_info)?;
    let cache_dir = get_cache_dir()?;
    if std::fs::create_dir_all(&cache_dir).is_err() {
        return None;
    }

    let cache_path = cache_dir.join(cache_filename);
    // A missing cache file is not a failure. Other read errors abort init.
    // Directory creation failure already returned None above.
    let cache_data = match std::fs::read(&cache_path) {
        Ok(data) => Some(data),
        Err(err) if err.kind() == io::ErrorKind::NotFound => None,
        Err(_) => return None,
    };
    unsafe {
        Some(
            device.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
                label: Some("app_pipeline_cache"),
                data: cache_data.as_deref(),
                fallback: true,
            }),
        )
    }
}

/// Save the WGPU pipeline cache to disk.
///
/// [`io::ErrorKind::Unsupported`] means the adapter cannot key a pipeline
/// cache. Directory creation and write failures propagate as IO errors. A cache
/// file that does not exist yet is created by the write and is not an error.
pub fn save_cache(cache: &wgpu::PipelineCache, adapter_info: &wgpu::AdapterInfo) -> io::Result<()> {
    let cache_filename = wgpu::util::pipeline_cache_key(adapter_info).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "Pipeline cache not supported by adapter",
        )
    })?;
    let cache_dir = get_cache_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cache dir not found"))?;

    std::fs::create_dir_all(&cache_dir)?;
    let cache_path = cache_dir.join(&cache_filename);

    if let Some(data) = cache.get_data() {
        std::fs::write(&cache_path, &data)?;
    }

    Ok(())
}
