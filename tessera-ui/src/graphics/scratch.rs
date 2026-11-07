use std::{ops::Deref, sync::Arc};

use parking_lot::Mutex;

/// How a scratch texture's size relates to the requested size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScratchFit {
    /// The texture has exactly the requested size, for attachments that must
    /// match another target.
    Exact,
    /// The texture may be larger, so nearby sizes share pooled textures.
    /// Address it through its actual size.
    Approx,
}

/// Describes a pooled texture requested with
/// [`GraphicsDevice::scratch_texture`](crate::GraphicsDevice::scratch_texture).
#[derive(Clone, Copy, Debug)]
pub struct ScratchTextureDescriptor {
    /// Minimum width in texels.
    pub width: u32,
    /// Minimum height in texels.
    pub height: u32,
    /// Texture format.
    pub format: wgpu::TextureFormat,
    /// Required usages.
    pub usage: wgpu::TextureUsages,
    /// MSAA sample count.
    pub sample_count: u32,
    /// Size rounding.
    pub fit: ScratchFit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Key {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    sample_count: u32,
}

#[derive(Debug)]
struct Entry {
    key: Key,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    last_used: u64,
}

#[derive(Debug, Default)]
pub(crate) struct ScratchPool {
    free: Vec<Entry>,
    frame: u64,
}

// Pooled textures unused for this many frames are released.
const RETAINED_FRAMES: u64 = 2;

impl ScratchPool {
    pub(crate) fn begin_frame(&mut self) {
        self.frame += 1;
        let frame = self.frame;
        self.free
            .retain(|entry| frame - entry.last_used <= RETAINED_FRAMES);
    }
}

/// Rounds a dimension up so nearby sizes share pooled textures: small sizes
/// round to 16, sizes up to 1024 to the next power of two, and larger sizes to
/// the next half step between powers of two.
fn approx_dimension(value: u32) -> u32 {
    const MIN: u32 = 16;
    const POW2_LIMIT: u32 = 1024;
    if value <= MIN {
        return MIN;
    }
    let ceil = value.next_power_of_two();
    if value <= POW2_LIMIT || value == ceil {
        return ceil;
    }
    let mid = (ceil >> 1) + (ceil >> 2);
    if value <= mid { mid } else { ceil }
}

/// A pooled texture. Work recorded after it is dropped may reuse it, so keep
/// it until all work that uses its contents is recorded.
pub struct ScratchTexture {
    entry: Option<Entry>,
    pool: Arc<Mutex<ScratchPool>>,
}

impl ScratchTexture {
    /// Returns the underlying texture.
    pub fn texture(&self) -> &wgpu::Texture {
        &self.entry().texture
    }

    /// Returns the actual size, which may exceed the requested size.
    pub fn size(&self) -> (u32, u32) {
        let key = self.entry().key;
        (key.width, key.height)
    }

    fn entry(&self) -> &Entry {
        self.entry.as_ref().expect("scratch texture is leased")
    }
}

impl Deref for ScratchTexture {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.entry().view
    }
}

impl Drop for ScratchTexture {
    fn drop(&mut self) {
        if let Some(mut entry) = self.entry.take() {
            let mut pool = self.pool.lock();
            entry.last_used = pool.frame;
            pool.free.push(entry);
        }
    }
}

impl crate::GraphicsDevice {
    /// Leases a pooled texture for work recorded in the current frame.
    pub fn scratch_texture(&self, descriptor: ScratchTextureDescriptor) -> ScratchTexture {
        let (width, height) = match descriptor.fit {
            ScratchFit::Exact => (descriptor.width.max(1), descriptor.height.max(1)),
            ScratchFit::Approx => (
                approx_dimension(descriptor.width),
                approx_dimension(descriptor.height),
            ),
        };
        let key = Key {
            width,
            height,
            format: descriptor.format,
            usage: descriptor.usage,
            sample_count: descriptor.sample_count,
        };
        let reused = {
            let mut pool = self.scratch.lock();
            pool.free
                .iter()
                .position(|entry| entry.key == key)
                .map(|index| pool.free.swap_remove(index))
        };
        let entry = reused.unwrap_or_else(|| {
            let texture = self.create_texture(
                "scratch_textures",
                &wgpu::TextureDescriptor {
                    label: Some("Scratch Texture"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: key.sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    format: key.format,
                    usage: key.usage,
                    view_formats: &[],
                },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            Entry {
                key,
                texture,
                view,
                last_used: 0,
            }
        });
        ScratchTexture {
            entry: Some(entry),
            pool: Arc::clone(&self.scratch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::approx_dimension;

    #[test]
    fn approximate_sizes_share_buckets_without_wasting_large_targets() {
        assert_eq!(approx_dimension(1), 16);
        assert_eq!(approx_dimension(17), 32);
        assert_eq!(approx_dimension(1000), 1024);
        assert_eq!(approx_dimension(1025), 1536);
        assert_eq!(approx_dimension(1536), 1536);
        assert_eq!(approx_dimension(1537), 2048);
        assert_eq!(approx_dimension(2048), 2048);
    }
}
