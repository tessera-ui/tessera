use std::{io, path::PathBuf};

fn get_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir()
}

/// Initialize a WGPU pipeline cache from disk if available
pub fn initialize_cache(
    device: &wgpu::Device,
    adapter_info: &wgpu::AdapterInfo,
) -> Option<wgpu::PipelineCache> {
    let cache_dir = get_cache_dir()?;
    let cache_path = cache_dir.join(wgpu::util::pipeline_cache_key(adapter_info)?);
    let cache_data = std::fs::read(&cache_path).ok();
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

/// Save the WGPU pipeline cache to disk
pub fn save_cache(cache: &wgpu::PipelineCache, adapter_info: &wgpu::AdapterInfo) -> io::Result<()> {
    let cache_dir = get_cache_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cache dir not found"))?;
    let cache_filename = wgpu::util::pipeline_cache_key(adapter_info)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, "Cache not supported"))?;

    let cache_path = cache_dir.join(&cache_filename);

    if let Some(data) = cache.get_data() {
        std::fs::write(&cache_path, &data)?;
    }

    Ok(())
}
