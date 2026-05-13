# Vais Programming Language — Windows Installer
# Usage: irm https://vais.dev/install.ps1 | iex
#        Invoke-WebRequest -Uri https://vais.dev/install.ps1 -OutFile install.ps1; .\install.ps1
#
# Environment variables:
#   VAIS_VERSION   - Specific version to install (default: latest)
#   VAIS_INSTALL   - Installation directory (default: $HOME\.vais\bin)

$ErrorActionPreference = "Stop"

$Repo = "vaislang/vais"
$InstallDir = if ($env:VAIS_INSTALL) { $env:VAIS_INSTALL } else { "$HOME\.vais\bin" }

function Write-Info($msg)  { Write-Host "  info  " -ForegroundColor Cyan -NoNewline; Write-Host $msg }
function Write-Ok($msg)    { Write-Host "  ok    " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Warn($msg)  { Write-Host "  warn  " -ForegroundColor Yellow -NoNewline; Write-Host $msg }
function Write-Err($msg)   { Write-Host "  error " -ForegroundColor Red -NoNewline; Write-Host $msg; exit 1 }

function Get-Platform {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($arch) {
        "X64"   { return "x86_64-pc-windows-msvc" }
        "Arm64" { return "aarch64-pc-windows-msvc" }
        default { Write-Err "Unsupported architecture: $arch" }
    }
}

function Get-LatestVersion {
    if ($env:VAIS_VERSION) {
        Write-Info "Installing specified version: $env:VAIS_VERSION"
        return $env:VAIS_VERSION
    }

    Write-Info "Fetching latest version..."
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ "User-Agent" = "vais-installer" }
        $version = $release.tag_name
        Write-Info "Latest version: $version"
        return $version
    } catch {
        Write-Err "Failed to fetch latest version. Set `$env:VAIS_VERSION manually or check your network."
    }
}

function Install-Vais {
    Write-Host ""
    Write-Host "  Vais Installer" -ForegroundColor White
    Write-Host ""

    $target = Get-Platform
    Write-Info "Detected platform: Windows ($target)"

    $version = Get-LatestVersion

    $archive = "vais-$version-$target.zip"
    $url = "https://github.com/$Repo/releases/download/$version/$archive"

    Write-Info "Downloading $archive..."

    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "vais-install-$(Get-Random)"
    New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
    $archivePath = Join-Path $tmpDir $archive

    try {
        Invoke-WebRequest -Uri $url -OutFile $archivePath -UseBasicParsing
    } catch {
        Write-Err "Download failed. Check that version $version exists for $target.`n  URL: $url"
    }

    Write-Info "Extracting..."
    $extractDir = Join-Path $tmpDir "extracted"
    Expand-Archive -Path $archivePath -DestinationPath $extractDir -Force

    # Find binary
    $binary = Get-ChildItem -Path $extractDir -Recurse -Filter "vaisc.exe" | Select-Object -First 1
    if (-not $binary) {
        Write-Err "Binary 'vaisc.exe' not found in archive."
    }

    # Create install directory
    Write-Info "Installing to $InstallDir\vaisc.exe..."
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    }
    Copy-Item -Path $binary.FullName -Destination (Join-Path $InstallDir "vaisc.exe") -Force

    # Install std library if present
    $stdDir = Split-Path $binary.FullName -Parent
    $stdSrc = Join-Path $stdDir "std"
    if (Test-Path $stdSrc) {
        $stdDest = Join-Path (Split-Path $InstallDir -Parent) "lib\vais\std"
        Write-Info "Installing standard library to $stdDest..."
        if (-not (Test-Path $stdDest)) {
            New-Item -ItemType Directory -Force -Path $stdDest | Out-Null
        }
        Copy-Item -Path "$stdSrc\*" -Destination $stdDest -Recurse -Force
    }

    # Clean up
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue

    # Add to PATH if not already present
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        Write-Info "Adding $InstallDir to user PATH..."
        [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$userPath", "User")
        $env:Path = "$InstallDir;$env:Path"
        Write-Ok "Added to PATH (restart your terminal to apply)"
    }

    # Verify
    $vaisc = Join-Path $InstallDir "vaisc.exe"
    if (Test-Path $vaisc) {
        try {
            $ver = & $vaisc --version 2>&1
            Write-Ok "Vais installed successfully! ($ver)"
        } catch {
            Write-Ok "Vais installed to $vaisc"
        }
    } else {
        Write-Err "Installation verification failed."
    }

    Write-Host ""
    Write-Host "  Getting started:" -ForegroundColor White
    Write-Host "    PS> echo 'F main() { puts(`"Hello, Vais!`") }' > hello.vais"
    Write-Host "    PS> vaisc run hello.vais"
    Write-Host ""
    Write-Host "    Docs:       https://vais.dev/docs/"
    Write-Host "    Playground:  https://vais.dev/playground/"
    Write-Host "    GitHub:      https://github.com/$Repo"
    Write-Host ""
}

Install-Vais
