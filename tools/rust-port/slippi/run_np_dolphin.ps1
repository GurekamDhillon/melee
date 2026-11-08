# SPDX-License-Identifier: MIT
# Start one local-test Slippi Dolphin (the patched netplay copy in _build/dolphin-netplay) for the loopback
# Dolphin-vs-Dolphin test (docs/rust-port/notes/slippi-online.md section 9). Headless, Null video, no audio, no window,
# no OS input: pads come from the named pipe \\.\pipe\<Name>slippibot1 (SLIPPI_PIPE_PREFIX). Matchmaking goes to the
# local mock (SLIPPI_MM_URL); SLIPPI_LOCAL_TEST=1 turns off the user API, game reports, STUN and browser pages and binds
# every netplay socket to 127.0.0.1. The disc path comes from GW_ISO_VANILLA in .env and is never printed.
#   powershell -File run_np_dolphin.ps1 -Name A -Out DIR -MmPort 43110 [-TimeoutSec 600] [-Addrs 804d5f90,...]
# Phase 6 (enet_mm.patch build): -Dir D (the Dolphin dir, default _build\dolphin-netplay), -MmEnet HOST:PORT (the
# ENet matchmaking protocol instead of the WebSocket mock), -RealNet (lifts the loopback binding: THE REAL NETWORK;
# web API, reports, STUN and browser stay off), -AppVersion V (create-ticket appVersion).
# Prints the PID on stdout and returns at once; the caller kills only that PID.
param(
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$Out,
    [int]$MmPort = 43110,
    [string]$Dir = '',
    [string]$MmEnet = '',
    [switch]$RealNet,
    [string]$AppVersion = '',
    [string]$Addrs = '804a04f0,804d6640,804d6cf4,80479d60,804c1fac,804a0bc0,81155a8c,81155a90'
)
$ErrorActionPreference = 'Stop'
$ws = if ($env:GW_WORKSPACE) { $env:GW_WORKSPACE } else { 'E:\Projects\Melee Workspace' }
$dol = if ($Dir) { $Dir } else { Join-Path $ws '_build\dolphin-netplay' }
$exe = Join-Path $dol 'dolphin-emu-nogui.exe'
$user = Join-Path $dol "user$Name"
$m = Select-String -Path (Join-Path $ws '.env') -Pattern '^(export )?GW_ISO_VANILLA=(.*)$'
if (-not $m) { Write-Error 'GW_ISO_VANILLA is not in .env'; exit 2 }
$iso = $m.Matches[0].Groups[2].Value.Trim().Trim('"')
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$env:SLIPPI_LOCAL_TEST = '1'
$env:SLIPPI_MM_URL = "ws://127.0.0.1:$MmPort/v1/queue"
if ($MmEnet) { $env:SLIPPI_MM_ENET = $MmEnet }
if ($RealNet) { $env:SLIPPI_REAL_NET = '1' }
if ($AppVersion) { $env:SLIPPI_MM_APP_VERSION = $AppVersion }
$env:SLIPPI_PIPE_PREFIX = $Name
$env:SLIPPI_CHECKSUM_LOG = Join-Path $Out "checksums_$Name.txt"
if (-not $env:NP_NO_STATELOG) { $env:MELEE_DOLPHIN_STATELOG = Join-Path $Out "state_$Name.txt" }
$env:MELEE_DOLPHIN_STATELOG_ADDRS = $Addrs
# one argument string with the paths quoted (Start-Process joins an array without quoting, and a user dir with spaces
# then silently becomes "E:\Projects\Melee")
$dargs = "-u `"$user`" -p headless -v Null -e `"$iso`""
$p = Start-Process -FilePath $exe -ArgumentList $dargs -PassThru -WindowStyle Hidden `
    -RedirectStandardError (Join-Path $Out "stderr_$Name.txt") -RedirectStandardOutput (Join-Path $Out "stdout_$Name.txt")
try { $p.PriorityClass = 'Idle' } catch {}
Write-Output $p.Id
