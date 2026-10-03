mod cosmic_text_system;
mod wgpu_atlas;
mod wgpu_context;
mod wgpu_renderer;

pub use cosmic_text_system::*;
pub use wgpu;
pub use wgpu_atlas::*;
pub use wgpu_context::*;
#[cfg(all(feature = "test-support", not(target_family = "wasm")))]
pub use wgpu_renderer::WgpuHeadlessRenderer;
pub use wgpu_renderer::{
    FontRasterizationSettings, GpuContext, SubpixelOrder, WgpuRenderer, WgpuSurfaceConfig,
    render_scale, set_render_scale,
};

/// Frame-cost diagnostics for on-device HUDs (the test phone's MIUI ROM ships
/// with logd disabled, so `log` output is unreachable). Primitive counts are
/// refreshed once per frame by `frame::FrameRequirements::for_scene`; phase
/// timings by `WgpuRenderer::draw`. `acquire` includes GPU back-pressure from
/// the *previous* frame's present, so a large acquire means GPU-bound.
pub mod perf {
    use std::sync::atomic::{AtomicU32, Ordering};

    pub static QUADS: AtomicU32 = AtomicU32::new(0);
    pub static PATH_VERTS: AtomicU32 = AtomicU32::new(0);
    pub static SPRITES: AtomicU32 = AtomicU32::new(0);
    pub static OTHER: AtomicU32 = AtomicU32::new(0);
    pub static ACQUIRE_MS: AtomicU32 = AtomicU32::new(0);
    pub static RENDER_MS: AtomicU32 = AtomicU32::new(0);
    pub static PRESENT_MS: AtomicU32 = AtomicU32::new(0);
    pub static FRAMES: AtomicU32 = AtomicU32::new(0);
    /// Milliseconds between consecutive draw() entries (0 = first frame).
    pub static FRAME_PERIOD_MS: AtomicU32 = AtomicU32::new(0);
    /// CPU wait for GPU completion right after submit (device.poll(Wait)).
    pub static GPU_WAIT_MS: AtomicU32 = AtomicU32::new(0);
    /// wgpu::PresentMode as u32 actually configured on the surface
    /// (0=Fifo 1=FifoRelaxed 2=Mailbox 3=Immediate 4=AutoVsync 5=AutoNoVsync).
    pub static PRESENT_MODE: AtomicU32 = AtomicU32::new(u32::MAX);
    /// Render passes begun this frame (reset at each draw entry).
    pub static PASS_COUNT: AtomicU32 = AtomicU32::new(0);
    /// wgpu::Backend as u32 (1=Vulkan 2=Metal 3=Dx12 4=GL 5=BrowserWebGpu).
    pub static BACKEND: AtomicU32 = AtomicU32::new(u32::MAX);
    /// Adapter name, first 16 bytes as 4 little-endian u32s.
    pub static NAME0: AtomicU32 = AtomicU32::new(0);
    pub static NAME1: AtomicU32 = AtomicU32::new(0);
    pub static NAME2: AtomicU32 = AtomicU32::new(0);
    pub static NAME3: AtomicU32 = AtomicU32::new(0);

    /// (quads, path_verts, sprites, other, acquire_ms, render_ms, present_ms, frames)
    pub fn snapshot() -> (u32, u32, u32, u32, u32, u32, u32, u32) {
        (
            QUADS.load(Ordering::Relaxed),
            PATH_VERTS.load(Ordering::Relaxed),
            SPRITES.load(Ordering::Relaxed),
            OTHER.load(Ordering::Relaxed),
            ACQUIRE_MS.load(Ordering::Relaxed),
            RENDER_MS.load(Ordering::Relaxed),
            PRESENT_MS.load(Ordering::Relaxed),
            FRAMES.load(Ordering::Relaxed),
        )
    }
}
