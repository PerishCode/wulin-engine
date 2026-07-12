param(
    [string] $Destination
)

$ErrorActionPreference = "Stop"
$PackageVersion = "1.619.4"
$PackageHash = "6a275381027ed758714eedf1ccaeea446b1d9afeddc1f6b6bbc3c85939ef9ffd02b7fae780cd50da635b66b09f5fce99535788551cd64e3663b9e59fe6f7d9de"
$PackageName = "microsoft.direct3d.d3d12.$PackageVersion.nupkg"
$PackageUrl = "https://api.nuget.org/v3-flatcontainer/microsoft.direct3d.d3d12/$PackageVersion/$PackageName"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path

if (-not $Destination) {
    $Destination = Join-Path $RepoRoot "out\deps\agility-sdk\$PackageVersion"
}

$CoreDll = Join-Path $Destination "build\native\bin\x64\D3D12Core.dll"
$LayersDll = Join-Path $Destination "build\native\bin\x64\d3d12SDKLayers.dll"
if ((Test-Path -LiteralPath $CoreDll) -and (Test-Path -LiteralPath $LayersDll)) {
    Write-Host "Agility SDK $PackageVersion is ready at $Destination"
    exit 0
}

if (Test-Path -LiteralPath $Destination) {
    throw "Incomplete Agility SDK directory exists at $Destination. Remove it explicitly before retrying."
}

$DownloadDirectory = Join-Path $RepoRoot "out\downloads"
$Archive = Join-Path $DownloadDirectory $PackageName
New-Item -ItemType Directory -Force -Path $DownloadDirectory | Out-Null

if (-not (Test-Path -LiteralPath $Archive)) {
    Invoke-WebRequest -Uri $PackageUrl -OutFile $Archive
}

$ActualHash = (Get-FileHash -Algorithm SHA512 -LiteralPath $Archive).Hash.ToLowerInvariant()
if ($ActualHash -ne $PackageHash) {
    throw "Agility SDK package hash mismatch. Expected $PackageHash, got $ActualHash."
}

New-Item -ItemType Directory -Path $Destination | Out-Null
tar -xf $Archive -C $Destination
if (($LASTEXITCODE -ne 0) -or -not (Test-Path -LiteralPath $CoreDll) -or -not (Test-Path -LiteralPath $LayersDll)) {
    throw "Agility SDK extraction did not produce the required x64 runtime DLLs."
}

Write-Host "Agility SDK $PackageVersion is ready at $Destination"
