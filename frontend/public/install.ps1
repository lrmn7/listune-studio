# Installs Listune Studio, the latest release, on Windows. In PowerShell:
#
#   irm https://studio.listune.app/install.ps1 | iex
#
# Options (environment variables, set before running):
#   $env:LISTUNE_VERSION = '0.0.2'   install this version instead of the latest
#   $env:LISTUNE_NO_LAUNCH = '1'     don't open Listune when done
#
# It installs for the current user (no admin needed), with Start menu and desktop shortcuts, and
# fetches WebView2 if Windows doesn't have it. Installed copies update themselves.

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue' # Invoke-WebRequest is many times slower with the progress bar.

function Install-Listune {
  $repo = 'Lrmn7/listune-studio'
  $base = if ($env:LISTUNE_DOWNLOAD_BASE) { $env:LISTUNE_DOWNLOAD_BASE } else { "https://github.com/$repo/releases" }
  $app = 'Listune Studio'

  # Windows PowerShell 5.1 may still default to TLS 1.0, which GitHub refuses.
  try { [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12 } catch {}

  if (-not [Environment]::Is64BitOperatingSystem) { throw 'Listune needs 64-bit Windows.' }
  if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64' -or $env:PROCESSOR_ARCHITEW6432 -eq 'ARM64') {
    Write-Host 'This is an ARM PC: Listune runs as an x64 app through Windows emulation.'
  }

  $version = "$env:LISTUNE_VERSION".Trim().TrimStart('v')
  if (-not $version) {
    try { $latest = Invoke-RestMethod -UseBasicParsing -Uri "$base/latest/download/latest.json" }
    catch { throw "Couldn't find the latest release at $base. $_" }
    $version = "$($latest.version)".TrimStart('v')
    if (-not $version) { throw 'The latest release has no version.' }
  }

  # GitHub swaps the space in "Listune Studio" for a dot in asset names.
  $file = "Listune.Studio_${version}_x64-setup.exe"
  $tmp = Join-Path ([IO.Path]::GetTempPath()) ("listune-" + [Guid]::NewGuid().ToString('N'))
  New-Item -ItemType Directory -Path $tmp | Out-Null
  try {
    $setup = Join-Path $tmp $file
    Write-Host "Downloading $app $version for Windows..."
    try { Invoke-WebRequest -UseBasicParsing -Uri "$base/download/v$version/$file" -OutFile $setup }
    catch { throw "Couldn't download $file. $_" }

    Write-Host 'Installing...'
    # /S: the installer's silent mode; it installs for this user and closes a running Listune first.
    $p = Start-Process -FilePath $setup -ArgumentList '/S' -Wait -PassThru
    if ($p.ExitCode -ne 0) { throw "The installer stopped with code $($p.ExitCode)." }
  }
  finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
  }

  $dir = Join-Path $env:LOCALAPPDATA $app
  $exe = Get-ChildItem -Path $dir -Filter '*.exe' -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notmatch '^uninstall' } | Select-Object -First 1
  Write-Host "Installed $app $version. Open it from the Start menu."
  if ($exe -and -not $env:LISTUNE_NO_LAUNCH) { Start-Process -FilePath $exe.FullName }
}

try { Install-Listune }
catch {
  Write-Host "Listune install failed: $_" -ForegroundColor Red
  # A function, not exit: `irm ... | iex` runs in the user's own PowerShell window.
}
