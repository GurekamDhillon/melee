# SPDX-License-Identifier: MIT
# Dolphin oracle: play one .slp in the patched Slippi Dolphin (headless, Null video, no audio, no window, no input)
# and write the NHV1 memory records. See docs/rust-port/notes/dolphin-oracle.md.
#   powershell -File run_dolphin.ps1 -Slp FILE.slp -Out DIR [-Wide] [-TimeoutSec 900]
# Writes DIR\dolphin.nhraw, DIR\dolphin.stderr.txt. The disc comes from GW_ISO_VANILLA in the workspace .env and is never printed.
# stdout/stderr MUST be redirected: dolphin-emu-nogui is a GUI-subsystem exe, and its first fprintf to an invalid
# stdio handle fast-fails the process (0xC0000409) right after "Starting core".
param(
    [Parameter(Mandatory = $true)][string]$Slp,
    [Parameter(Mandatory = $true)][string]$Out,
    [switch]$Wide,
    [int]$TimeoutSec = 900
)
$ErrorActionPreference = 'Stop'
$ws = if ($env:GW_WORKSPACE) { $env:GW_WORKSPACE } else { 'E:\Projects\Melee Workspace' }
$dol = if ($env:DOLPHIN_ORACLE_DIR) { $env:DOLPHIN_ORACLE_DIR } else { Join-Path $ws '_build\dolphin-oracle' }
$exe = Join-Path $dol 'src\Binary\x64\dolphin-emu-nogui.exe'
$user = Join-Path $dol 'user'
# DOLPHIN_USER_DIR: a private copy of that user directory (made on first use), for many concurrent runs (covtest)
if ($env:DOLPHIN_USER_DIR) {
    if (-not (Test-Path (Join-Path $env:DOLPHIN_USER_DIR 'Config'))) { Copy-Item -Recurse -Force $user $env:DOLPHIN_USER_DIR }
    $user = $env:DOLPHIN_USER_DIR
}
if (-not (Test-Path $exe)) { Write-Error "dolphin oracle not built: $exe"; exit 2 }
$m = Select-String -Path (Join-Path $ws '.env') -Pattern '^(export )?GW_ISO_VANILLA=(.*)$'
if (-not $m) { Write-Error 'GW_ISO_VANILLA is not in .env'; exit 2 }
$iso = $m.Matches[0].Groups[2].Value.Trim().Trim('"')
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$Slp = (Resolve-Path $Slp).Path
$comm = Join-Path $Out 'comm.json'
# shouldResync=false: Slippi's RestoreGameFrame.asm otherwise writes the recorded x/y/facing/percent and the RNG seed into
# the fighters every frame (PDB_SHOULD_RESYNC), which would make the console state partly a copy of the file.
# DOLPHIN_RESYNC=1 restores Slippi's default.
$resync = ($env:DOLPHIN_RESYNC -eq '1')
@{ mode = 'normal'; replay = $Slp; isRealTimeMode = $false; shouldResync = $resync; commandId = [guid]::NewGuid().ToString() } | ConvertTo-Json | Set-Content -Encoding ascii $comm
$raw = Join-Path $Out 'dolphin.nhraw'
Remove-Item -ErrorAction SilentlyContinue $raw
$env:MELEE_DOLPHIN_NHV1 = $raw
$env:MELEE_DOLPHIN_NHV1_EXIT = '1'
$env:MELEE_DOLPHIN_NHV1_WIDE = if ($Wide) { '1' } else { '0' }
# the recording console's widescreen code (camera bounds, bubble region: the logic's off-screen test), not in Slippi's
# playback set; DOLPHIN_NO_REPLAY_CODES=1 plays without it (notes/dolphin-oracle.md, "Widescreen codes")
if ($env:DOLPHIN_NO_REPLAY_CODES -eq '1') { Remove-Item -ErrorAction SilentlyContinue (Join-Path $user 'GameSettings\GALE01.ini') }
else { python -I (Join-Path $PSScriptRoot 'replay_codes.py') $Slp $user }
# a Slippi 1.x replay has no Gecko list: play it with the 1.7.1 console's codes instead of playback's built-in legacy
# list (legacy171_codes.py; needs an oracle exe with MELEE_DOLPHIN_LEGACY_CODES, _build/dolphin-oracle-slp171).
# DOLPHIN_LEGACY171=0 keeps the built-in list; DOLPHIN_LEGACY171_ARGS passes --no-spawns / --qol.
$legacy = Join-Path $Out 'legacy171.bin'
Remove-Item -ErrorAction SilentlyContinue $legacy
Remove-Item -ErrorAction SilentlyContinue Env:MELEE_DOLPHIN_LEGACY_CODES
if ($env:DOLPHIN_LEGACY171 -ne '0') {
    $la = @(); if ($env:DOLPHIN_LEGACY171_ARGS) { $la = $env:DOLPHIN_LEGACY171_ARGS -split ' ' }
    python -I (Join-Path $PSScriptRoot 'legacy171_codes.py') $Slp $legacy @la
    if (Test-Path $legacy) { $env:MELEE_DOLPHIN_LEGACY_CODES = $legacy }
}
$dargs = @('-u', "`"$user`"", '-p', 'headless', '-v', 'Null', '-i', "`"$comm`"", '-e', "`"$iso`"")
$p = Start-Process -FilePath $exe -ArgumentList $dargs -PassThru -WindowStyle Hidden `
    -RedirectStandardError (Join-Path $Out 'dolphin.stderr.txt') -RedirectStandardOutput (Join-Path $Out 'dolphin.stdout.txt')
try { $p.PriorityClass = $(if ($env:DOLPHIN_PRIORITY) { $env:DOLPHIN_PRIORITY } else { 'Idle' }) } catch {}
if (-not $p.WaitForExit($TimeoutSec * 1000)) {
    $p.Kill()
    Write-Output "dolphin: timeout after $TimeoutSec s"
    exit 1
}
$p.WaitForExit()
$size = if (Test-Path $raw) { (Get-Item $raw).Length } else { 0 }
Write-Output "dolphin: exit $($p.ExitCode), $size bytes of records"
if ($size -le 12) { exit 1 }
exit 0
