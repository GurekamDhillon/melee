# Copy this runner wherever convenient; pass the integrated harness directory.
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$Exe,
    [Parameter(Mandatory=$true)][string]$Iso,
    [string]$Harness = $PSScriptRoot,
    [string]$Runtime,
    [string]$Out = 'sweep-all-results',
    [switch]$Subset,
    [switch]$Plan,
    [string]$CatalogLog,
    [int]$Frames = 180,
    [int]$TimeoutSeconds = 75
)
$ErrorActionPreference = 'Stop'
$runnerArgs = @('-B', (Join-Path $Harness 'sweep_all.py'), '--harness', $Harness,
    '--exe', $Exe, '--iso', $Iso, '--out', $Out, '--frames', $Frames, '--timeout', $TimeoutSeconds)
if ($Runtime) { $runnerArgs += @('--runtime', $Runtime) }
if ($Subset) { $runnerArgs += '--subset' }
if ($Plan) { $runnerArgs += '--plan' }
if ($CatalogLog) { $runnerArgs += @('--catalog-log', $CatalogLog) }
& python @runnerArgs
exit $LASTEXITCODE
