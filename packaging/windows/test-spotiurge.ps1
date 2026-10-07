param(
    [Parameter(Mandatory)][string]$Installer,
    [Parameter(Mandatory)][string]$Binary,
    [Parameter(Mandatory)][string]$WorkDir
)
$ErrorActionPreference = 'Stop'
$Installer = (Resolve-Path $Installer).Path
$Binary = (Resolve-Path $Binary).Path
$WorkDir = [IO.Path]::GetFullPath($WorkDir)
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{6CA54290-9AB4-468A-8436-76EFF996B96B}_is1'
if ((Test-Path $uninstallKey) -or (Test-Path "$env:LOCALAPPDATA\Programs\Spotiurge")) {
    throw 'An existing Spotiurge installation must not be replaced by a package test'
}
if (Test-Path $WorkDir) { throw 'Use a fresh package-test directory' }
New-Item -ItemType Directory $WorkDir | Out-Null
$installDir = Join-Path $WorkDir 'installed'
$upstreamKeys = @(
    'HKEY_CURRENT_USER\Software\Classes\spotify',
    'HKEY_CURRENT_USER\Software\Classes\Spotifast.spotify',
    'HKEY_CURRENT_USER\Software\Spotifast',
    'HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Uninstall\{FCED1EA0-EBF5-4C32-BA3B-A3AD724BACC3}_is1'
)
function Get-UpstreamRegistration {
    # Values stay in memory. They are never printed or included in the report.
    @($upstreamKeys | ForEach-Object {
        $key = $_
        if (Test-Path "Registry::$key") { (& reg.exe query $key /s 2>$null) -join "`n" }
        else { '<absent>' }
    }) -join "`n"
}
function Invoke-PackageProcess([string]$File, [string[]]$Arguments) {
    $process = Start-Process -FilePath $File -ArgumentList $Arguments -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "Package process failed: $($process.ExitCode)" }
}
$upstreamBefore = Get-UpstreamRegistration
$profile = Join-Path $env:APPDATA 'sergeserbinenko\spotiurge\config'
New-Item -ItemType Directory -Force $profile | Out-Null
$fixture = Join-Path $profile ("package-test-{0}.txt" -f [guid]::NewGuid())
[IO.File]::WriteAllText($fixture, 'preserve this personal profile fixture')
$fixtureHash = (Get-FileHash $fixture -Algorithm SHA256).Hash
$binaryHash = (Get-FileHash $Binary -Algorithm SHA256).Hash
$installed = $false
try {
    foreach ($phase in @('install', 'upgrade')) {
        Invoke-PackageProcess $Installer @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/DIR=`"$installDir`"", "/LOG=`"$WorkDir\$phase.log`"")
        $installed = $true
        $exe = Join-Path $installDir 'Spotiurge.exe'
        if ((Get-FileHash $exe -Algorithm SHA256).Hash -ne $binaryHash) { throw 'Installed executable differs from its payload' }
        if ((Get-Item $exe).VersionInfo.ProductName -ne 'Spotiurge') { throw 'Windows executable has the wrong product identity' }
        if ((Get-ItemProperty $uninstallKey).DisplayName -notlike 'Spotiurge*') { throw 'Wrong installer identity' }
        if (-not (Test-Path "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Spotiurge.lnk")) { throw 'Missing Start menu shortcut' }
        $command = (Get-Item 'HKCU:\Software\Classes\Spotiurge.spotify\shell\open\command').GetValue('')
        if ($command -ne "`"$exe`" `"%1`"") { throw 'Wrong independent link handler' }
        if ((Get-UpstreamRegistration) -cne $upstreamBefore) { throw 'Upstream registration changed' }
        if ((Get-FileHash $fixture -Algorithm SHA256).Hash -ne $fixtureHash) { throw 'Profile fixture changed' }
        Invoke-PackageProcess $exe @('--version')
    }
    Invoke-PackageProcess (Join-Path $installDir 'unins000.exe') @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/LOG=`"$WorkDir\uninstall.log`"")
    $installed = $false
    foreach ($key in @($uninstallKey, 'HKCU:\Software\Classes\Spotiurge.spotify', 'HKCU:\Software\Spotiurge\Capabilities')) {
        if (Test-Path $key) { throw "Uninstall retained an owned key: $key" }
    }
    $registered = Get-Item 'HKCU:\Software\RegisteredApplications' -ErrorAction SilentlyContinue
    if ($registered -and ($null -ne $registered.GetValue('Spotiurge'))) { throw 'Uninstall retained its app registration' }
    if (Test-Path "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Spotiurge.lnk") { throw 'Uninstall retained its shortcut' }
    if (Test-Path (Join-Path $installDir 'Spotiurge.exe')) { throw 'Uninstall retained the executable' }
    if ((Get-UpstreamRegistration) -cne $upstreamBefore) { throw 'Uninstall changed upstream registration' }
    if ((Get-FileHash $fixture -Algorithm SHA256).Hash -ne $fixtureHash) { throw 'Uninstall changed the profile fixture' }
    $report = [ordered]@{
        install = 'passed'; upgrade = 'passed'; uninstall = 'passed'
        productName = 'Spotiurge'; profilePreserved = $true; upstreamRegistrationPreserved = $true
        installerSha256 = (Get-FileHash $Installer -Algorithm SHA256).Hash.ToLowerInvariant()
        executableSha256 = $binaryHash.ToLowerInvariant()
    }
    $report | ConvertTo-Json | Set-Content (Join-Path $WorkDir 'verification.json')
    $report | ConvertTo-Json
} finally {
    if ($installed -and (Test-Path (Join-Path $installDir 'unins000.exe'))) {
        Invoke-PackageProcess (Join-Path $installDir 'unins000.exe') @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART')
    }
    Remove-Item $fixture -ErrorAction SilentlyContinue
}
