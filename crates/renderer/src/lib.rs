mod loaders;
mod raster;

use std::path::Path;

pub use raster::RgbaBitmap;

#[derive(Clone, Debug)]
pub struct RenderOptions {
    pub size: u32,
    pub max_triangles: usize,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            size: 256,
            max_triangles: 120_000,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RenderError {
    #[error("unsupported model format")]
    UnsupportedFormat,
    #[error("model has no renderable triangles")]
    EmptyModel,
    #[error(transparent)]
    Load(#[from] anyhow::Error),
}

pub fn render_thumbnail(
    path: impl AsRef<Path>,
    options: &RenderOptions,
) -> Result<RgbaBitmap, RenderError> {
    let path = path.as_ref();
    let mut scene = loaders::load_scene(path)?;

    if scene.triangles.is_empty() {
        return Err(RenderError::EmptyModel);
    }

    if scene.triangles.len() > options.max_triangles {
        scene.triangles.truncate(options.max_triangles);
    }

    Ok(raster::render(&scene, options.size.max(32).min(1024)))
}

#[derive(Clone)]
pub(crate) struct Scene {
    triangles: Vec<Triangle>,
}

impl Scene {
    fn new() -> Self {
        Self {
            triangles: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Triangle {
    vertices: [Vertex; 3],
    color: [u8; 4],
    texture: Option<std::sync::Arc<Texture>>,
}

#[derive(Clone, Copy)]
pub(crate) struct Vertex {
    position: glam::Vec3,
    normal: glam::Vec3,
    uv: glam::Vec2,
}

#[derive(Clone)]
pub(crate) struct Texture {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    wrap_s: WrapMode,
    wrap_t: WrapMode,
    flip_v: bool,
}

impl Texture {
    fn from_image(image: image::DynamicImage) -> Self {
        Self::from_image_with_orientation(image, true)
    }

    pub(crate) fn from_gltf_image(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self {
            width,
            height,
            pixels,
            wrap_s: WrapMode::Repeat,
            wrap_t: WrapMode::Repeat,
            flip_v: false,
        }
    }

    fn from_image_with_orientation(image: image::DynamicImage, flip_v: bool) -> Self {
        let rgba = image.to_rgba8();
        Self {
            width: rgba.width(),
            height: rgba.height(),
            pixels: rgba.into_raw(),
            wrap_s: WrapMode::Repeat,
            wrap_t: WrapMode::Repeat,
            flip_v,
        }
    }

    pub(crate) fn with_wrap(mut self, wrap_s: WrapMode, wrap_t: WrapMode) -> Self {
        self.wrap_s = wrap_s;
        self.wrap_t = wrap_t;
        self
    }

    fn sample(&self, uv: glam::Vec2) -> [u8; 4] {
        let u = self.wrap_s.apply(uv.x);
        let raw_v = if self.flip_v { 1.0 - uv.y } else { uv.y };
        let v = self.wrap_t.apply(raw_v);
        let x = u * self.width.saturating_sub(1) as f32;
        let y = v * self.height.saturating_sub(1) as f32;
        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(self.width.saturating_sub(1));
        let y1 = (y0 + 1).min(self.height.saturating_sub(1));
        let tx = x - x0 as f32;
        let ty = y - y0 as f32;
        let c00 = self.pixel(x0, y0);
        let c10 = self.pixel(x1, y0);
        let c01 = self.pixel(x0, y1);
        let c11 = self.pixel(x1, y1);
        let mut out = [0u8; 4];
        for i in 0..4 {
            let top = c00[i] as f32 * (1.0 - tx) + c10[i] as f32 * tx;
            let bottom = c01[i] as f32 * (1.0 - tx) + c11[i] as f32 * tx;
            out[i] = (top * (1.0 - ty) + bottom * ty).round().clamp(0.0, 255.0) as u8;
        }
        out
    }

    fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }
}

#[derive(Clone, Copy)]
pub(crate) enum WrapMode {
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}

impl WrapMode {
    fn apply(self, value: f32) -> f32 {
        match self {
            WrapMode::ClampToEdge => value.clamp(0.0, 1.0),
            WrapMode::MirroredRepeat => {
                let wrapped = value.rem_euclid(2.0);
                if wrapped <= 1.0 {
                    wrapped
                } else {
                    2.0 - wrapped
                }
            }
            WrapMode::Repeat => value.rem_euclid(1.0),
        }
    }
}
