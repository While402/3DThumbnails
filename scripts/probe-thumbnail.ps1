param(
    [Parameter(Mandatory=$true)]
    [string]$Path,
    [string]$OutPng = ""
)

$ErrorActionPreference = "Stop"
$Resolved = (Resolve-Path $Path).Path

Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Sequential)]
public struct SIZE {
    public int cx;
    public int cy;
}

[Flags]
public enum SIIGBF : uint {
    RESIZETOFIT = 0x00000000,
    BIGGERSIZEOK = 0x00000001,
    MEMORYONLY = 0x00000002,
    ICONONLY = 0x00000004,
    THUMBNAILONLY = 0x00000008,
    INCACHEONLY = 0x00000010
}

[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("bcc18b79-ba16-442f-80c4-8a59c30c463b")]
public interface IShellItemImageFactory {
    void GetImage(SIZE size, SIIGBF flags, out IntPtr phbm);
}

public static class ShellThumbProbe {
    [DllImport("shell32.dll", CharSet = CharSet.Unicode, PreserveSig = false)]
    public static extern void SHCreateItemFromParsingName(
        string pszPath,
        IntPtr pbc,
        [MarshalAs(UnmanagedType.LPStruct)] Guid riid,
        [MarshalAs(UnmanagedType.Interface)] out IShellItemImageFactory ppv
    );

    [DllImport("gdi32.dll")]
    public static extern bool DeleteObject(IntPtr hObject);

    public static IntPtr GetThumbnail(string path, int size) {
        Guid iid = new Guid("bcc18b79-ba16-442f-80c4-8a59c30c463b");
        IShellItemImageFactory factory;
        SHCreateItemFromParsingName(path, IntPtr.Zero, iid, out factory);
        SIZE requested = new SIZE { cx = size, cy = size };
        IntPtr hbmp;
        factory.GetImage(requested, SIIGBF.BIGGERSIZEOK, out hbmp);
        return hbmp;
    }
}
"@

$hbmp = [ShellThumbProbe]::GetThumbnail($Resolved, 256)
if ($hbmp -eq [IntPtr]::Zero) {
    throw "Shell returned null bitmap"
}

try {
    if ($OutPng) {
        $bitmap = [System.Drawing.Image]::FromHbitmap($hbmp)
        try {
            $bitmap.Save((Join-Path (Get-Location) $OutPng), [System.Drawing.Imaging.ImageFormat]::Png)
        } finally {
            $bitmap.Dispose()
        }
    }
    Write-Host "Shell thumbnail OK: $Resolved"
} finally {
    [ShellThumbProbe]::DeleteObject($hbmp) | Out-Null
}
