# CodeMapper installer — Windows (PowerShell)
# Usage: irm https://raw.githubusercontent.com/lirrensi/codemap/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$REPO = "lirrensi/codemap"
$BINARY = "codemap.exe"

# --- helpers ---
function Info($msg)  { Write-Host "> $msg" -ForegroundColor Blue }
function Ok($msg)    { Write-Host "✓ $msg" -ForegroundColor Green }
function Warn($msg)  { Write-Host "! $msg" -ForegroundColor Yellow }
function Die($msg)   { Write-Host "x $msg" -ForegroundColor Red; exit 1 }

# --- detect arch ---
function Get-Arch {
    $arch = (Get-CimInstance Win32_Processor).Architecture
    switch ($arch) {
        0 { return "x86_64" }  # x86
        5 { return "aarch64" } # ARM
        9 { return "x86_64" }  # x64
        12 { return "aarch64" } # ARM64
        default { return "x86_64" }
    }
}

# --- get latest version from GitHub ---
function Get-LatestVersion {
    $url = "https://api.github.com/repos/$REPO/releases/latest"
    try {
        $response = Invoke-RestMethod -Uri $url -UseBasicParsing
        return $response.tag_name
    }
    catch {
        Die "Could not fetch latest release: $_"
    }
}

# --- main ---
function Main {
    $arch = Get-Arch
    Info "Detecting platform... ${arch}-windows"

    # Get latest version
    Info "Fetching latest release..."
    $version = Get-LatestVersion
    Info "Latest version: $version"

    # Build URL
    $filename = "$BINARY-$arch-windows.zip"
    $url = "https://github.com/$REPO/releases/download/$version/$filename"
    Info "Downloading from: $url"

    # Create temp directory
    $tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) "codemap-$(New-Guid)"
    New-Item -ItemType Directory -Path $tmpdir -Force | Out-Null
    $archive = Join-Path $tmpdir "codemap.zip"

    try {
        # Download
        $ProgressPreference = 'SilentlyContinue'
        Invoke-WebRequest -Uri $url -OutFile $archive -UseBasicParsing
        $ProgressPreference = 'Continue'

        # Extract
        Info "Extracting..."
        Expand-Archive -Path $archive -DestinationPath $tmpdir -Force

        # Find binary
        $binPath = Join-Path $tmpdir $BINARY
        if (-not (Test-Path $binPath)) {
            # Maybe it's nested in a folder
            $found = Get-ChildItem -Path $tmpdir -Filter $BINARY -Recurse | Select-Object -First 1
            if ($found) {
                $binPath = $found.FullName
            } else {
                Die "Binary not found in archive"
            }
        }

        # Choose install directory
        $installDir = $env:CODEMAP_INSTALL_DIR
        if (-not $installDir) {
            # Try user-local bin first (no admin needed)
            $installDir = Join-Path $env:LOCALAPPDATA "codemap\bin"
        }

        # Create install dir if needed
        if (-not (Test-Path $installDir)) {
            New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        }

        # Copy binary
        $destPath = Join-Path $installDir $BINARY
        Copy-Item -Path $binPath -Destination $destPath -Force

        # Add to user PATH if not already there
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$installDir*") {
            Info "Adding $installDir to user PATH..."
            [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
            $env:Path = "$env:Path;$installDir"
            Warn "PATH updated. You may need to restart your terminal."
        }

        # Verify
        Ok "Installed $BINARY to $destPath"

        Write-Host ""
        Info "Quick start:"
        Write-Host "  codemap                    # scan current directory"
        Write-Host "  codemap setup              # add pre-commit hook"
        Write-Host "  codemap --help             # see all options"
    }
    finally {
        # Cleanup
        if (Test-Path $tmpdir) {
            Remove-Item -Path $tmpdir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

Main
