; Instalador por usuario de Switchman (sin administrador).
; Compilar: ISCC /DAppVersion=0.4.0 /DArch=x64 /DSourceDir=<carpeta publicada> installer\switchman.iss

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef Arch
  #define Arch "x64"
#endif
#ifndef SourceDir
  #error Falta /DSourceDir con la carpeta publicada (Switchman.exe y switchman-cli.exe)
#endif

[Setup]
AppId={{6F1D2C4B-8E3A-4B7D-9C2E-5A1F0B3D7E91}
AppName=Switchman
AppVersion={#AppVersion}
AppVerName=Switchman {#AppVersion}
AppPublisher=Switchman
AppPublisherURL=https://github.com/hector-horta/switchman
AppSupportURL=https://github.com/hector-horta/switchman/issues
VersionInfoVersion={#AppVersion}
DefaultDirName={userpf}\Switchman
DisableDirPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
MinVersion=10.0.17763
ArchitecturesAllowed={#Arch == "arm64" ? "arm64" : "x64compatible"}
ArchitecturesInstallIn64BitMode={#Arch == "arm64" ? "arm64" : "x64compatible"}
OutputDir=..\target\dist
OutputBaseFilename=Switchman-{#AppVersion}-win-{#Arch}-setup
SetupIconFile=..\assets\switchman.ico
UninstallDisplayIcon={app}\Switchman.exe
UninstallDisplayName=Switchman
WizardStyle=modern
Compression=lzma2/ultra64
SolidCompression=yes
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[CustomMessages]
en.ShortcutComment=Routes links and removes tracking parameters
es.ShortcutComment=Enruta enlaces y quita parámetros de rastreo
en.OpenSettings=Open Windows Settings to make Switchman the default browser
es.OpenSettings=Abrir la Configuración de Windows para elegir Switchman como navegador predeterminado
en.LaunchApp=Open Switchman to choose your browsers and rules
es.LaunchApp=Abrir Switchman para elegir tus navegadores y reglas

[InstallDelete]
; Restos de la instalación por script (install.ps1).
Type: files; Name: "{app}\install.ps1"

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{userprograms}\Switchman"; Filename: "{app}\Switchman.exe"; WorkingDir: "{app}"; Comment: "{cm:ShortcutComment}"

[Run]
Filename: "{app}\switchman-cli.exe"; Parameters: "register --quiet"; Flags: runhidden waituntilterminated
Filename: "ms-settings:defaultapps?registeredAppUser=Switchman"; Description: "{cm:OpenSettings}"; Flags: postinstall shellexec nowait skipifsilent
Filename: "{app}\Switchman.exe"; Description: "{cm:LaunchApp}"; Flags: postinstall nowait skipifsilent

[UninstallRun]
Filename: "{app}\switchman-cli.exe"; Parameters: "unregister"; Flags: runhidden waituntilterminated; RunOnceId: "Unregister"
