# ytd installer for Windows (PowerShell 5+)
#
#   irm https://raw.githubusercontent.com/EaeDave/ytd/main/install.ps1 | iex
#
# Installs to: %LOCALAPPDATA%\ytd\bin\ytd.exe
# Adds that folder to the *current session* PATH and the user PATH permanently.
# Optionally installs yt-dlp + ffmpeg via winget (asks Y/n).

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$Repo = "EaeDave/ytd"
$BinName = "ytd.exe"
$Asset = "ytd-x86_64-pc-windows-msvc.exe"
$InstallDir = Join-Path $env:LOCALAPPDATA "ytd\bin"
$InstallPath = Join-Path $InstallDir $BinName
$LegacyDir = Join-Path $env:LOCALAPPDATA "ytui-dl\bin"
$LegacyPath = Join-Path $LegacyDir "ytui-dl.exe"

function Write-Info([string]$msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-Warn([string]$msg) { Write-Host "!!  $msg" -ForegroundColor Yellow }

function Pause-Soft {
    Write-Host ""
    Write-Host "Press Enter to continue..." -ForegroundColor DarkGray
    try { [void](Read-Host) } catch { Start-Sleep -Seconds 8 }
}

function Die([string]$msg) {
    Write-Host "error: $msg" -ForegroundColor Red
    Pause-Soft
    # Prefer return over exit so parent shells (and -NoExit) stay usable
    throw $msg
}

function Get-LatestTag {
    try {
        $rel = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" `
            -Headers @{ "User-Agent" = "ytd-install" }
        if ($rel.tag_name) { return [string]$rel.tag_name }
    } catch {
        Write-Warn "GitHub API failed: $($_.Exception.Message)"
    }

    if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
        $final = & curl.exe -fsSLI -o NUL -w "%{url_effective}" -A "ytd-install" `
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
    # 1) Permanent user PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $userPath) { $userPath = "" }
    $parts = $userPath -split ";" | Where-Object { $_ -ne "" }
    if ($parts -notcontains $dir) {
        $newPath = if ($userPath.TrimEnd(";")) { "$userPath;$dir" } else { $dir }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Warn "Added $dir to your user PATH (new terminals will see ytd)"
    }

    # 2) Current session PATH (critical — parent shells won't get this unless we set it here)
    $sessionParts = $env:Path -split ";" | Where-Object { $_ -ne "" }
    if ($sessionParts -notcontains $dir) {
        $env:Path = "$dir;$env:Path"
    }
}

function Download-File([string]$Url, [string]$OutFile, [int]$MinBytes = 64) {
    $dir = Split-Path -Parent $OutFile
    if ($dir -and -not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir | Out-Null
    }
    if (Test-Path $OutFile) { Remove-Item -Force $OutFile }

    if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
        Write-Info "using curl.exe"
        & curl.exe -fL --retry 4 --retry-all-errors --connect-timeout 15 --max-time 600 `
            -A "ytd-install" -o $OutFile $Url
        if ($LASTEXITCODE -ne 0) {
            throw "curl.exe failed with exit code $LASTEXITCODE"
        }
    } else {
        Write-Info "using System.Net.WebClient"
        $wc = New-Object System.Net.WebClient
        $wc.Headers.Add("User-Agent", "ytd-install")
        try {
            $wc.DownloadFile($Url, $OutFile)
        } finally {
            $wc.Dispose()
        }
    }

    if (-not (Test-Path $OutFile)) {
        throw "download missing: $OutFile"
    }
    $len = (Get-Item $OutFile).Length
    if ($len -lt $MinBytes) {
        throw "download incomplete or too small ($len bytes, need >= $MinBytes): $OutFile"
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
        Write-Warn "winget not found — install yt-dlp and ffmpeg manually"
        return
    }

    Write-Host ""
    Write-Host "Runtime dependencies:" -ForegroundColor Magenta
    if ($needYt) { Write-Host "  - yt-dlp  (required)" -ForegroundColor Yellow }
    else { Write-Host "  - yt-dlp  (found)" -ForegroundColor Green }
    if ($needFf) { Write-Host "  - ffmpeg  (recommended)" -ForegroundColor Yellow }
    else { Write-Host "  - ffmpeg  (found)" -ForegroundColor Green }
    Write-Host ""

    # When piped (irm | iex), Read-Host often still works in an interactive console
    $ans = "Y"
    try {
        $ans = Read-Host "Install missing deps with winget now? [Y/n]"
    } catch {
        $ans = "Y"
    }
    if ($ans -match '^[nN]') {
        Write-Warn "skipped winget installs"
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

    $machine = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $user = [Environment]::GetEnvironmentVariable("Path", "User")
    $env:Path = "$machine;$user"
}

# --- args ---
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
ytd Windows installer

  irm https://raw.githubusercontent.com/EaeDave/ytd/main/install.ps1 | iex

  .\install.ps1 --force | --uninstall | --check | --skip-deps
"@
            return
        }
    }
}

try {
    if ($Uninstall) {
        $removed = $false
        foreach ($p in @($InstallPath, $LegacyPath, (Join-Path $LegacyDir "ytd.exe"))) {
            if (Test-Path $p) {
                Remove-Item -Force $p
                Write-Info "removed $p"
                $removed = $true
            }
        }
        if (-not $removed) {
            Die "not installed at $InstallPath"
        }
        Write-Info "config/downloads were not removed"
        return
    }

    Write-Info "resolving latest release…"
    $tag = Get-LatestTag
    if (-not $tag) { Die "empty release tag" }
    $remote = $tag.TrimStart("v")
    Write-Info "latest release: $remote"

    $localVer = $null
    foreach ($probe in @($InstallPath, (Join-Path $LegacyDir "ytd.exe"), $LegacyPath)) {
        if (-not (Test-Path $probe)) { continue }
        try {
            $out = & $probe --version 2>$null
            if ("$out" -match "([\d.]+)\s*$") {
                $localVer = $Matches[1]
                break
            }
        } catch {}
    }

    if ($Check) {
        Write-Host "installed:  $(if ($localVer) { $localVer } else { '(not found)' })"
        Write-Host "remote:     $remote"
        return
    }

    if ($localVer -and -not $Force) {
        if (-not (Version-Gt $remote $localVer)) {
            Write-Info "already up to date ($localVer)"
            Ensure-UserPath $InstallDir
            if (-not $SkipDeps) { Ensure-RuntimeDeps }
            Write-Host ""
            Write-Info "run: ytd   or   & `"$InstallPath`""
            return
        }
        Write-Info "updating $localVer → $remote"
    } else {
        Write-Info "installing $remote"
    }

    $downloadUrl = "https://github.com/$Repo/releases/download/$tag/$Asset"
    $tmp = Join-Path $env:TEMP "ytd-install-$PID.exe"
    Write-Info "downloading $Asset"
    Write-Info $downloadUrl
    try {
        Download-File -Url $downloadUrl -OutFile $tmp -MinBytes 100000
        Write-Info ("download complete ({0:N0} bytes)" -f (Get-Item $tmp).Length)
    } catch {
        Die "download failed: $downloadUrl — $($_.Exception.Message)"
    }

    $sumUrl = "$downloadUrl.sha256"
    try {
        $sumTmp = Join-Path $env:TEMP "ytd-install-$PID.sha256"
        # checksum files are tiny (~100 bytes)
        Download-File -Url $sumUrl -OutFile $sumTmp -MinBytes 32
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
            Write-Warn "could not rename existing binary (close ytd if running): $($_.Exception.Message)"
        }
    }
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir | Out-Null
    }
    Copy-Item -Force $tmp $InstallPath
    Remove-Item -Force $tmp -ErrorAction SilentlyContinue
    if (Test-Path $old) {
        Remove-Item -Force $old -ErrorAction SilentlyContinue
    }
    # Drop old ytui-dl install if present
    foreach ($p in @($LegacyPath, (Join-Path $LegacyDir "ytd.exe"))) {
        if (Test-Path $p) {
            try { Remove-Item -Force $p; Write-Info "removed old binary $p" } catch {}
        }
    }

    Ensure-UserPath $InstallDir

    Write-Info "installed: $InstallPath"
    try {
        $v = & $InstallPath --version 2>&1 | Out-String
        $v = $v.Trim()
        if ($LASTEXITCODE -eq -1073741515 -or $LASTEXITCODE -eq 0xC0000135) {
            # STATUS_DLL_NOT_FOUND — classic missing VC++ runtime on dynamic MSVC builds.
            Write-Warn "binary failed with STATUS_DLL_NOT_FOUND (missing Visual C++ runtime DLL)"
            Write-Warn "installing Microsoft Visual C++ Redistributable via winget…"
            if (Test-Cmd "winget") {
                try {
                    winget install -e --id Microsoft.VCRedist.2015+.x64 `
                        --accept-package-agreements --accept-source-agreements
                } catch {
                    Write-Warn "winget VC++ install failed: $($_.Exception.Message)"
                }
                $v = & $InstallPath --version 2>&1 | Out-String
                $v = $v.Trim()
            } else {
                Write-Warn "install VC++ Redistributable from:"
                Write-Warn "  https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist"
            }
        }
        if ($v) { Write-Info $v }
        elseif ($LASTEXITCODE -ne 0) {
            Write-Warn "could not run --version (exit $LASTEXITCODE)"
        }
    } catch {
        Write-Warn "could not run --version: $($_.Exception.Message)"
    }

    if (-not $SkipDeps) {
        Ensure-RuntimeDeps
    }

    Write-Host ""
    Write-Host "================================================" -ForegroundColor Green
    Write-Info "done!"
    Write-Host "  Command:    ytd" -ForegroundColor White
    Write-Host "  Full path:  & `"$InstallPath`"" -ForegroundColor White
    Write-Host "  Or reopen Windows Terminal and run:  ytd" -ForegroundColor White
    Write-Host "  If nothing appears:  ytd --doctor" -ForegroundColor Yellow
    Write-Host "  Log file:  $env:LOCALAPPDATA\ytd\last-run.log" -ForegroundColor DarkGray
    Write-Host "================================================" -ForegroundColor Green
    Write-Host ""
    # Try invoking immediately in this session
    try {
        Write-Info "smoke test in this session:"
        & $InstallPath --version
        Write-Info "doctor (TTY self-check):"
        & $InstallPath --doctor
    } catch {
        Write-Warn "smoke test failed: $($_.Exception.Message)"
    }
} catch {
    Write-Host "error: $($_.Exception.Message)" -ForegroundColor Red
    Pause-Soft
}
