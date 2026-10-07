; Spotiurge's independent per-user development installer. Compile through
; package-spotiurge.ps1 after building the locked Windows release binary.
#ifndef Version
  #error Version is required
#endif
#ifndef Arch
  #error Arch is required (x86_64 or aarch64)
#endif
#ifndef Binary
  #error Binary is required
#endif
#ifndef OutputDir
  #error OutputDir is required
#endif
#if Arch == "aarch64"
  #define InnoArch "arm64"
#else
  #define InnoArch "x64compatible"
#endif
#define AppName "Spotiurge"
#define AppExeName "Spotiurge.exe"

[Setup]
; Stable fork identity, distinct from Spotifast's installer GUID.
AppId={{6CA54290-9AB4-468A-8436-76EFF996B96B}
AppName={#AppName}
AppVersion={#Version}
AppVerName={#AppName} {#Version}
AppPublisher=Serge Serbinenko
AppCopyright=Copyright (c) 2026 Carmine Paolino; Spotiurge by Serge Serbinenko
AppPublisherURL=https://github.com/SergeSerb2/Spotiurge
AppSupportURL=https://github.com/SergeSerb2/Spotiurge/issues
DefaultDirName={localappdata}\Programs\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed={#InnoArch}
ArchitecturesInstallIn64BitMode={#InnoArch}
MinVersion=10.0
LicenseFile=..\..\LICENSE
OutputDir={#OutputDir}
OutputBaseFilename=Spotiurge-{#Version}-windows-{#Arch}-setup
SetupIconFile=spotiurge.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
UninstallDisplayIcon={app}\{#AppExeName}
#define Dash Pos("-", Version)
#if Dash > 0
  #define NumericVersion Copy(Version, 1, Dash - 1)
#else
  #define NumericVersion Version
#endif
VersionInfoVersion={#NumericVersion}.0

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "{#Binary}"; DestDir: "{app}"; DestName: "{#AppExeName}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "WINDOWS-INSTALL.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Registry]
; Register a separate choice in Default apps. Do not take over spotify: or
; write Spotifast's keys. Windows controls the user's explicit default choice.
Root: HKCU; Subkey: "Software\Classes\Spotiurge.spotify"; ValueType: string; ValueName: ""; ValueData: "URL:Spotify link"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Spotiurge.spotify"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\Spotiurge.spotify\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExeName}"",0"
Root: HKCU; Subkey: "Software\Classes\Spotiurge.spotify\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExeName}"" ""%1"""
Root: HKCU; Subkey: "Software\Spotiurge\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "{#AppName}"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Spotiurge\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Personal music discovery and playback"
Root: HKCU; Subkey: "Software\Spotiurge\Capabilities\URLAssociations"; ValueType: string; ValueName: "spotify"; ValueData: "Spotiurge.spotify"
Root: HKCU; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "Spotiurge"; ValueData: "Software\Spotiurge\Capabilities"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#AppExeName}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent
