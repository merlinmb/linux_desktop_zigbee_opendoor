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
    [switch]$SkipBuild,  # just restart the already-installed AppImage
    [switch]$Setup       # one-time: install Node 20, Rust and Tauri system libs on the target, then exit
)

$ErrorActionPreference = 'Stop'
$target = "$User@$HostName"
if (-not $RepoUrl) { $RepoUrl = (git -C (Split-Path $PSScriptRoot -Parent) remote get-url origin).Trim() }

if ($Setup) {
    # Needs a TTY for the sudo password, so the commands go on the ssh command line (not stdin).
    $setupCmd = @(
        'sudo apt-get update'
        'sudo apt-get install -y build-essential curl wget file git libssl-dev libgtk-3-dev libwebkit2gtk-4.0-dev libayatana-appindicator3-dev librsvg2-dev libx11-dev'
        'curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -'
        'sudo apt-get install -y nodejs'
        'curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
    ) -join ' && '
    Write-Host "Installing build prerequisites on $target (will prompt for sudo password) ..." -ForegroundColor Cyan
    ssh -t $target $setupCmd
    if ($LASTEXITCODE -ne 0) { throw "Setup failed (exit $LASTEXITCODE)" }
    Write-Host 'Setup done. Re-run without -Setup to build and deploy.' -ForegroundColor Green
    exit 0
}

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
for c in git node npm cargo; do
  command -v `$c >/dev/null || { echo "Missing '`$c' on target. Run: .\scripts\deploy.ps1 -User $User -Setup" >&2; exit 127; }
done
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
