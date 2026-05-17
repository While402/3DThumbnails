use std::path::PathBuf;

use anyhow::{bail, Context};
use renderer::{render_thumbnail, RenderOptions};

fn main() -> anyhow::Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() < 2 {
        bail!("usage: thumbgen <model.obj|model.fbx|model.glb|model.gltf> <out.png> [size]");
    }

    let input = PathBuf::from(&args[0]);
    let output = PathBuf::from(&args[1]);
    let size = args
        .get(2)
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(256);

    let bitmap = render_thumbnail(
        &input,
        &RenderOptions {
            size,
            ..Default::default()
        },
    )
    .with_context(|| format!("failed to render {}", input.display()))?;

    image::save_buffer(
        &output,
        &bitmap.pixels,
        bitmap.width,
        bitmap.height,
        image::ColorType::Rgba8,
    )
    .with_context(|| format!("failed to write {}", output.display()))?;

    Ok(())
}
