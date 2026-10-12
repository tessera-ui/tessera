//! GPU timestamp measurements for headless profiling.
//!
//! ## Usage
//!
//! Compare GPU frame execution with CPU profiling in opt-in headless sessions.

use std::{sync::mpsc, time::Duration};

use super::RenderCore;

pub(super) struct GpuTimingProbe {
    queries: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
    timestamp_period: f64,
    capacity: u32,
    point_count: u32,
    stage_labels: Vec<String>,
    detailed: bool,
    frame_index: u64,
}

impl GpuTimingProbe {
    pub(super) const FEATURES: wgpu::Features =
        wgpu::Features::TIMESTAMP_QUERY.union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);

    pub(super) fn requested() -> bool {
        std::env::var("TESSERA_HEADLESS_GPU_TIMING")
            .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
    }

    pub(super) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self::with_capacity(device, queue.get_timestamp_period() as f64, 2)
    }

    fn with_capacity(device: &wgpu::Device, timestamp_period: f64, capacity: u32) -> Self {
        let queries = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Headless GPU timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count: capacity,
        });
        let resolve = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Headless GPU timestamp resolve"),
            size: capacity as u64 * wgpu::QUERY_SIZE as u64,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Headless GPU timestamp readback"),
            size: capacity as u64 * wgpu::QUERY_SIZE as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Self {
            queries,
            resolve,
            readback,
            timestamp_period,
            capacity,
            point_count: 2,
            stage_labels: Vec::new(),
            detailed: false,
            frame_index: 0,
        }
    }

    pub(super) fn begin(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        pass_count: usize,
        frame_index: u64,
    ) {
        let detailed = tracing::enabled!(tracing::Level::DEBUG)
            && pass_count <= (wgpu::QUERY_SET_MAX_QUERIES - 2) as usize;
        let required = if detailed { pass_count as u32 + 2 } else { 2 };
        if required > self.capacity {
            *self =
                Self::with_capacity(device, self.timestamp_period, required.next_power_of_two());
        }
        self.detailed = detailed;
        self.point_count = required;
        self.frame_index = frame_index;
        self.stage_labels.clear();
        encoder.write_timestamp(&self.queries, 0);
    }

    pub(super) fn mark(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        label: impl FnOnce() -> String,
    ) {
        if self.detailed {
            self.stage_labels.push(label());
            encoder.write_timestamp(&self.queries, self.stage_labels.len() as u32);
        }
    }

    pub(super) fn finish(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if self.detailed {
            self.mark(encoder, || "Output blit".to_owned());
        } else {
            encoder.write_timestamp(&self.queries, 1);
        }
        encoder.resolve_query_set(&self.queries, 0..self.point_count, &self.resolve, 0);
        encoder.copy_buffer_to_buffer(
            &self.resolve,
            0,
            &self.readback,
            0,
            self.point_count as u64 * wgpu::QUERY_SIZE as u64,
        );
    }

    fn read(&self, device: &wgpu::Device) -> Result<Duration, String> {
        let slice = self
            .readback
            .slice(..self.point_count as u64 * wgpu::QUERY_SIZE as u64);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        let result = (|| {
            device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(Duration::from_secs(5)),
                })
                .map_err(|err| err.to_string())?;
            rx.recv_timeout(Duration::from_secs(5))
                .map_err(|err| err.to_string())?
                .map_err(|err| err.to_string())?;
            let mapped = slice.get_mapped_range().map_err(|err| err.to_string())?;
            let start = u64::from_ne_bytes(mapped[..8].try_into().unwrap());
            let end = u64::from_ne_bytes(mapped[mapped.len() - 8..].try_into().unwrap());
            if self.detailed {
                let timestamps: Vec<u64> = mapped
                    .chunks_exact(8)
                    .map(|bytes| u64::from_ne_bytes(bytes.try_into().unwrap()))
                    .collect();
                for (index, (label, timestamps)) in self
                    .stage_labels
                    .iter()
                    .zip(timestamps.windows(2))
                    .enumerate()
                {
                    tracing::debug!(
                        frame_index = self.frame_index,
                        stage_index = index,
                        label,
                        elapsed_ns = (timestamps[1].saturating_sub(timestamps[0]) as f64
                            * self.timestamp_period) as u64,
                        "gpu_stage"
                    );
                }
            }
            let nanos = end.saturating_sub(start) as f64 * self.timestamp_period;
            Ok(Duration::from_nanos(nanos.min(u64::MAX as f64) as u64))
        })();
        self.readback.unmap();
        result
    }
}

impl RenderCore {
    pub(crate) fn read_gpu_time(&mut self) -> Option<Duration> {
        let result = self.gpu_timing.as_ref()?.read(&self.device);
        match result {
            Ok(duration) => Some(duration),
            Err(err) => {
                tracing::warn!("Disabling headless GPU timing after readback failure: {err}");
                self.gpu_timing = None;
                None
            }
        }
    }
}
