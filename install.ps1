# ytui-dl installer for Windows (PowerShell)
#
#   irm https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.ps1 | iex
#   irm ... | iex -Force
#   & { irm ... } | iex   # if execution policy is strict, prefer:
#   powershell -ExecutionPolicy Bypass -Command "irm ... | iex"
#
# Installs to: %LOCALAPPDATA%\ytui-dl\bin\ytui-dl.exe
# Adds that folder to the user PATH when missing.

$ErrorActionPreference = "Stop"
$Repo = "EaeDave/ytui-dl"
$BinName = "ytui-dl.exe"
$Asset = "ytui-dl-x86_64-pc-windows-msvc.exe"
$InstallDir = Join-Path $env:LOCALAPPDATA "ytui-dl\bin"
$InstallPath = Join-Path $InstallDir $BinName

function Write-Info($msg) { Write-Host "==> $msg" }
function Write-Warn($msg) { Write-Host "!!  $msg" -ForegroundColor Yellow }
function Die($msg) { Write-Host "error: $msg" -ForegroundColor Red; exit 1 }

function Get-LatestTag {
    try {
        $rel = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ "User-Agent" = "ytui-dl-install" }
        if ($rel.tag_name) { return $rel.tag_name }
    } catch {}
    # Fallback: follow GitHub redirect
    $url = "https://github.com/$Repo/releases/latest"
    $req = [System.Net.HttpWebRequest]::Create($url)
    $req.AllowAutoRedirect = $false
    $req.UserAgent = "ytui-dl-install"
    try {
        $resp = $req.GetResponse()
    } catch [System.Net.WebException] {
        $resp = $_.Exception.Response
    }
    if ($resp -and $resp.Headers["Location"]) {
        return ($resp.Headers["Location"] -split "/")[-1]
    }
    Die "could not resolve latest release tag"
}

function Version-Gt([string]$a, [string]$b) {
    $a = $a.TrimStart("v")
    $b = $b.TrimStart("v")
    try {
        return ([version]$a) -gt ([version]$b)
    } catch {
        return $a -ne $b -and ($a -gt $b)
    }
}

function Ensure-UserPath([string]$dir) {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $userPath) { $userPath = "" }
    $parts = $userPath -split ";" | Where-Object { $_ -ne "" }
    if ($parts -contains $dir) {
        return
    }
    $newPath = if ($userPath.TrimEnd(";")) { "$userPath;$dir" } else { $dir }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path = "$dir;$env:Path"
    Write-Warn "Added $dir to your user PATH (open a new terminal if ytui-dl is not found)"
}

$Force = $false
$Uninstall = $false
$Check = $false
foreach ($arg in $args) {
    switch -Regex ($arg) {
        "^--force$|^-f$" { $Force = $true }
        "^--uninstall$" { $Uninstall = $true }
        "^--check$" { $Check = $true }
        "^--help$|^-h$" {
            Write-Host @"
ytui-dl Windows installer

Usage:
  irm https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.ps1 | iex
  irm ... | iex  (with args, save script and run:)
    .\install.ps1
    .\install.ps1 -Force
    .\install.ps1 --uninstall
    .\install.ps1 --check
"@
            exit 0
        }
    }
}

if ($Uninstall) {
    if (Test-Path $InstallPath) {
        Remove-Item -Force $InstallPath
        Write-Info "removed $InstallPath"
        Write-Info "config/downloads were not removed"
    } else {
        Die "not installed at $InstallPath"
    }
    exit 0
}

Write-Info "resolving latest release…"
try {
    $tag = Get-LatestTag
} catch {
    Die "could not resolve latest release: $_"
}
if (-not $tag) { Die "empty release tag" }
$remote = $tag.TrimStart("v")
Write-Info "latest release: $remote"

$localVer = $null
if (Test-Path $InstallPath) {
    try {
        $out = & $InstallPath --version 2>$null
        if ($out -match "([\d.]+)\s*$") { $localVer = $Matches[1] }
    } catch {}
}

if ($Check) {
    Write-Host "installed:  $(if ($localVer) { $localVer } else { '(not found)' })"
    Write-Host "remote:     $remote"
    exit 0
}

if ($localVer -and -not $Force) {
    if (-not (Version-Gt $remote $localVer)) {
        Write-Info "already up to date ($localVer)"
        exit 0
    }
    Write-Info "updating $localVer → $remote"
} else {
    Write-Info "installing $remote"
}

$downloadUrl = "https://github.com/$Repo/releases/download/$tag/$Asset"
$tmp = Join-Path $env:TEMP "ytui-dl-install-$PID.exe"
Write-Info "downloading $Asset"
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tmp -UseBasicParsing
} catch {
    Die "download failed: $downloadUrl — $_"
}

$sumUrl = "$downloadUrl.sha256"
try {
    $sumText = (Invoke-WebRequest -Uri $sumUrl -UseBasicParsing).Content
    $expected = ($sumText -split "\s+")[0].ToLower()
    $actual = (Get-FileHash -Algorithm SHA256 $tmp).Hash.ToLower()
    if ($expected -ne $actual) {
        Die "SHA256 mismatch (expected $expected, got $actual)"
    }
    Write-Info "checksum OK"
} catch {
    Write-Warn "could not verify checksum (continuing): $_"
}

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

# Replace running binary if needed
$old = "$InstallPath.old"
if (Test-Path $InstallPath) {
    try {
        if (Test-Path $old) { Remove-Item -Force $old }
        Rename-Item -Force $InstallPath $old
    } catch {
        Write-Warn "could not rename existing binary (close ytui-dl if running): $_"
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
} catch {}
Write-Info "runtime deps: install yt-dlp and ffmpeg (winget install yt-dlp ffmpeg)"
Write-Info "then run: ytui-dl"
