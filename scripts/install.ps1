$ErrorActionPreference = 'Stop'

# Install a published ORust release without Git or a local Rust build.
$Repository = if ($env:ORUST_REPOSITORY) { $env:ORUST_REPOSITORY } else { 'Whales-Group/Orust' }
$Version = if ($env:ORUST_VERSION) { $env:ORUST_VERSION } else { 'latest' }
$OrustHome = if ($env:ORUST_HOME) { $env:ORUST_HOME } else { Join-Path $env:USERPROFILE '.orust' }

function Fail($Message) {
    throw "[orust] error: $Message"
}

if ($Version -eq 'latest') {
    $release = Invoke-RestMethod "https://api.github.com/repos/$Repository/releases/latest"
    $Version = $release.tag_name
}
$Version = $Version.TrimStart('v')
$Tag = "v$Version"
$Archive = "orust-$Version-windows-x86_64.zip"
$BaseUrl = "https://github.com/$Repository/releases/download/$Tag"
$Temp = Join-Path ([System.IO.Path]::GetTempPath()) ("orust-install-" + [Guid]::NewGuid())
$VersionDir = Join-Path $OrustHome "versions\$Version"
$BinDir = Join-Path $OrustHome 'bin'

try {
    New-Item -ItemType Directory -Force -Path $Temp | Out-Null
    Write-Host "[orust] Downloading ORust $Version for Windows x86_64"
    Invoke-WebRequest "$BaseUrl/$Archive" -OutFile (Join-Path $Temp $Archive)
    Invoke-WebRequest "$BaseUrl/SHA256SUMS" -OutFile (Join-Path $Temp 'SHA256SUMS')

    $expected = ((Get-Content (Join-Path $Temp 'SHA256SUMS') | Where-Object { $_ -match [regex]::Escape($Archive) } | Select-Object -First 1) -split '\s+')[0]
    if (-not $expected) { Fail "SHA256SUMS does not contain $Archive" }
    $actual = (Get-FileHash (Join-Path $Temp $Archive) -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected.ToLowerInvariant()) { Fail "checksum verification failed for $Archive" }

    if (Test-Path $VersionDir) { Remove-Item $VersionDir -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $VersionDir, $BinDir | Out-Null
    Expand-Archive (Join-Path $Temp $Archive) -DestinationPath $VersionDir -Force
    if (-not (Test-Path (Join-Path $VersionDir 'orust.exe'))) { Fail 'release archive does not contain orust.exe' }
    if (-not (Test-Path (Join-Path $VersionDir 'orust-lsp.exe'))) { Fail 'release archive does not contain orust-lsp.exe' }
    Copy-Item (Join-Path $VersionDir 'orust.exe') (Join-Path $BinDir 'orust.exe') -Force
    Copy-Item (Join-Path $VersionDir 'orust-lsp.exe') (Join-Path $BinDir 'orust-lsp.exe') -Force

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $entries = @($userPath -split ';' | Where-Object { $_ -and $_ -ne $BinDir })
    [Environment]::SetEnvironmentVariable('Path', (($entries + $BinDir) -join ';'), 'User')
    Write-Host "[orust] Installed orust and orust-lsp to $BinDir"
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host '[orust] Rust/Cargo is not installed. Install it with https://rustup.rs'
    }
    Write-Host '[orust] Open a new PowerShell window, then run: orust --version'
}
finally {
    if (Test-Path $Temp) { Remove-Item $Temp -Recurse -Force }
}
