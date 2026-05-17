Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

public static class ComInterfaceProbe2 {
    [DllImport("ole32.dll")]
    static extern int CoInitializeEx(IntPtr pvReserved, uint dwCoInit);

    [DllImport("ole32.dll")]
    static extern void CoUninitialize();

    [DllImport("ole32.dll")]
    static extern int CoCreateInstance(ref Guid rclsid, IntPtr pUnkOuter, uint dwClsContext, ref Guid riid, out IntPtr ppv);

    public static int Probe(string clsidText, string iidText) {
        CoInitializeEx(IntPtr.Zero, 2);
        Guid clsid = new Guid(clsidText);
        Guid iid = new Guid(iidText);
        IntPtr obj;
        int hr = CoCreateInstance(ref clsid, IntPtr.Zero, 1, ref iid, out obj);
        if (obj != IntPtr.Zero) Marshal.Release(obj);
        CoUninitialize();
        return hr;
    }
}
"@

$clsids = [ordered]@{
    Legacy = "4C6F2B8A-5D2E-4C64-9AC7-B6FD046A8241"
    Obj = "0EF2C8D1-7B70-48C9-B7B8-0F45D3D00001"
    Fbx = "0EF2C8D1-7B70-48C9-B7B8-0F45D3D00002"
    Glb = "0EF2C8D1-7B70-48C9-B7B8-0F45D3D00003"
    Gltf = "0EF2C8D1-7B70-48C9-B7B8-0F45D3D00004"
}

$iids = [ordered]@{
    IUnknown = "00000000-0000-0000-C000-000000000046"
    IThumbnailProvider = "e357fccd-a995-4576-b01f-234630154e96"
    IInitializeWithStream = "b824b49d-22ac-4161-ac8a-9916e8fa3f7f"
    IInitializeWithItem = "7f73be3f-fb79-493c-a6c7-7ee14e245841"
    IInitializeWithFile = "b7d14566-0509-4cce-a71f-0a554233bd9b"
}

foreach ($clsid in $clsids.GetEnumerator()) {
    foreach ($entry in $iids.GetEnumerator()) {
        $hr = [ComInterfaceProbe2]::Probe($clsid.Value, $entry.Value)
        Write-Host "$($clsid.Key) $($entry.Key) => 0x$($hr.ToString('X8'))"
    }
}

