use std::{cell::Cell, rc::Rc};

use glow::HasContext as _;
use khronos_egl as egl;
use wgpu::hal::gles::{Api as Gl, TextureInner};

use crate::android::platform_views::AndroidSliceSurface;

pub(super) struct SlicePresenter(Rc<Context>);

struct Context {
    device: wgpu::Device,
    display: egl::Display,
    config: egl::Config,
    context: egl::Context,
    framebuffer: Cell<Option<glow::Framebuffer>>,
    copied: Cell<Option<glow::Fence>>,
}

pub(super) struct SliceSurface {
    context: Rc<Context>,
    surface: Option<egl::Surface>,
    pub(super) texture: wgpu::Texture,
}

impl SlicePresenter {
    pub(super) fn new(device: &wgpu::Device) -> Result<Self, String> {
        let hal = unsafe { device.as_hal::<Gl>() }.ok_or("Missing WGPU GLES device")?;
        let source = hal.context();
        let display = *source.raw_display().ok_or("Missing WGPU EGL display")?;
        let egl = source.egl_instance().ok_or("Missing WGPU EGL instance")?;
        let config = egl
            .choose_first_config(
                display,
                &[
                    egl::SURFACE_TYPE,
                    egl::WINDOW_BIT,
                    egl::RENDERABLE_TYPE,
                    egl::OPENGL_ES3_BIT,
                    egl::RED_SIZE,
                    8,
                    egl::GREEN_SIZE,
                    8,
                    egl::BLUE_SIZE,
                    8,
                    egl::ALPHA_SIZE,
                    8,
                    egl::SAMPLE_BUFFERS,
                    0,
                    egl::NONE,
                ],
            )
            .map_err(|error| format!("Choosing an EGL slice configuration: {error}"))?
            .ok_or("No RGBA EGL window configuration")?;
        egl.bind_api(egl::OPENGL_ES_API)
            .map_err(|error| format!("Binding EGL to OpenGL ES: {error}"))?;
        // EGL chooses the default framebuffer's alpha bits from the context's config,
        // not from the ANativeWindow's format. Share WGPU textures with an RGBA context.
        let raw_context = source.raw_context();
        if raw_context.is_null() {
            return Err("Missing WGPU EGL context".into());
        }
        let share = unsafe { egl::Context::from_ptr(raw_context) };
        let context = egl
            .create_context(
                display,
                config,
                Some(share),
                &[egl::CONTEXT_CLIENT_VERSION, 3, egl::NONE],
            )
            .map_err(|error| format!("Creating the shared EGL slice context: {error}"))?;
        Ok(Self(Rc::new(Context {
            device: device.clone(),
            display,
            config,
            context,
            framebuffer: Cell::new(None),
            copied: Cell::new(None),
        })))
    }

    pub(super) fn create_surface(
        &self,
        window: &AndroidSliceSurface,
        size: (u32, u32),
    ) -> Result<SliceSurface, String> {
        let context = &self.0;
        Ok(SliceSurface {
            context: context.clone(),
            surface: Some(context.create_window_surface(window)?),
            texture: slice_texture(&context.device, size),
        })
    }

    pub(super) fn wait_before_rendering(&self) {
        if let Some(fence) = self.0.copied.take()
            && let Some(hal) = unsafe { self.0.device.as_hal::<Gl>() }
        {
            let gl = hal.context().lock();
            unsafe {
                gl.wait_sync(fence, 0, glow::TIMEOUT_IGNORED);
                gl.delete_sync(fence);
            }
        }
    }

    pub(super) fn present(&self, slices: &[&SliceSurface]) -> Result<(), String> {
        let context = &self.0;
        let textures = slices
            .iter()
            .map(|slice| {
                let texture =
                    unsafe { slice.texture.as_hal::<Gl>() }.ok_or("Missing EGL slice texture")?;
                match texture.inner {
                    TextureInner::Texture { raw, .. } => Ok(raw),
                    _ => Err("The EGL slice requires a shareable texture"),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let hal = unsafe { context.device.as_hal::<Gl>() }.ok_or("Missing WGPU GLES device")?;
        let source = hal.context();
        // Keep WGPU out of its GL context while calling the same EGL-loaded entrypoints
        // with the shared context current. Dropping the guard unbinds the current context.
        let gl = source.lock();
        let egl = source.egl_instance().ok_or("Missing WGPU EGL instance")?;
        let mut ready = Some(unsafe { gl.fence_sync(glow::SYNC_GPU_COMMANDS_COMPLETE, 0) }?);
        unsafe { gl.flush() };
        let result = (|| {
            for (slice, texture) in slices.iter().zip(textures) {
                let surface = slice.surface.ok_or("Missing EGL window surface")?;
                egl.make_current(
                    context.display,
                    Some(surface),
                    Some(surface),
                    Some(context.context),
                )
                .map_err(|error| format!("Binding the EGL slice surface: {error}"))?;
                if let Some(fence) = ready.take() {
                    unsafe {
                        gl.wait_sync(fence, 0, glow::TIMEOUT_IGNORED);
                        gl.delete_sync(fence);
                    }
                }
                let framebuffer = if let Some(framebuffer) = context.framebuffer.get() {
                    framebuffer
                } else {
                    let framebuffer = unsafe { gl.create_framebuffer() }?;
                    context.framebuffer.set(Some(framebuffer));
                    framebuffer
                };
                let width = slice.texture.width() as i32;
                let height = slice.texture.height() as i32;
                unsafe {
                    gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(framebuffer));
                    gl.framebuffer_texture_2d(
                        glow::READ_FRAMEBUFFER,
                        glow::COLOR_ATTACHMENT0,
                        glow::TEXTURE_2D,
                        Some(texture),
                        0,
                    );
                    gl.blit_framebuffer(
                        0,
                        height,
                        width,
                        0,
                        0,
                        0,
                        width,
                        height,
                        glow::COLOR_BUFFER_BIT,
                        glow::NEAREST,
                    );
                    gl.framebuffer_texture_2d(
                        glow::READ_FRAMEBUFFER,
                        glow::COLOR_ATTACHMENT0,
                        glow::TEXTURE_2D,
                        None,
                        0,
                    );
                }
                egl.swap_buffers(context.display, surface)
                    .map_err(|error| format!("Presenting the EGL slice surface: {error}"))?;
            }
            Ok(())
        })();
        if let Some(fence) = ready {
            unsafe { gl.delete_sync(fence) };
        } else {
            // No WGPU pass may overwrite a staging texture before the RGBA context has read it.
            match unsafe { gl.fence_sync(glow::SYNC_GPU_COMMANDS_COMPLETE, 0) } {
                Ok(fence) => {
                    context.copied.set(Some(fence));
                    unsafe { gl.flush() };
                }
                Err(_) => unsafe { gl.finish() },
            }
        }
        // The WGPU guard unbinds the context even if EGL failed mid-frame.
        result
    }
}

impl SliceSurface {
    pub(super) fn resize(
        &mut self,
        window: &AndroidSliceSurface,
        size: (u32, u32),
    ) -> Result<(), String> {
        // Legacy EGL window surfaces can retain the old drawable extent after
        // their producer's default size changes. Reconnect the producer on resize.
        self.destroy_surface();
        self.surface = Some(self.context.create_window_surface(window)?);
        self.texture = slice_texture(&self.context.device, size);
        Ok(())
    }

    fn destroy_surface(&mut self) {
        if let Some(surface) = self.surface.take()
            && let Some(hal) = unsafe { self.context.device.as_hal::<Gl>() }
            && let Some(egl) = hal.context().egl_instance()
        {
            let _ = egl.destroy_surface(self.context.display, surface);
        }
    }
}

impl Drop for SliceSurface {
    fn drop(&mut self) {
        self.destroy_surface();
    }
}

impl Context {
    fn create_window_surface(&self, window: &AndroidSliceSurface) -> Result<egl::Surface, String> {
        let hal = unsafe { self.device.as_hal::<Gl>() }.ok_or("Missing WGPU GLES device")?;
        let egl = hal
            .context()
            .egl_instance()
            .ok_or("Missing WGPU EGL instance")?;
        // The owner destroys this EGL surface before releasing the native window reference.
        unsafe {
            egl.create_window_surface(
                self.display,
                self.config,
                window.native_window().as_ptr().cast(),
                None,
            )
        }
        .map_err(|error| format!("Creating an RGBA EGL slice surface: {error}"))
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if let Some(hal) = unsafe { self.device.as_hal::<Gl>() } {
            let source = hal.context();
            if let Some(egl) = source.egl_instance() {
                if let Some(fence) = self.copied.take() {
                    let gl = source.lock();
                    unsafe { gl.delete_sync(fence) };
                }
                let _ = egl.destroy_context(self.display, self.context);
            }
        }
    }
}

fn slice_texture(device: &wgpu::Device, (width, height): (u32, u32)) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Android EGL scene slice"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        // TEXTURE_BINDING keeps GLES from allocating a context-private framebuffer object.
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}
