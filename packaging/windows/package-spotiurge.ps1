param(
    [Parameter(Mandatory)][ValidateScript({ Test-Path $_ -PathType Leaf })][string]$Binary,
    [Parameter(Mandatory)][ValidatePattern('^\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$')][string]$Version,
    [ValidateSet('x86_64', 'aarch64')][string]$Arch = 'x86_64',
    [string]$OutputDir = 'dist',
    [string]$Compiler = 'ISCC.exe'
)
$ErrorActionPreference = 'Stop'
$Binary = (Resolve-Path $Binary).Path
$OutputDir = [IO.Path]::GetFullPath($OutputDir)
New-Item -ItemType Directory -Force $OutputDir | Out-Null
& $Compiler "/DVersion=$Version" "/DArch=$Arch" "/DBinary=$Binary" "/DOutputDir=$OutputDir" (Join-Path $PSScriptRoot 'spotiurge.iss')
if ($LASTEXITCODE -ne 0) { throw "Inno Setup failed: $LASTEXITCODE" }
$installer = Join-Path $OutputDir "Spotiurge-$Version-windows-$Arch-setup.exe"
if (-not (Test-Path $installer -PathType Leaf)) { throw 'The installer was not produced' }
Copy-Item $Binary (Join-Path $OutputDir 'Spotiurge.exe') -Force
Copy-Item (Join-Path $PSScriptRoot 'WINDOWS-INSTALL.txt') $OutputDir -Force
$hashes = @($installer, (Join-Path $OutputDir 'Spotiurge.exe')) | ForEach-Object {
    "{0}  {1}" -f (Get-FileHash $_ -Algorithm SHA256).Hash.ToLowerInvariant(), [IO.Path]::GetFileName($_)
}
[IO.File]::WriteAllLines((Join-Path $OutputDir 'checksums.txt'), $hashes, [Text.UTF8Encoding]::new($false))
Write-Output $installer
