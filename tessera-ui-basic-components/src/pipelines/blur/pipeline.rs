use std::{collections::HashMap, num::NonZeroUsize};

use encase::{ShaderType, UniformBuffer, internal::WriteInto};
use lru::LruCache;
use smallvec::SmallVec;
use tessera_ui::{
    compute::pipeline::{ComputablePipeline, ComputeContext},
    wgpu,
};

use super::command::{DualBlurCommand, downscale_factor_for_radius};

const MAX_SAMPLES: usize = 16;
const WEIGHT_CACHE_CAPACITY: usize = 64;
const WEIGHT_QUANTIZATION: f32 = 100.0;

/// Compute optimized Gaussian blur weights and offsets using hardware bilinear
/// interpolation. This reduces the number of texture samples by leveraging the
/// GPU's built-in linear filtering.
///
/// Returns (weights, offsets, sample_count) where:
/// - weights[0] is the center weight
/// - weights[i] (i > 0) is the weight for both +offset[i] and -offset[i]
/// - offsets[i] is the pixel offset from center
/// - sample_count is the actual number of samples needed
fn compute_optimized_blur_params(radius: f32) -> WeightCacheEntry {
    if radius <= 0.0 {
        let mut weights = [0.0f32; MAX_SAMPLES];
        weights[0] = 1.0;
        return WeightCacheEntry {
            weights,
            offsets: [0.0f32; MAX_SAMPLES],
            sample_count: 1,
        };
    }

    // Standard deviation: radius / 3 gives a good Gaussian falloff
    let sigma = (radius / 3.0).max(0.1);
    let two_sigma_sq = 2.0 * sigma * sigma;

    // Compute discrete Gaussian weights for integer pixel offsets
    let int_radius = radius.ceil() as i32;

    // Compute raw Gaussian weights (not normalized yet)
    let mut raw_weights = SmallVec::<[f32; 64]>::with_capacity((int_radius + 1) as usize);
    raw_weights.resize((int_radius + 1) as usize, 0.0);
    for i in 0..=int_radius {
        let x = i as f32;
        raw_weights[i as usize] = (-x * x / two_sigma_sq).exp();
    }

    // Now apply bilinear optimization by combining adjacent samples
    let mut weights = SmallVec::<[f32; MAX_SAMPLES]>::with_capacity(MAX_SAMPLES);
    let mut offsets = SmallVec::<[f32; MAX_SAMPLES]>::with_capacity(MAX_SAMPLES);

    // Center sample (index 0, not duplicated in shader)
    weights.push(raw_weights[0]);
    offsets.push(0.0);

    // Combine pairs of adjacent samples using bilinear interpolation
    // For each pair (i, i+1), compute the optimal sampling position
    let mut i = 1;
    while i <= int_radius && weights.len() < MAX_SAMPLES {
        let w1 = raw_weights[i as usize];
        let w2 = if i < int_radius {
            raw_weights[(i + 1) as usize]
        } else {
            0.0
        };

        let combined_weight = w1 + w2;
        if combined_weight > 1e-6 {
            // Optimal offset for bilinear sampling to combine w1 at i and w2 at i+1
            let offset = if w2 > 1e-6 {
                (i as f32 * w1 + (i + 1) as f32 * w2) / combined_weight
            } else {
                i as f32
            };

            weights.push(combined_weight);
            offsets.push(offset);

            // Skip next position since we combined it
            i += 2;
        } else {
            i += 1;
        }
    }

    // Normalize weights so that center + 2 * sum(side_weights) = 1.0
    // (factor of 2 because shader samples both +offset and -offset for each side
    // weight)
    let total_weight: f32 = weights[0] + 2.0 * weights[1..].iter().sum::<f32>();
    for w in &mut weights {
        *w /= total_weight;
    }

    // Pad to MAX_SAMPLES
    let sample_count = weights.len() as u32;

    let mut weights_array = [0.0f32; MAX_SAMPLES];
    let mut offsets_array = [0.0f32; MAX_SAMPLES];
    for idx in 0..weights.len() {
        weights_array[idx] = weights[idx];
        offsets_array[idx] = offsets[idx];
    }

    WeightCacheEntry {
        weights: weights_array,
        offsets: offsets_array,
        sample_count,
    }
}

#[derive(Clone)]
struct WeightCacheEntry {
    weights: [f32; MAX_SAMPLES],
    offsets: [f32; MAX_SAMPLES],
    sample_count: u32,
}

#[derive(ShaderType)]
struct BlurUniforms {
    radius: f32,
    direction_x: f32,
    direction_y: f32,
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
    sample_count: u32,
}

#[derive(ShaderType)]
struct WeightsAndOffsets {
    weights: [glam::Vec4; 16],
    offsets: [glam::Vec4; 16],
}

#[derive(ShaderType)]
struct DownsampleUniforms {
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
    scale: u32,
}

#[derive(ShaderType)]
struct UpsampleUniforms {
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
    scale: u32,
}

/// Compute pipeline handling downsample, blur, and upsample passes.
pub struct BlurPipeline {
    downsample_pipeline: wgpu::ComputePipeline,
    blur_pipeline: wgpu::ComputePipeline,
    upsample_pipeline: wgpu::ComputePipeline,
    downsample_bind_group_layout: wgpu::BindGroupLayout,
    blur_bind_group_layout: wgpu::BindGroupLayout,
    upsample_bind_group_layout: wgpu::BindGroupLayout,
    downsample_sampler: wgpu::Sampler,
    texture_pool: HashMap<(u32, u32), Vec<wgpu::Texture>>,
    weight_cache: LruCache<u32, WeightCacheEntry>,
}

impl BlurPipeline {
    /// Builds the blur pipeline, optionally using an existing pipeline cache.
    pub fn new(device: &wgpu::Device, pipeline_cache: Option<&wgpu::PipelineCache>) -> Self {
        let downsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur Downsample Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("downsample.wgsl").into()),
        });
        let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("blur.wgsl").into()),
        });
        let upsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur Upsample Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("upsample.wgsl").into()),
        });

        let downsample_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blur Downsample Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let downsample_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // 0: Uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // 1: Source Texture (Sampled)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // 2: Destination Texture (Storage)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    // 3: Linear sampler for hardware filtering
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("blur_downsample_bind_group_layout"),
            });

        let blur_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // 0: Uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // 1: Source Texture (Sampled)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // 2: Destination Texture (Storage)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    // 3: Linear sampler for hardware bilinear filtering
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // 4: Pre-computed weights and offsets
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("blur_pass_bind_group_layout"),
            });

        let upsample_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // 0: Uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // 1: Source Texture (Sampled)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // 2: Destination Texture (Storage)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    // 3: Linear sampler for filtering
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("blur_upsample_bind_group_layout"),
            });

        let downsample_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Blur Downsample Pipeline Layout"),
                bind_group_layouts: &[&downsample_bind_group_layout],
                immediate_size: 0,
            });
        let blur_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blur Pipeline Layout"),
            bind_group_layouts: &[&blur_bind_group_layout],
            immediate_size: 0,
        });
        let upsample_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Blur Upsample Pipeline Layout"),
                bind_group_layouts: &[&upsample_bind_group_layout],
                immediate_size: 0,
            });

        let downsample_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Blur Downsample Pipeline"),
                layout: Some(&downsample_pipeline_layout),
                module: &downsample_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: pipeline_cache,
            });
        let blur_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Blur Pipeline"),
            layout: Some(&blur_pipeline_layout),
            module: &blur_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: pipeline_cache,
        });
        let upsample_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Blur Upsample Pipeline"),
            layout: Some(&upsample_pipeline_layout),
            module: &upsample_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: pipeline_cache,
        });

        Self {
            downsample_pipeline,
            blur_pipeline,
            upsample_pipeline,
            downsample_bind_group_layout,
            blur_bind_group_layout,
            upsample_bind_group_layout,
            downsample_sampler,
            texture_pool: HashMap::new(),
            weight_cache: LruCache::new(
                NonZeroUsize::new(WEIGHT_CACHE_CAPACITY)
                    .expect("WEIGHT_CACHE_CAPACITY must be non-zero"),
            ),
        }
    }

    fn texture_key(width: u32, height: u32) -> (u32, u32) {
        (width.max(1), height.max(1))
    }

    fn acquire_texture(&mut self, device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        let key = Self::texture_key(width, height);
        if let Some(bucket) = self.texture_pool.get_mut(&key)
            && let Some(texture) = bucket.pop()
        {
            return texture;
        }

        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Blur Intermediate Texture"),
            size: wgpu::Extent3d {
                width: key.0,
                height: key.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        })
    }

    fn release_texture(&mut self, texture: wgpu::Texture, width: u32, height: u32) {
        let key = Self::texture_key(width, height);
        self.texture_pool.entry(key).or_default().push(texture);
    }

    fn quantize_radius(radius: f32) -> u32 {
        ((radius * WEIGHT_QUANTIZATION).round().max(0.0)) as u32
    }

    fn weights_for_radius(&mut self, radius: f32) -> WeightCacheEntry {
        let key = Self::quantize_radius(radius);
        if let Some(entry) = self.weight_cache.get(&key) {
            return entry.clone();
        }

        let computed = compute_optimized_blur_params(radius);
        self.weight_cache.put(key, computed.clone());
        computed
    }

    fn create_uniform_buffer<T: ShaderType + WriteInto>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        data: &T,
    ) -> wgpu::Buffer {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(data).expect("buffer write failed");
        let bytes = buffer.into_inner();
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform_buffer, 0, &bytes);
        uniform_buffer
    }
}

impl ComputablePipeline<DualBlurCommand> for BlurPipeline {
    /// Dispatches one or more blur compute commands within the active pass.
    fn dispatch(&mut self, context: &mut ComputeContext<DualBlurCommand>) {
        for item in context.items {
            let target_area = item.target_area;
            let area_x = target_area.x.0 as u32;
            let area_y = target_area.y.0 as u32;
            let area_width = target_area.width.0 as u32;
            let area_height = target_area.height.0 as u32;

            if area_width == 0 || area_height == 0 {
                continue;
            }

            let max_radius = item
                .command
                .passes
                .iter()
                .map(|pass| pass.radius)
                .fold(0.0f32, f32::max);
            let scale = downscale_factor_for_radius(max_radius).max(1);
            let down_width = area_width.div_ceil(scale);
            let down_height = area_height.div_ceil(scale);

            if down_width == 0 || down_height == 0 {
                continue;
            }

            let downsample_texture = self.acquire_texture(context.device, down_width, down_height);
            let downsample_view =
                downsample_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let blur_texture = self.acquire_texture(context.device, down_width, down_height);
            let blur_view = blur_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Downsample pass
            let downsample_uniforms = DownsampleUniforms {
                area_x,
                area_y,
                area_width,
                area_height,
                scale,
            };
            let downsample_uniform_buffer = Self::create_uniform_buffer(
                context.device,
                context.queue,
                "Blur Downsample Uniform Buffer",
                &downsample_uniforms,
            );
            let downsample_bind_group =
                context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &self.downsample_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: downsample_uniform_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(context.input_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::TextureView(&downsample_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::Sampler(&self.downsample_sampler),
                            },
                        ],
                        label: Some("blur_downsample_bind_group"),
                    });
            context.compute_pass.set_pipeline(&self.downsample_pipeline);
            context
                .compute_pass
                .set_bind_group(0, &downsample_bind_group, &[]);
            let downsample_workgroups_x = down_width.div_ceil(8);
            let downsample_workgroups_y = down_height.div_ceil(8);
            if downsample_workgroups_x == 0 || downsample_workgroups_y == 0 {
                self.release_texture(downsample_texture, down_width, down_height);
                self.release_texture(blur_texture, down_width, down_height);
                continue;
            }
            context.compute_pass.dispatch_workgroups(
                downsample_workgroups_x,
                downsample_workgroups_y,
                1,
            );

            // Directional blur pass
            let mut read_view = downsample_view.clone();
            let mut write_view = blur_view.clone();
            for pass in &item.command.passes {
                let effective_radius = (pass.radius / scale as f32).max(0.0);

                // Fetch cached optimized blur parameters
                let weight_entry = self.weights_for_radius(effective_radius);

                let blur_uniforms = BlurUniforms {
                    radius: effective_radius,
                    direction_x: pass.direction.0,
                    direction_y: pass.direction.1,
                    area_x: 0,
                    area_y: 0,
                    area_width: down_width,
                    area_height: down_height,
                    sample_count: weight_entry.sample_count,
                };
                let blur_uniform_buffer = Self::create_uniform_buffer(
                    context.device,
                    context.queue,
                    "Blur Pass Uniform Buffer",
                    &blur_uniforms,
                );

                // Create weights and offsets buffer (padded to vec4 for alignment)
                let weights_and_offsets = WeightsAndOffsets {
                    weights: std::array::from_fn(|i| {
                        glam::Vec4::new(weight_entry.weights[i], 0.0, 0.0, 0.0)
                    }),
                    offsets: std::array::from_fn(|i| {
                        glam::Vec4::new(weight_entry.offsets[i], 0.0, 0.0, 0.0)
                    }),
                };
                let weights_buffer = Self::create_uniform_buffer(
                    context.device,
                    context.queue,
                    "Blur Weights and Offsets Buffer",
                    &weights_and_offsets,
                );

                let blur_bind_group =
                    context
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.blur_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: blur_uniform_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(&read_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: wgpu::BindingResource::TextureView(&write_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: wgpu::BindingResource::Sampler(
                                        &self.downsample_sampler,
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: weights_buffer.as_entire_binding(),
                                },
                            ],
                            label: Some("blur_directional_bind_group"),
                        });
                context.compute_pass.set_pipeline(&self.blur_pipeline);
                context
                    .compute_pass
                    .set_bind_group(0, &blur_bind_group, &[]);
                context.compute_pass.dispatch_workgroups(
                    downsample_workgroups_x,
                    downsample_workgroups_y,
                    1,
                );

                std::mem::swap(&mut read_view, &mut write_view);
            }

            // Upsample pass
            let upsample_uniforms = UpsampleUniforms {
                area_x,
                area_y,
                area_width,
                area_height,
                scale,
            };
            let upsample_uniform_buffer = Self::create_uniform_buffer(
                context.device,
                context.queue,
                "Blur Upsample Uniform Buffer",
                &upsample_uniforms,
            );
            let upsample_bind_group =
                context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &self.upsample_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: upsample_uniform_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&read_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::TextureView(context.output_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::Sampler(&self.downsample_sampler),
                            },
                        ],
                        label: Some("blur_upsample_bind_group"),
                    });
            context.compute_pass.set_pipeline(&self.upsample_pipeline);
            context
                .compute_pass
                .set_bind_group(0, &upsample_bind_group, &[]);
            let upsample_workgroups_x = area_width.div_ceil(8);
            let upsample_workgroups_y = area_height.div_ceil(8);
            if upsample_workgroups_x == 0 || upsample_workgroups_y == 0 {
                self.release_texture(downsample_texture, down_width, down_height);
                self.release_texture(blur_texture, down_width, down_height);
                continue;
            }
            context.compute_pass.dispatch_workgroups(
                upsample_workgroups_x,
                upsample_workgroups_y,
                1,
            );

            self.release_texture(downsample_texture, down_width, down_height);
            self.release_texture(blur_texture, down_width, down_height);
        }
    }
}
