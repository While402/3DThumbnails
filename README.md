# 3DThumbnails

3DThumbnails is a lightweight Windows Explorer thumbnail provider for 3D model files. It generates preview images directly inside File Explorer, so `.obj`, `.fbx`, `.glb`, and `.gltf` files are easier to browse without opening Blender, a game engine, or a full 3D viewer.

If you work with 3D assets, game models, CAD-adjacent meshes, marketplace downloads, modding files, or quick asset folders, 3DThumbnails helps you visually identify models faster from the standard Windows file browser.

## Features

- Windows Explorer thumbnails for 3D models.
- Supports `.obj`, `.fbx`, `.glb`, and `.gltf`.
- MSI installer for Windows 11 and modern Windows desktop environments.
- Fast Rust thumbnail provider loaded by the Windows Shell.
- Lightweight custom CPU rasterizer, with no Blender dependency and no external 3D engine runtime.
- Diffuse/base-color texture support for OBJ and glTF/GLB assets when textures are available.
- FBX geometry and texture loading through `ufbx`.
- Transparent PNG-style thumbnails with simple lighting and automatic model framing.
- Uninstallable from Windows Installed apps.

## Download

Download the latest installer from the GitHub Releases page:

```text
3DThumbnails-1.0.0-x64.msi
```

Run the MSI, then reopen File Explorer or refresh the folder view. If Windows still shows old thumbnails, switch the folder view size once or clear the Explorer thumbnail cache.

## Supported Formats

| Format | Extension | Notes |
| --- | --- | --- |
| Wavefront OBJ | `.obj` | Geometry, normals, materials, and diffuse textures where resolvable. |
| Autodesk FBX | `.fbx` | Geometry and basic material texture support through `ufbx`. |
| glTF 2.0 | `.gltf` | Scene hierarchy, node transforms, base-color textures, samplers, and `KHR_texture_transform`. |
| Binary glTF | `.glb` | Embedded glTF assets and base-color textures. |

## Why Use 3DThumbnails?

Windows Explorer does not provide native thumbnails for common 3D model formats. 3DThumbnails fills that gap with a small native thumbnail provider focused on quick previews, clean installation, and low overhead.

It is useful for:

- Game developers browsing asset folders.
- 3D artists reviewing exported models.
- Modders organizing OBJ, FBX, GLB, and glTF files.
- Marketplace users sorting downloaded 3D assets.
- Technical artists checking model libraries quickly.

## Technology Stack

3DThumbnails is written in Rust and uses a small purpose-built rendering pipeline instead of bundling a heavyweight viewer.

Key libraries and components:

- `windows` / `windows-core`: Windows COM and Shell integration for `IThumbnailProvider`.
- Custom Rust CPU rasterizer: projects triangles, samples textures, shades the model, and outputs RGBA thumbnails.
- `gltf`: glTF/GLB loading, embedded resources, samplers, and `KHR_texture_transform` support.
- `ufbx`: FBX parsing with broad compatibility and direct access to mesh attributes, UVs, materials, and embedded textures.
- `tobj`: Wavefront OBJ loading with triangulation and single-index mesh data.
- `image`: PNG/JPEG texture decoding and thumbnail image output.
- `glam`: fast vector and matrix math for camera projection, transforms, normals, and UV interpolation.
- `WiX Toolset`: MSI packaging and Windows installation metadata.

No Blender, Unreal Engine, Unity, Assimp runtime, or web browser runtime is required by the installed thumbnail provider.

## Build From Source

Requirements:

- Windows x64.
- Rust stable toolchain.
- WiX Toolset v3 or v4 available in `PATH`, or the local `.tools/wix314` helper used by the build script.

Build the thumbnail provider DLL:

```powershell
cargo build -p thumbnail_provider --release
```

Build the MSI installer:

```powershell
.\scripts\build-msi.ps1
```

The generated installer is written to the repository root:

```text
3DThumbnails-1.0.0-x64.msi
```

## Test The Renderer Without Explorer

The `thumbgen` command-line tool renders a model directly to a PNG file. This is useful for debugging model loading and texture mapping without involving Windows Explorer.

```powershell
cargo run -p thumbgen -- path\to\model.glb out.png 256
```

## Logs

Explorer thumbnail events are written to:

```text
C:\ProgramData\3DThumbnails\3dthumbs.log
```

## Development Notes

The repository is organized as a Rust workspace:

- `crates/renderer`: model loading, CPU rasterization, texture sampling, and thumbnail image generation.
- `crates/thumbnail_provider`: Windows COM thumbnail provider DLL.
- `crates/thumbgen`: command-line renderer for local testing.
- `wix`: MSI installer definition.
- `scripts`: registration, probing, cache-clearing, and packaging helpers.

Generated folders such as `target`, `target-msi`, WiX object files, MSI files, and local tool downloads are ignored by Git.
Windows Explorer 3D thumbnails, 3D model thumbnail provider, OBJ thumbnail viewer, FBX thumbnail preview, GLB thumbnail preview, glTF thumbnail preview, Windows 11 3D file thumbnails, Rust Windows Shell extension, 3D asset browser, game asset thumbnail generator.

## License

MIT
