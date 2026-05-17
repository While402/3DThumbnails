Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class AssocProbe {
    [DllImport("shlwapi.dll", CharSet = CharSet.Unicode)]
    static extern uint AssocQueryStringW(
        uint flags,
        uint str,
        string pszAssoc,
        string pszExtra,
        StringBuilder pszOut,
        ref uint pcchOut
    );

    public static string Query(string assoc, string extra) {
        uint chars = 0;
        uint hr = AssocQueryStringW(0, 16, assoc, extra, null, ref chars);
        StringBuilder sb = new StringBuilder((int)chars);
        hr = AssocQueryStringW(0, 16, assoc, extra, sb, ref chars);
        return "hr=0x" + hr.ToString("X8") + " value=" + sb.ToString();
    }
}
"@

$iidThumbnailProvider = "{e357fccd-a995-4576-b01f-234630154e96}"
$iidExtractImage = "{bb2e617c-0920-11d1-9a0b-00c04fc2d6c1}"

foreach ($assoc in @(".obj", ".fbx", ".glb", ".gltf")) {
    Write-Host "$assoc IThumbnailProvider => $([AssocProbe]::Query($assoc, $iidThumbnailProvider))"
    Write-Host "$assoc IExtractImage      => $([AssocProbe]::Query($assoc, $iidExtractImage))"
}

