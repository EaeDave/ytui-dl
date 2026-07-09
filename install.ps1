# ytui-dl installer for Windows (PowerShell 5+)
#
#   irm https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.ps1 | iex
#
# Installs to: %LOCALAPPDATA%\ytui-dl\bin\ytui-dl.exe
# Optionally installs yt-dlp + ffmpeg via winget (asks Y/n).
#
# Tip: if the window closes too fast, run:
#   powershell -NoExit -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.ps1 | iex"

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"  # faster, avoids flaky IWR progress UI

$Repo = "EaeDave/ytui-dl"
$BinName = "ytui-dl.exe"
$Asset = "ytui-dl-x86_64-pc-windows-msvc.exe"
$InstallDir = Join-Path $env:LOCALAPPDATA "ytui-dl\bin"
$InstallPath = Join-Path $InstallDir $BinName

function Write-Info([string]$msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-Warn([string]$msg) { Write-Host "!!  $msg" -ForegroundColor Yellow }
function Pause-Exit([int]$code = 1) {
    Write-Host ""
    Write-Host "Press Enter to close..." -ForegroundColor DarkGray
    try {
        # Works when interactive; no-ops if stdin is closed
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    } catch {
        try { [void](Read-Host) } catch { Start-Sleep -Seconds 12 }
    }
    exit $code
}
function Die([string]$msg) {
    Write-Host "error: $msg" -ForegroundColor Red
    Pause-Exit 1
}

function Get-LatestTag {
    try {
        $rel = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" `
            -Headers @{ "User-Agent" = "ytui-dl-install" }
        if ($rel.tag_name) { return [string]$rel.tag_name }
    } catch {
        Write-Warn "GitHub API failed: $($_.Exception.Message)"
    }

    # Fallback: curl follows redirects and prints final URL
    if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
        $final = & curl.exe -fsSLI -o NUL -w "%{url_effective}" -A "ytui-dl-install" `
            "https://github.com/$Repo/releases/latest" 2>$null
        if ($LASTEXITCODE -eq 0 -and $final) {
            return ($final.Trim() -split "/")[-1]
        }
    }
    Die "could not resolve latest release tag"
}

function Version-Gt([string]$a, [string]$b) {
    $a = $a.TrimStart("v")
    $b = $b.TrimStart("v")
    try {
        return ([version]$a) -gt ([version]$b)
    } catch {
        return $a -ne $b
    }
}

function Ensure-UserPath([string]$dir) {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $userPath) { $userPath = "" }
    $parts = $userPath -split ";" | Where-Object { $_ -ne "" }
    if ($parts -contains $dir) { return }
    $newPath = if ($userPath.TrimEnd(";")) { "$userPath;$dir" } else { $dir }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path = "$dir;$env:Path"
    Write-Warn "Added $dir to your user PATH (open a new terminal if needed)"
}

function Download-File([string]$Url, [string]$OutFile) {
    $dir = Split-Path -Parent $OutFile
    if ($dir -and -not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir | Out-Null
    }
    if (Test-Path $OutFile) { Remove-Item -Force $OutFile }

    # Prefer curl.exe (ships with Windows 10+) — more reliable than Invoke-WebRequest for large files
    if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
        Write-Info "using curl.exe"
        & curl.exe -fL --retry 4 --retry-all-errors --connect-timeout 15 --max-time 600 `
            -A "ytui-dl-install" -o $OutFile $Url
        if ($LASTEXITCODE -ne 0) {
            throw "curl.exe failed with exit code $LASTEXITCODE"
        }
    } else {
        Write-Info "using System.Net.WebClient"
        $wc = New-Object System.Net.WebClient
        $wc.Headers.Add("User-Agent", "ytui-dl-install")
        try {
            $wc.DownloadFile($Url, $OutFile)
        } finally {
            $wc.Dispose()
        }
    }

    if (-not (Test-Path $OutFile) -or (Get-Item $OutFile).Length -lt 1024) {
        throw "download incomplete or too small: $OutFile"
    }
}

function Test-Cmd([string]$Name) {
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Ensure-RuntimeDeps {
    $needYt = -not (Test-Cmd "yt-dlp")
    $needFf = -not (Test-Cmd "ffmpeg")

    if (-not $needYt -and -not $needFf) {
        Write-Info "yt-dlp and ffmpeg already on PATH"
        return
    }

    if (-not (Test-Cmd "winget")) {
        Write-Warn "winget not found — install yt-dlp and ffmpeg manually, then re-open the terminal"
        if ($needYt) { Write-Warn "  missing: yt-dlp" }
        if ($needFf) { Write-Warn "  missing: ffmpeg" }
        return
    }

    Write-Host ""
    Write-Host "Runtime dependencies:" -ForegroundColor Magenta
    if ($needYt) { Write-Host "  - yt-dlp  (required to download)" -ForegroundColor Yellow }
    else { Write-Host "  - yt-dlp  (found)" -ForegroundColor Green }
    if ($needFf) { Write-Host "  - ffmpeg  (recommended for merge / WhatsApp profile)" -ForegroundColor Yellow }
    else { Write-Host "  - ffmpeg  (found)" -ForegroundColor Green }
    Write-Host ""

    $ans = Read-Host "Install missing deps with winget now? [Y/n]"
    if ($ans -match '^[nN]') {
        Write-Warn "skipped winget installs — ytui-dl needs yt-dlp on PATH to work"
        return
    }

    if ($needYt) {
        Write-Info "winget install yt-dlp.yt-dlp …"
        try {
            winget install -e --id yt-dlp.yt-dlp --accept-package-agreements --accept-source-agreements
        } catch {
            Write-Warn "yt-dlp install failed: $($_.Exception.Message)"
        }
    }
    if ($needFf) {
        Write-Info "winget install Gyan.FFmpeg …"
        try {
            winget install -e --id Gyan.FFmpeg --accept-package-agreements --accept-source-agreements
        } catch {
            Write-Warn "ffmpeg install failed: $($_.Exception.Message)"
        }
    }

    # Refresh PATH from Machine + User for this session
    $machine = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $user = [Environment]::GetEnvironmentVariable("Path", "User")
    $env:Path = "$machine;$user"

    if (Test-Cmd "yt-dlp") { Write-Info "yt-dlp OK" } else { Write-Warn "yt-dlp still not on PATH — open a new terminal" }
    if (Test-Cmd "ffmpeg") { Write-Info "ffmpeg OK" } else { Write-Warn "ffmpeg still not on PATH — open a new terminal" }
}

# --- args (when script is saved and run as .\install.ps1) ---
$Force = $false
$Uninstall = $false
$Check = $false
$SkipDeps = $false
foreach ($arg in $args) {
    switch -Regex ($arg) {
        "^--force$|^-f$" { $Force = $true }
        "^--uninstall$" { $Uninstall = $true }
        "^--check$" { $Check = $true }
        "^--skip-deps$" { $SkipDeps = $true }
        "^--help$|^-h$" {
            Write-Host @"
ytui-dl Windows installer

  irm https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.ps1 | iex

Or download install.ps1 and run:
  .\install.ps1
  .\install.ps1 --force
  .\install.ps1 --uninstall
  .\install.ps1 --check
  .\install.ps1 --skip-deps
"@
            exit 0
        }
    }
}

try {
    if ($Uninstall) {
        if (Test-Path $InstallPath) {
            Remove-Item -Force $InstallPath
            Write-Info "removed $InstallPath"
            Write-Info "config/downloads were not removed"
        } else {
            Die "not installed at $InstallPath"
        }
        Pause-Exit 0
    }

    Write-Info "resolving latest release…"
    $tag = Get-LatestTag
    if (-not $tag) { Die "empty release tag" }
    $remote = $tag.TrimStart("v")
    Write-Info "latest release: $remote"

    $localVer = $null
    if (Test-Path $InstallPath) {
        try {
            $out = & $InstallPath --version 2>$null
            if ("$out" -match "([\d.]+)\s*$") { $localVer = $Matches[1] }
        } catch {}
    }

    if ($Check) {
        Write-Host "installed:  $(if ($localVer) { $localVer } else { '(not found)' })"
        Write-Host "remote:     $remote"
        Pause-Exit 0
    }

    if ($localVer -and -not $Force) {
        if (-not (Version-Gt $remote $localVer)) {
            Write-Info "already up to date ($localVer)"
            if (-not $SkipDeps) { Ensure-RuntimeDeps }
            Pause-Exit 0
        }
        Write-Info "updating $localVer → $remote"
    } else {
        Write-Info "installing $remote"
    }

    $downloadUrl = "https://github.com/$Repo/releases/download/$tag/$Asset"
    $tmp = Join-Path $env:TEMP "ytui-dl-install-$PID.exe"
    Write-Info "downloading $Asset"
    Write-Info $downloadUrl
    try {
        Download-File -Url $downloadUrl -OutFile $tmp
        $size = (Get-Item $tmp).Length
        Write-Info ("download complete ({0:N0} bytes)" -f $size)
    } catch {
        Die "download failed: $downloadUrl — $($_.Exception.Message)"
    }

    $sumUrl = "$downloadUrl.sha256"
    try {
        $sumTmp = Join-Path $env:TEMP "ytui-dl-install-$PID.sha256"
        Download-File -Url $sumUrl -OutFile $sumTmp
        $sumText = Get-Content -Raw $sumTmp
        Remove-Item -Force $sumTmp -ErrorAction SilentlyContinue
        $expected = ($sumText -split "\s+")[0].ToLower()
        $actual = (Get-FileHash -Algorithm SHA256 $tmp).Hash.ToLower()
        if ($expected -ne $actual) {
            Die "SHA256 mismatch (expected $expected, got $actual)"
        }
        Write-Info "checksum OK"
    } catch {
        Write-Warn "could not verify checksum (continuing): $($_.Exception.Message)"
    }

    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir | Out-Null
    }

    $old = "$InstallPath.old"
    if (Test-Path $InstallPath) {
        try {
            if (Test-Path $old) { Remove-Item -Force $old }
            Rename-Item -Force $InstallPath $old
        } catch {
            Write-Warn "could not rename existing binary (close ytui-dl if running): $($_.Exception.Message)"
        }
    }

    Copy-Item -Force $tmp $InstallPath
    Remove-Item -Force $tmp -ErrorAction SilentlyContinue
    if (Test-Path $old) {
        Remove-Item -Force $old -ErrorAction SilentlyContinue
    }

    Ensure-UserPath $InstallDir

    Write-Info "installed: $InstallPath"
    try {
        $v = & $InstallPath --version
        Write-Info "$v"
    } catch {
        Write-Warn "could not run --version: $($_.Exception.Message)"
    }

    if (-not $SkipDeps) {
        Ensure-RuntimeDeps
    }

    Write-Host ""
    Write-Info "done. Open a new terminal and run: ytui-dl"
    Write-Host "  (Windows Terminal recommended)" -ForegroundColor DarkGray
    Pause-Exit 0
} catch {
    Die $_.Exception.Message
}
