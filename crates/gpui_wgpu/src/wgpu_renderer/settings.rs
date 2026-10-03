use std::sync::atomic::{AtomicU32, Ordering};

use gpui::{get_gamma_correction_ratios, DevicePixels, Size};

/// Logical-scene → surface-pixel scale factor for the render-scale knob
/// (`set_render_scale`). Stored as percent so an atomic suffices. Rendering
/// below native resolution trades sharpness for fill rate — the dominant GPU
/// cost on budget mobile GPUs (the quad/text shaders are fill-bound).
static RENDER_SCALE_PERCENT: AtomicU32 = AtomicU32::new(100);

/// Sets the render scale (clamped to 0.25..=1.0). Affects every renderer that
/// resizes afterwards; call `WgpuRenderer::set_render_scale` to apply it to a
/// live surface immediately.
pub fn set_render_scale(scale: f32) {
    let percent = (scale.clamp(0.25, 1.0) * 100.0).round() as u32;
    RENDER_SCALE_PERCENT.store(percent, Ordering::SeqCst);
}

/// The current render scale (1.0 = native resolution).
pub fn render_scale() -> f32 {
    RENDER_SCALE_PERCENT.load(Ordering::SeqCst) as f32 / 100.0
}

pub struct WgpuSurfaceConfig {
    pub size: Size<DevicePixels>,
    pub transparent: bool,
    /// Preferred presentation mode, falling back to FIFO when unsupported.
    pub preferred_present_mode: Option<wgpu::PresentMode>,
}

/// Physical order of the color components in an LCD subpixel layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubpixelOrder {
    RedGreenBlue,
    BlueGreenRed,
}

/// Platform text-rasterization settings consumed by the glyph shaders.
#[derive(Clone, Copy, Debug)]
pub struct FontRasterizationSettings {
    pub(super) gamma_ratios: [f32; 4],
    pub(super) grayscale_enhanced_contrast: f32,
    pub(super) subpixel_enhanced_contrast: f32,
    pub(super) subpixel_order: SubpixelOrder,
}

impl FontRasterizationSettings {
    /// Creates settings equivalent to the retired DirectWrite renderer's parameters.
    pub fn new(
        gamma: f32,
        grayscale_enhanced_contrast: f32,
        subpixel_enhanced_contrast: f32,
        subpixel_order: SubpixelOrder,
    ) -> Self {
        Self {
            gamma_ratios: get_gamma_correction_ratios(gamma.clamp(1.0, 2.2)),
            grayscale_enhanced_contrast: finite_contrast(grayscale_enhanced_contrast),
            subpixel_enhanced_contrast: finite_contrast(subpixel_enhanced_contrast),
            subpixel_order,
        }
    }

    /// Disables contrast and gamma correction, like the retired macOS Metal glyph shader.
    pub fn uncorrected() -> Self {
        Self::new(1.0, 0.0, 0.0, SubpixelOrder::RedGreenBlue)
    }

    fn from_environment() -> Self {
        let gamma = environment_f32("ZED_FONTS_GAMMA", 1.8).clamp(1.0, 2.2);
        let grayscale_enhanced_contrast =
            environment_f32("ZED_FONTS_GRAYSCALE_ENHANCED_CONTRAST", 1.0).max(0.0);
        let subpixel_enhanced_contrast =
            environment_f32("ZED_FONTS_SUBPIXEL_ENHANCED_CONTRAST", 0.5).max(0.0);
        Self::new(
            gamma,
            grayscale_enhanced_contrast,
            subpixel_enhanced_contrast,
            SubpixelOrder::RedGreenBlue,
        )
    }
}

pub(super) struct RenderingParameters {
    pub(super) path_sample_count: u32,
    pub(super) font_rasterization: FontRasterizationSettings,
}

impl RenderingParameters {
    pub(super) fn new(adapter: &wgpu::Adapter, surface_format: wgpu::TextureFormat) -> Self {
        let format_features = adapter.get_texture_format_features(surface_format);
        // Adreno 610-class phones: 4× MSAA on the (full-screen) path pass
        // dominated paint time (~25 ms of 39 ms at 1080×2400), capping UI
        // frame rate on Android. Default to 1× there; desktop keeps MSAA 4.
        // Override with MDRV_PATH_MSAA=1|2|4.
        let path_sample_count = std::env::var("MDRV_PATH_MSAA")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|&sc| matches!(sc, 1 | 2 | 4))
            .unwrap_or(if cfg!(target_os = "android") {
                1
            } else {
                [4, 2, 1]
                    .into_iter()
                    .find(|&sample_count| {
                        format_features.flags.sample_count_supported(sample_count)
                    })
                    .unwrap_or(1)
            });
        Self {
            path_sample_count,
            font_rasterization: FontRasterizationSettings::from_environment(),
        }
    }
}

fn environment_f32(name: &str, default: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite())
        .unwrap_or(default)
}

fn finite_contrast(contrast: f32) -> f32 {
    // Infinity survives max(0), then produces inf/inf (or inf*0) in the coverage
    // shader. Fall back to neutral correction instead of making every glyph NaN.
    if contrast.is_finite() {
        contrast.max(0.0)
    } else {
        0.0
    }
}
