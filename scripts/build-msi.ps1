param(
    [string]$Configuration = "release"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$TargetRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root "target" }
$TargetDir = Join-Path $TargetRoot $Configuration
$WixObj = Join-Path $Root "wix\obj"
$WixBin = Join-Path $Root "wix\bin"
$ProductWxs = Join-Path $Root "wix\Product.wxs"
$OutputMsi = Join-Path $Root "3DThumbnails-1.0.0-x64.msi"
$LocalWix = Join-Path $Root ".tools\wix314"

cargo build -p thumbnail_provider --release

New-Item -ItemType Directory -Force -Path $WixObj, $WixBin | Out-Null

$wix = Get-Command wix.exe -ErrorAction SilentlyContinue
if ($wix) {
    & $wix.Source build $ProductWxs -arch x64 -d "TargetDir=$TargetDir" -d "ProjectDir=$Root" -o $OutputMsi
    exit $LASTEXITCODE
}

$candle = Get-Command candle.exe -ErrorAction SilentlyContinue
$light = Get-Command light.exe -ErrorAction SilentlyContinue
if (-not $candle -and (Test-Path (Join-Path $LocalWix "candle.exe"))) {
    $candle = Get-Item (Join-Path $LocalWix "candle.exe")
}
if (-not $light -and (Test-Path (Join-Path $LocalWix "light.exe"))) {
    $light = Get-Item (Join-Path $LocalWix "light.exe")
}
if ($candle -and $light) {
    & $candle.FullName -arch x64 "-dTargetDir=$TargetDir" "-dProjectDir=$Root" -out (Join-Path $WixObj "Product.wixobj") $ProductWxs
    & $light.FullName -out $OutputMsi (Join-Path $WixObj "Product.wixobj")
    exit $LASTEXITCODE
}

throw "WiX Toolset not found. Install WiX v4 (`winget install WiXToolset.WiXToolset`) or WiX v3, then rerun this script."
