<#
.SYNOPSIS
  Update Open Door Monitor on a Linux Mint host: pull, build AppImage, install, restart.

.EXAMPLE
  .\scripts\deploy.ps1 -User merlin
  .\scripts\deploy.ps1 -User merlin -RepoPath '~/src/opendoor' -SkipBuild
#>
param(
    [Parameter(Mandatory = $true)][string]$User,
    [string]$HostName = 'flight.local',
    [string]$RepoPath = '~/linux_desktop_zigbee_opendoor',
    [string]$InstallPath = '~/opendoor-monitor.AppImage',
    [string]$Branch = 'main',
    [string]$RepoUrl,     # defaults to this repo's origin; used to clone if RepoPath is missing
    [switch]$SkipBuild   # just restart the already-installed AppImage
)

$ErrorActionPreference = 'Stop'
$target = "$User@$HostName"
if (-not $RepoUrl) { $RepoUrl = (git -C (Split-Path $PSScriptRoot -Parent) remote get-url origin).Trim() }

# The remote builds from git, so make sure what you want deployed is pushed.
Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    if (git status --porcelain) { Write-Warning 'Uncommitted local changes will NOT be deployed.' }
    $unpushed = git log "origin/$Branch..HEAD" --oneline 2>$null
    if ($unpushed) {
        Write-Warning "Unpushed commits (remote will not see them):`n$unpushed"
        if ((Read-Host 'Continue anyway? [y/N]') -ne 'y') { exit 1 }
    }
} finally { Pop-Location }

$build = if ($SkipBuild) { '' } else { @"
if [ ! -d $RepoPath/.git ]; then
  echo "Cloning $RepoUrl -> $RepoPath"
  git clone $RepoUrl $RepoPath
fi
cd $RepoPath
git fetch origin
git checkout $Branch
git pull --ff-only origin $Branch
npm install
npm run tauri:build
APPIMAGE=`$(ls -t src-tauri/target/release/bundle/appimage/*.AppImage | head -n1)
echo "Installing `$APPIMAGE -> $InstallPath"
cp -f "`$APPIMAGE" $InstallPath
chmod +x $InstallPath
"@ }

$remote = @"
set -e
[ -f ~/.cargo/env ] && . ~/.cargo/env
[ -s ~/.nvm/nvm.sh ] && . ~/.nvm/nvm.sh
$build
pkill -f opendoor-monitor || true
sleep 1
DISPLAY=:0 setsid nohup $InstallPath >/tmp/opendoor-monitor.log 2>&1 < /dev/null &
echo "Started. Log: /tmp/opendoor-monitor.log"
"@

Write-Host "Deploying to $target ..." -ForegroundColor Cyan
# Strip CR so bash doesn't choke on Windows line endings
($remote -replace "`r", '') | ssh $target 'bash -l -s'
if ($LASTEXITCODE -ne 0) { throw "Deploy failed (exit $LASTEXITCODE)" }
Write-Host 'Done.' -ForegroundColor Green
