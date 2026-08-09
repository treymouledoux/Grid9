#define MyAppName "Grid9"
#define MyAppVersion "2023-003"
#define MyAppPublisher "treymouledoux"
#define MyAppURL "https://github.com/treymouledoux/Grid9"
#define MyAppExeName "Grid9.exe"
#define MyAppAssocName MyAppName + " Script"
#define MyAppAssocExt ".g9"
#define MyAppAssocKey StringChange(MyAppAssocName, " ", "") + MyAppAssocExt

[Setup]
AppId={{0C1D1885-945C-4EBD-9F2D-F9ECCCBEFD9D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
MinVersion=10.0.18362
ChangesAssociations=yes
DisableProgramGroupPage=yes
LicenseFile=LICENSE.txt
OutputBaseFilename=Grid9 Setup {#MyAppVersion}
ChangesEnvironment=true

Compression=lzma
SolidCompression=yes

WizardStyle=modern
SetupIconFile=Grid9\innosetup\icon.ico
WizardImageFile=Grid9\innosetup\installer_banner.bmp
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "envPath"; Description: "Add to PATH variables"; GroupDescription: "Registry:"; Components: baseinstall
Name: "startmenuicon"; Description: "Create a startmenu shortcut"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; Components: baseinstall
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; Components: baseinstall

[Types]
Name: "custom"; Description: "Custom installation"; Flags: iscustom

[Components]
Name: "baseinstall"; Description: "Includes necessary base files."; Flags: exclusive
Name: "baseinstall\documentation"; Description: "Html documentation about the language. (recomended)"
Name: "baseinstall\examples"; Description: "Example grid9 scripts. (recomended)"
Name: "baseinstall\dllfix"; Description: "Pcre dlls for if Nim is not installed. (recomended)"
Name: "baseinstall\bleachbit"; Description: "Install bleachbit cleaner for Grid9."
Name: "baseinstall\converter"; Description: "Installs the grid9 converter."
Name: "baseinstall\retrogadget"; Description: "Install retro gadget port for Grid9."
Name: "componentRepair"; Description: "Repair examples and/or install examples and legacy examples."; Flags: exclusive

[Dirs]
Name: "C:\ProgramData\Grid9"

[Files]
Source: "Grid9\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion; Components: baseinstall
Source: "Grid9\innosetup\icon.ico"; DestDir: "{app}"; Flags: ignoreversion; Components: baseinstall
Source: "Grid9\innosetup\iconBeta.ico"; DestDir: "{app}"; Flags: ignoreversion; Components: baseinstall
Source: "Grid9\Grid9Converter.exe"; DestDir: "{app}"; Flags: ignoreversion; Components: baseinstall\converter
Source: "Grid9\pcre.dll"; DestDir: "{app}"; Flags: ignoreversion; Components: baseinstall\dllfix
Source: "Grid9\pcre64.dll"; DestDir: "{app}"; Flags: ignoreversion; Components: baseinstall\dllfix
Source: "Grid9\pcre32.dll"; DestDir: "{app}"; Flags: ignoreversion; Components: baseinstall\dllfix
Source: "Grid9\documentation\*"; DestDir: "C:\ProgramData\Grid9"; Flags: ignoreversion recursesubdirs createallsubdirs; Components: baseinstall\documentation
Source: "Grid9\examples\*"; DestDir: "C:\ProgramData\Grid9"; Flags: ignoreversion recursesubdirs createallsubdirs; Components: baseinstall\examples componentRepair
Source: "Grid9\bleachbit\*"; DestDir: "C:\Program Files (x86)\BleachBit\share\cleaners"; Flags: ignoreversion recursesubdirs createallsubdirs; Components: baseinstall\bleachbit
Source: "Grid9\retrogadget\*"; DestDir: "{userdocs}\My Games\Retro\Gadgets"; Flags: ignoreversion recursesubdirs createallsubdirs; Components: baseinstall\retrogadget componentRepair

[Registry]
Root: HKA; Subkey: "Software\Classes\{#MyAppAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#MyAppAssocKey}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\{#MyAppAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\{#MyAppAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"
Root: HKA; Subkey: "Software\Classes\{#MyAppAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}\SupportedTypes"; ValueType: string; ValueName: ".g9"; ValueData: ""

[Icons]
Name: "{autoprograms}\Grid9\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: startmenuicon
Name: "{autoprograms}\Grid9\Grid9 Converter"; Filename: "{app}\Grid9Converter.exe"; Tasks: startmenuicon
Name: "{autodesktop}\Grid9\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Code]
const EnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';

procedure EnvAddPath(Path: string);
var
    Paths: string;
begin
    { Retrieve current path (use empty string if entry not exists) }
    if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths)
    then Paths := '';

    { Skip if string already found in path }
    if Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';') > 0 then exit;

    { App string to the end of the path variable }
    Paths := Paths + ';'+ Path +';'

    { Overwrite (or create if missing) path environment variable }
    if RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths)
    then Log(Format('The [%s] added to PATH: [%s]', [Path, Paths]))
    else Log(Format('Error while adding the [%s] to PATH: [%s]', [Path, Paths]));
end;

procedure EnvRemovePath(Path: string);
var
    Paths: string;
    P: Integer;
begin
    { Skip if registry entry not exists }
    if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths) then
        exit;

    { Skip if string not found in path }
    P := Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';');
    if P = 0 then exit;

    { Update path variable }
    Delete(Paths, P - 1, Length(Path) + 1);

    { Overwrite path environment variable }
    if RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths)
    then Log(Format('The [%s] removed from PATH: [%s]', [Path, Paths]))
    else Log(Format('Error while removing the [%s] from PATH: [%s]', [Path, Paths]));
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
    if (CurStep = ssPostInstall) and WizardIsTaskSelected('envPath') 
     then EnvAddPath(ExpandConstant('{app}'));
end;

procedure CurUninstallStepChanged (CurUninstallStep: TUninstallStep);
 var
     mres : integer;
 begin
    case CurUninstallStep of                   
      usPostUninstall:
        begin
          EnvRemovePath(ExpandConstant('{app}'));

          mres := MsgBox('Do you want to Remove logs and cached code?', mbConfirmation, MB_YESNO or MB_DEFBUTTON2)
          if mres = IDYES then
            DelTree(ExpandConstant('C:\ProgramData\Grid9'), True, True, True);
       end;
   end;
end; 
