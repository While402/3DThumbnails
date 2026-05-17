use std::{
    ffi::c_void,
    fs::{create_dir_all, OpenOptions},
    io::Write,
    path::PathBuf,
    ptr::null_mut,
    sync::Mutex,
};

use once_cell::sync::Lazy;
use renderer::{render_thumbnail, RenderOptions};
use windows::{
    core::{implement, Error, IUnknown, Interface, Result, GUID, HRESULT, PCWSTR},
    Win32::{
        Foundation::{
            BOOL, CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_POINTER, S_FALSE,
            S_OK,
        },
        Graphics::Gdi::{
            CreateDIBSection, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP,
        },
        System::Com::{
            CoTaskMemFree, IClassFactory, IClassFactory_Impl, IStream, STATFLAG_NONAME,
            STREAM_SEEK_SET,
        },
        UI::Shell::PropertiesSystem::{
            IInitializeWithFile, IInitializeWithFile_Impl, IInitializeWithStream,
            IInitializeWithStream_Impl,
        },
        UI::Shell::{
            IInitializeWithItem, IInitializeWithItem_Impl, IThumbnailProvider,
            IThumbnailProvider_Impl, SIGDN_FILESYSPATH, WTSAT_ARGB, WTS_ALPHATYPE,
        },
    },
};

const PROVIDER_VERSION: &str = env!("CARGO_PKG_VERSION");

const CLSID_OBJ_PROVIDER: GUID = GUID::from_u128(0x0ef2c8d1_7b70_48c9_b7b8_0f45d3d00001);
const CLSID_FBX_PROVIDER: GUID = GUID::from_u128(0x0ef2c8d1_7b70_48c9_b7b8_0f45d3d00002);
const CLSID_GLB_PROVIDER: GUID = GUID::from_u128(0x0ef2c8d1_7b70_48c9_b7b8_0f45d3d00003);
const CLSID_GLTF_PROVIDER: GUID = GUID::from_u128(0x0ef2c8d1_7b70_48c9_b7b8_0f45d3d00004);
const CLSID_LEGACY_PROVIDER: GUID = GUID::from_u128(0x4c6f2b8a_5d2e_4c64_9ac7_b6fd046a8241);

#[derive(Clone, Copy)]
struct ProviderInfo {
    clsid: GUID,
    extension: &'static str,
}

const PROVIDERS: [ProviderInfo; 5] = [
    ProviderInfo {
        clsid: CLSID_LEGACY_PROVIDER,
        extension: ".model",
    },
    ProviderInfo {
        clsid: CLSID_OBJ_PROVIDER,
        extension: ".obj",
    },
    ProviderInfo {
        clsid: CLSID_FBX_PROVIDER,
        extension: ".fbx",
    },
    ProviderInfo {
        clsid: CLSID_GLB_PROVIDER,
        extension: ".glb",
    },
    ProviderInfo {
        clsid: CLSID_GLTF_PROVIDER,
        extension: ".gltf",
    },
];

static LOG_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[implement(
    IInitializeWithFile,
    IInitializeWithItem,
    IInitializeWithStream,
    IThumbnailProvider
)]
struct ThumbnailProvider {
    extension_hint: &'static str,
    path: Mutex<Option<PathBuf>>,
}

impl ThumbnailProvider {
    fn new(extension_hint: &'static str) -> Self {
        Self {
            extension_hint,
            path: Mutex::new(None),
        }
    }
}

#[allow(non_snake_case)]
impl IInitializeWithFile_Impl for ThumbnailProvider_Impl {
    fn Initialize(&self, pszfilepath: &PCWSTR, _grfmode: u32) -> Result<()> {
        let path = unsafe { pszfilepath.to_string() }.map_err(|_| Error::from(E_FAIL))?;
        log_line(&format!("v{PROVIDER_VERSION} Initialize {path}"));
        *self.path.lock().map_err(|_| Error::from(E_FAIL))? = Some(PathBuf::from(path));
        Ok(())
    }
}

#[allow(non_snake_case)]
impl IInitializeWithItem_Impl for ThumbnailProvider_Impl {
    fn Initialize(
        &self,
        psi: Option<&windows::Win32::UI::Shell::IShellItem>,
        _grfmode: u32,
    ) -> Result<()> {
        let item = psi.ok_or_else(|| Error::from(E_POINTER))?;
        let display_name = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH)? };
        let path = unsafe { display_name.to_string() }.map_err(|_| Error::from(E_FAIL))?;
        unsafe { CoTaskMemFree(Some(display_name.0 as *const c_void)) };
        log_line(&format!("v{PROVIDER_VERSION} InitializeWithItem {path}"));
        *self.path.lock().map_err(|_| Error::from(E_FAIL))? = Some(PathBuf::from(path));
        Ok(())
    }
}

#[allow(non_snake_case)]
impl IInitializeWithStream_Impl for ThumbnailProvider_Impl {
    fn Initialize(&self, pstream: Option<&IStream>, _grfmode: u32) -> Result<()> {
        let stream = pstream.ok_or_else(|| Error::from(E_POINTER))?;
        let temp_path = write_stream_to_temp_model(stream, self.extension_hint)?;
        log_line(&format!(
            "v{PROVIDER_VERSION} InitializeWithStream {}",
            temp_path.display()
        ));
        *self.path.lock().map_err(|_| Error::from(E_FAIL))? = Some(temp_path);
        Ok(())
    }
}

#[allow(non_snake_case)]
impl IThumbnailProvider_Impl for ThumbnailProvider_Impl {
    fn GetThumbnail(
        &self,
        cx: u32,
        phbmp: *mut HBITMAP,
        pdwalpha: *mut WTS_ALPHATYPE,
    ) -> Result<()> {
        if phbmp.is_null() || pdwalpha.is_null() {
            return Err(Error::from(E_POINTER));
        }

        let path = self
            .path
            .lock()
            .map_err(|_| Error::from(E_FAIL))?
            .clone()
            .ok_or_else(|| Error::from(E_FAIL))?;

        let size = cx.clamp(32, 512);
        match render_thumbnail(
            &path,
            &RenderOptions {
                size,
                ..Default::default()
            },
        ) {
            Ok(bitmap) => unsafe {
                let hbmp = create_hbitmap(&bitmap.pixels, bitmap.width, bitmap.height)?;
                *phbmp = hbmp;
                *pdwalpha = WTSAT_ARGB;
                log_line(&format!(
                    "v{PROVIDER_VERSION} OK {} {}px",
                    path.display(),
                    size
                ));
                Ok(())
            },
            Err(error) => {
                log_line(&format!(
                    "v{PROVIDER_VERSION} ERR {}: {error}",
                    path.display()
                ));
                Err(Error::from(E_FAIL))
            }
        }
    }
}

#[implement(IClassFactory)]
struct ClassFactory {
    provider: ProviderInfo,
}

#[allow(non_snake_case)]
impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Option<&IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut c_void,
    ) -> Result<()> {
        if ppvobject.is_null() {
            return Err(Error::from(E_POINTER));
        }
        unsafe { *ppvobject = null_mut() };

        if punkouter.is_some() {
            return Err(Error::from(CLASS_E_NOAGGREGATION));
        }

        let unknown: IUnknown = ThumbnailProvider::new(self.provider.extension).into();
        let hr = unsafe { unknown.query(riid, ppvobject) };
        if hr.is_ok() {
            Ok(())
        } else {
            Err(Error::from(hr))
        }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}

#[no_mangle]
pub extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        log_line("DllGetClassObject E_POINTER");
        return E_POINTER;
    }
    unsafe { *ppv = null_mut() };

    let Some(provider) = PROVIDERS
        .iter()
        .copied()
        .find(|provider| provider.clsid == unsafe { *rclsid })
    else {
        log_line("DllGetClassObject CLASS_E_CLASSNOTAVAILABLE");
        return CLASS_E_CLASSNOTAVAILABLE;
    };

    log_line(&format!(
        "v{PROVIDER_VERSION} DllGetClassObject OK {}",
        provider.extension
    ));
    let factory: IClassFactory = ClassFactory { provider }.into();
    let hr = unsafe { factory.query(riid, ppv) };
    if hr.is_ok() {
        S_OK
    } else {
        hr
    }
}

#[no_mangle]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

unsafe fn create_hbitmap(pixels: &[u8], width: u32, height: u32) -> Result<HBITMAP> {
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut bits: *mut c_void = null_mut();
    let hbmp = CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0)?;
    if hbmp.0.is_null() || bits.is_null() {
        return Err(Error::from(E_FAIL));
    }

    let out = std::slice::from_raw_parts_mut(bits as *mut u8, (width * height * 4) as usize);
    for (src, dst) in pixels.chunks_exact(4).zip(out.chunks_exact_mut(4)) {
        dst[0] = src[2];
        dst[1] = src[1];
        dst[2] = src[0];
        dst[3] = src[3];
    }

    Ok(hbmp)
}

fn log_line(message: &str) {
    let _guard = LOG_LOCK.lock();
    let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown-time".to_string());

    if let Some(path) = log_path() {
        let _ = create_dir_all(path.parent().unwrap_or_else(|| std::path::Path::new(".")));
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{timestamp} {message}");
        }
    }
}

fn log_path() -> Option<PathBuf> {
    std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .map(|p| p.join("3DThumbnails").join("3dthumbs.log"))
        .or_else(|| {
            directories::ProjectDirs::from("dev", "3DThumbnails", "3DThumbnails")
                .map(|d| d.data_local_dir().join("3dthumbs.log"))
        })
}

fn write_stream_to_temp_model(stream: &IStream, extension_hint: &str) -> Result<PathBuf> {
    let mut stat = unsafe { std::mem::zeroed() };
    unsafe {
        stream.Stat(&mut stat, STATFLAG_NONAME)?;
        stream.Seek(0, STREAM_SEEK_SET, None)?;
    }

    let mut path = std::env::temp_dir();
    path.push("3DThumbnails");
    create_dir_all(&path).map_err(|_| Error::from(E_FAIL))?;
    let stem = format!(
        "stream-{}-{}",
        std::process::id(),
        time::OffsetDateTime::now_utc().unix_timestamp_nanos()
    );
    path.push(format!("{stem}.bin"));

    let mut file = std::fs::File::create(&path).map_err(|_| Error::from(E_FAIL))?;
    let mut remaining = stat.cbSize;
    let mut buffer = vec![0u8; 64 * 1024];
    let mut prefix = Vec::with_capacity(128);

    loop {
        let requested = buffer.len().min(remaining.min(u32::MAX as u64) as usize) as u32;
        if requested == 0 {
            break;
        }

        let mut read = 0u32;
        let hr = unsafe {
            stream.Read(
                buffer.as_mut_ptr() as *mut c_void,
                requested,
                Some(&mut read),
            )
        };
        if hr.is_err() {
            return Err(Error::from(hr));
        }
        if read == 0 {
            break;
        }

        if prefix.len() < 128 {
            let take = (128 - prefix.len()).min(read as usize);
            prefix.extend_from_slice(&buffer[..take]);
        }
        file.write_all(&buffer[..read as usize])
            .map_err(|_| Error::from(E_FAIL))?;
        remaining = remaining.saturating_sub(read as u64);
    }

    let extension = match extension_hint.strip_prefix('.') {
        Some("model") | None => guess_model_extension(&prefix),
        Some(ext) if !ext.is_empty() => ext,
        _ => guess_model_extension(&prefix),
    };
    let final_path = path.with_file_name(format!("{stem}.{extension}"));
    std::fs::rename(&path, &final_path).map_err(|_| Error::from(E_FAIL))?;
    Ok(final_path)
}

fn guess_model_extension(prefix: &[u8]) -> &'static str {
    if prefix.starts_with(b"glTF") {
        return "glb";
    }
    if prefix.starts_with(b"Kaydara FBX Binary") {
        return "fbx";
    }

    let trimmed = prefix
        .iter()
        .copied()
        .skip_while(|b| b.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if trimmed.starts_with(b"{") {
        return "gltf";
    }
    if trimmed.starts_with(b"; FBX") || trimmed.windows(3).any(|w| w == b"FBX") {
        return "fbx";
    }
    if trimmed.starts_with(b"v ")
        || trimmed.starts_with(b"#")
        || trimmed.starts_with(b"o ")
        || trimmed.starts_with(b"mtllib")
    {
        return "obj";
    }

    "glb"
}
