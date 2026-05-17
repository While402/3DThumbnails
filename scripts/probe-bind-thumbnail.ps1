param(
    [Parameter(Mandatory=$true)]
    [string]$Path
)

$ErrorActionPreference = "Stop"
$Resolved = (Resolve-Path $Path).Path

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("43826d1e-e718-42ee-bc55-a1e261c37bfe")]
public interface IShellItem {
    void BindToHandler(
        IntPtr pbc,
        [MarshalAs(UnmanagedType.LPStruct)] Guid bhid,
        [MarshalAs(UnmanagedType.LPStruct)] Guid riid,
        out IntPtr ppv
    );
    void GetParent(out IntPtr ppsi);
    void GetDisplayName(uint sigdnName, out IntPtr ppszName);
    void GetAttributes(uint sfgaoMask, out uint psfgaoAttribs);
    void Compare(IntPtr psi, uint hint, out int piOrder);
}

[ComImport]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
[Guid("e357fccd-a995-4576-b01f-234630154e96")]
public interface IThumbnailProvider {
    void GetThumbnail(uint cx, out IntPtr phbmp, out uint pdwAlpha);
}

public static class BindThumbProbe {
    [DllImport("shell32.dll", CharSet = CharSet.Unicode, PreserveSig = false)]
    public static extern void SHCreateItemFromParsingName(
        string pszPath,
        IntPtr pbc,
        [MarshalAs(UnmanagedType.LPStruct)] Guid riid,
        [MarshalAs(UnmanagedType.Interface)] out IShellItem ppv
    );

    public static int Bind(string path) {
        Guid iidShellItem = new Guid("43826d1e-e718-42ee-bc55-a1e261c37bfe");
        Guid bhidThumbnail = new Guid("7b2e650a-8e20-4f4a-b09e-6597afc72fb0");
        Guid iidThumbnailProvider = new Guid("e357fccd-a995-4576-b01f-234630154e96");
        IShellItem item;
        SHCreateItemFromParsingName(path, IntPtr.Zero, iidShellItem, out item);
        IntPtr providerPtr;
        item.BindToHandler(IntPtr.Zero, bhidThumbnail, iidThumbnailProvider, out providerPtr);
        if (providerPtr == IntPtr.Zero) throw new Exception("BindToHandler returned null provider");
        IThumbnailProvider provider = (IThumbnailProvider)Marshal.GetObjectForIUnknown(providerPtr);
        IntPtr hbmp;
        uint alpha;
        provider.GetThumbnail(256, out hbmp, out alpha);
        if (hbmp != IntPtr.Zero) Marshal.Release(providerPtr);
        return 0;
    }
}
"@

[BindThumbProbe]::Bind($Resolved)
Write-Host "BindToHandler thumbnail OK: $Resolved"

