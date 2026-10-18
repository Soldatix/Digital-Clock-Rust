; RC14 FIX2 internal: optional Screen Saver menu integration; public release HOLD.
; Do not compile until the FIX2 helper binary is staged in payload\Tools.
#define HelperSHA GetSHA256OfFile("payload\Tools\DigitalClock-Windows-ScreenSaver-Setup-x64.exe")
#define BridgeSHA GetSHA256OfFile("payload\AppsAndGames-DigitalClock-Windows.scr")
; RC14 CORE REVIEW CANDIDATE - no public release permitted.
[Setup]
AppId={{2FD8A176-9C95-4435-AEF6-7F3BB57C5C8D}
AppName=Digital Clock
AppVersion=3.0.0
AppVerName=Digital Clock 3.0.0
VersionInfoVersion=3.0.0.0
MinVersion=10.0
AppPublisher=Apps & Games
AppPublisherURL=https://appsandgames.org/digital-clock
AppSupportURL=https://appsandgames.org/digital-clock
AppUpdatesURL=https://appsandgames.org/digital-clock
DefaultDirName={localappdata}\Programs\AppsAndGames\DigitalClockWindows
DefaultGroupName=Apps & Games\Digital Clock Windows
UsePreviousAppDir=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
; EN remains the default on every run; user may choose another language.
LanguageDetectionMethod=none
ShowLanguageDialog=yes
UsePreviousLanguage=no
SetupIconFile=payload\DigitalClock.ico
DisableProgramGroupPage=yes
InfoBeforeFile=payload\MIGRATION-READ-ME-FIRST.txt
OutputDir=output
OutputBaseFilename=DigitalClock-Windows-Installer-RC14-UX-FIX22-MULTILANG-REVIEW-x64
UninstallDisplayIcon={app}\DigitalClock.exe
Compression=lzma2
SolidCompression=yes
CloseApplications=yes
RestartApplications=no
[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "hr"; MessagesFile: "compiler:Default.isl,lang\Croatian.isl"
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "it"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[CustomMessages]
en.TaskActivate=Activate Digital Clock Screen Saver after installation
en.TaskAutoStart=Start Digital Clock when signing in to Windows
en.TaskKeepSaver=Keep Digital Clock in the Windows Screen Saver list (requires Administrator approval; separate system component)
en.LaunchDigitalClock=Launch Digital Clock
en.HelperMissing=Optional System Screen Saver installer is missing. The app was installed, but no system integration was attempted.
en.HelperHashMismatch=Screen Saver helper hash mismatch. No administrator request was made.
en.HelperCancelled=System Screen Saver integration was cancelled or failed. The per-user application remains installed; Windows settings were not changed.
en.SystemSaverMissing=The helper reported success but the System32 Screen Saver is missing. Integration must be reviewed.
en.SystemSaverHashMismatch=System32 Screen Saver file does not match this release. No Screen Saver was activated.
en.RegisteredSaver=Digital Clock is registered in the Windows Screen Saver list. Your active Screen Saver setting was not changed.
en.UninstallHelperFirst=The optional Digital Clock Windows Screen Saver (System Integration) is still registered. Uninstall that separate component first, then uninstall Digital Clock. No files were removed.
en.SelectOtherSaverFirst=Please select (None) or another Screen Saver in Windows settings before uninstalling Digital Clock. Your settings will not be changed automatically.
hr.TaskActivate=Aktiviraj Digital Clock čuvar zaslona nakon instalacije
hr.TaskAutoStart=Pokreni Digital Clock pri prijavi u Windows
hr.TaskKeepSaver=Zadrži Digital Clock na Windows popisu čuvara zaslona (potrebna potvrda administratora; zasebna sistemska komponenta)
hr.LaunchDigitalClock=Pokreni Digital Clock
hr.HelperMissing=Nedostaje instalacijski program za dodatni sistemski čuvar zaslona. Aplikacija je instalirana, ali integracija nije pokrenuta.
hr.HelperHashMismatch=SHA-256 provjera pomoćnog programa nije uspjela. Nije zatražena administratorska potvrda.
hr.HelperCancelled=Integracija sistemskog čuvara zaslona otkazana je ili nije uspjela. Korisnička aplikacija ostaje instalirana; postavke Windowsa nisu promijenjene.
hr.SystemSaverMissing=Pomoćni program javio je uspjeh, ali u System32 nema čuvara zaslona. Potrebna je dodatna provjera.
hr.SystemSaverHashMismatch=Datoteka čuvara zaslona u System32 ne odgovara ovom izdanju. Čuvar zaslona nije aktiviran.
hr.RegisteredSaver=Digital Clock registriran je na Windows popisu čuvara zaslona. Aktivna postavka nije promijenjena.
hr.UninstallHelperFirst=Dodatna komponenta Digital Clock Windows Screen Saver (System Integration) još je registrirana. Najprije deinstaliraj tu zasebnu komponentu, a zatim Digital Clock. Nijedna datoteka nije uklonjena.
hr.SelectOtherSaverFirst=Prije deinstalacije Digital Clocka odaberi (None) ili drugi čuvar zaslona u postavkama Windowsa. Tvoje postavke neće se automatski mijenjati.
de.TaskActivate=Digital Clock-Bildschirmschoner nach der Installation aktivieren
de.TaskAutoStart=Digital Clock bei der Windows-Anmeldung starten
de.TaskKeepSaver=Digital Clock in der Windows-Bildschirmschonerliste behalten (Administratorbestätigung erforderlich; separate Systemkomponente)
de.LaunchDigitalClock=Digital Clock starten
de.HelperMissing=Das Installationsprogramm für den optionalen System-Bildschirmschoner fehlt. Die App wurde installiert, aber keine Systemintegration durchgeführt.
de.HelperHashMismatch=Die Hashprüfung des Bildschirmschoner-Hilfsprogramms ist fehlgeschlagen. Es wurde keine Administratorbestätigung angefordert.
de.HelperCancelled=Die Integration des System-Bildschirmschoners wurde abgebrochen oder ist fehlgeschlagen. Die Benutzer-App bleibt installiert; Windows-Einstellungen wurden nicht geändert.
de.SystemSaverMissing=Das Hilfsprogramm meldete Erfolg, aber der Bildschirmschoner fehlt in System32. Die Integration muss überprüft werden.
de.SystemSaverHashMismatch=Die Bildschirmschonerdatei in System32 stimmt nicht mit dieser Version überein. Der Bildschirmschoner wurde nicht aktiviert.
de.RegisteredSaver=Digital Clock ist in der Windows-Bildschirmschonerliste registriert. Der aktive Bildschirmschoner wurde nicht geändert.
de.UninstallHelperFirst=Die optionale Komponente Digital Clock Windows Screen Saver (System Integration) ist noch registriert. Bitte zuerst diese separate Komponente, dann Digital Clock deinstallieren. Es wurden keine Dateien entfernt.
de.SelectOtherSaverFirst=Bitte vor der Deinstallation von Digital Clock in den Windows-Einstellungen (Kein) oder einen anderen Bildschirmschoner wählen. Einstellungen werden nicht automatisch geändert.
it.TaskActivate=Attiva lo screen saver Digital Clock dopo l’installazione
it.TaskAutoStart=Avvia Digital Clock all’accesso a Windows
it.TaskKeepSaver=Mantieni Digital Clock nell’elenco degli screen saver di Windows (richiede l’approvazione dell’amministratore; componente di sistema separato)
it.LaunchDigitalClock=Avvia Digital Clock
it.HelperMissing=Manca il programma di installazione dello screen saver di sistema opzionale. L’app è installata, ma l’integrazione non è stata eseguita.
it.HelperHashMismatch=La verifica hash del programma ausiliario non è riuscita. Non è stata richiesta l’approvazione dell’amministratore.
it.HelperCancelled=L’integrazione dello screen saver di sistema è stata annullata o non è riuscita. L’app resta installata e le impostazioni di Windows non sono state modificate.
it.SystemSaverMissing=Il programma ausiliario ha segnalato il successo, ma lo screen saver manca in System32. Verificare l’integrazione.
it.SystemSaverHashMismatch=Il file dello screen saver in System32 non corrisponde a questa versione. Lo screen saver non è stato attivato.
it.RegisteredSaver=Digital Clock è registrato nell’elenco degli screen saver di Windows. Lo screen saver attivo non è stato modificato.
it.UninstallHelperFirst=Il componente opzionale Digital Clock Windows Screen Saver (System Integration) è ancora registrato. Disinstallare prima il componente separato, poi Digital Clock. Nessun file è stato rimosso.
it.SelectOtherSaverFirst=Prima di disinstallare Digital Clock, seleziona (Nessuno) o un altro screen saver nelle impostazioni di Windows. Le impostazioni non saranno modificate automaticamente.
fr.TaskActivate=Activer l’écran de veille Digital Clock après l’installation
fr.TaskAutoStart=Lancer Digital Clock à l’ouverture de session Windows
fr.TaskKeepSaver=Conserver Digital Clock dans la liste des écrans de veille Windows (autorisation administrateur requise ; composant système distinct)
fr.LaunchDigitalClock=Lancer Digital Clock
fr.HelperMissing=Le programme d’installation de l’écran de veille système facultatif est introuvable. L’application a été installée, mais aucune intégration système n’a été effectuée.
fr.HelperHashMismatch=La vérification de l’empreinte du programme auxiliaire a échoué. Aucune autorisation administrateur n’a été demandée.
fr.HelperCancelled=L’intégration de l’écran de veille système a été annulée ou a échoué. L’application reste installée ; les paramètres Windows n’ont pas été modifiés.
fr.SystemSaverMissing=Le programme auxiliaire a signalé un succès, mais l’écran de veille manque dans System32. L’intégration doit être vérifiée.
fr.SystemSaverHashMismatch=Le fichier de l’écran de veille dans System32 ne correspond pas à cette version. Aucun écran de veille n’a été activé.
fr.RegisteredSaver=Digital Clock est inscrit dans la liste des écrans de veille Windows. L’écran de veille actif n’a pas été modifié.
fr.UninstallHelperFirst=Le composant facultatif Digital Clock Windows Screen Saver (System Integration) est encore enregistré. Désinstallez d’abord ce composant distinct, puis Digital Clock. Aucun fichier n’a été supprimé.
fr.SelectOtherSaverFirst=Avant de désinstaller Digital Clock, choisissez (Aucun) ou un autre écran de veille dans les paramètres Windows. Vos paramètres ne seront pas modifiés automatiquement.
es.TaskActivate=Activar el protector de pantalla Digital Clock después de instalar
es.TaskAutoStart=Iniciar Digital Clock al iniciar sesión en Windows
es.TaskKeepSaver=Mantener Digital Clock en la lista de protectores de pantalla de Windows (requiere autorización de administrador; componente del sistema independiente)
es.LaunchDigitalClock=Iniciar Digital Clock
es.HelperMissing=Falta el instalador del protector de pantalla del sistema opcional. La aplicación se instaló, pero no se intentó la integración.
es.HelperHashMismatch=Falló la verificación hash del programa auxiliar. No se solicitó autorización de administrador.
es.HelperCancelled=La integración del protector de pantalla del sistema se canceló o falló. La aplicación sigue instalada y no se modificaron los ajustes de Windows.
es.SystemSaverMissing=El programa auxiliar informó que finalizó correctamente, pero falta el protector de pantalla en System32. Es necesario revisar la integración.
es.SystemSaverHashMismatch=El archivo del protector de pantalla en System32 no coincide con esta versión. No se activó ningún protector de pantalla.
es.RegisteredSaver=Digital Clock está registrado en la lista de protectores de pantalla de Windows. El protector activo no se ha modificado.
es.UninstallHelperFirst=El componente opcional Digital Clock Windows Screen Saver (System Integration) sigue registrado. Desinstala primero ese componente independiente y después Digital Clock. No se eliminó ningún archivo.
es.SelectOtherSaverFirst=Antes de desinstalar Digital Clock, selecciona (Ninguno) u otro protector en los ajustes de Windows. Tus ajustes no se modificarán automáticamente.

[Tasks]
Name: "activateScreensaver"; Description: "{cm:TaskActivate}"; Flags: unchecked
Name: "autoStart"; Description: "{cm:TaskAutoStart}"; Flags: unchecked
Name: "installSystemSaver"; Description: "{cm:TaskKeepSaver}"; Flags: unchecked
; Optional System32 helper is bundled but NEVER run unless the user selects its unchecked task.
[Files]
Source: "payload\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
[Icons]
Name: "{group}\Digital Clock"; Filename: "{app}\DigitalClock.exe"
Name: "{group}\Activate Digital Clock Screen Saver"; Filename: "{sys}\rundll32.exe"; Parameters: "desk.cpl,InstallScreenSaver ""{app}\AppsAndGames-DigitalClock-Windows.scr"""; IconFilename: "{app}\DigitalClock.exe"
Name: "{group}\Uninstall Digital Clock"; Filename: "{uninstallexe}"
[Registry]
Root: HKCU; Subkey: "Software\AppsAndGames\DigitalClockWindowsScreenSaverBridge"; ValueType: string; ValueName: "Target"; ValueData: "{app}\AppsAndGames-DigitalClock-Windows.scr"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "AppsAndGamesDigitalClockWindows"; ValueData: """{app}\DigitalClock.exe"""; Tasks: autoStart; Flags: uninsdeletevalue
[Run]
Filename: "{sys}\rundll32.exe"; Parameters: "desk.cpl,InstallScreenSaver ""{app}\AppsAndGames-DigitalClock-Windows.scr"""; Flags: postinstall skipifsilent; Tasks: activateScreensaver
Filename: "{app}\DigitalClock.exe"; Description: "{cm:LaunchDigitalClock}"; Flags: nowait postinstall skipifsilent
[Code]
// RC14 FIX2: optional per-machine System32 component; main Setup stays per-user.
// The helper has a separate uninstall entry and a fail-closed ownership check.
const
  HelperOwnerKey = 'Software\AppsAndGames\DigitalClockWindowsScreenSaverHelper';

procedure CurStepChanged(CurStep: TSetupStep);
var
  Helper, Bridge: String;
  ExitCode: Integer;
  Started: Boolean;
begin
  if (CurStep <> ssPostInstall) or WizardSilent then Exit;
  if not WizardIsTaskSelected('installSystemSaver') then Exit;

  Helper := ExpandConstant('{app}\Tools\DigitalClock-Windows-ScreenSaver-Setup-x64.exe');
  Bridge := ExpandConstant('{sys}\AppsAndGames-DigitalClock-Windows.scr');
  if not FileExists(Helper) then
  begin
    MsgBox(CustomMessage('HelperMissing'), mbError, MB_OK);
    Exit;
  end;
  // Detect modified/untrusted helper BEFORE UAC; repair/ownership also validated by the helper.
  if CompareText(GetSHA256OfFile(Helper), '{#HelperSHA}') <> 0 then
  begin
    MsgBox(CustomMessage('HelperHashMismatch'), mbError, MB_OK);
    Exit;
  end;
  ExitCode := -1;
  Started := ShellExec('runas', Helper, '/SP- /VERYSILENT /NORESTART', '', SW_SHOWNORMAL, ewWaitUntilTerminated, ExitCode);
  if (not Started) or (ExitCode <> 0) then
  begin
    MsgBox(CustomMessage('HelperCancelled'), mbInformation, MB_OK);
    Exit;
  end;
  if not FileExists(Bridge) then
  begin
    MsgBox(CustomMessage('SystemSaverMissing'), mbError, MB_OK);
    Exit;
  end;
  if CompareText(GetSHA256OfFile(Bridge), '{#BridgeSHA}') <> 0 then
  begin
    MsgBox(CustomMessage('SystemSaverHashMismatch'), mbError, MB_OK);
    Exit;
  end;
  MsgBox(CustomMessage('RegisteredSaver'), mbInformation, MB_OK);
end;

// FIX2.2: treat both the long path and its real Windows 8.3 alias as OUR saver.
// Never guess APPSAN~1.SCR; that alias may be owned by the older Rust pilot.
function MatchesInstalledSaver(const ActiveValue, InstalledPath: String): Boolean;
var
  Candidate, ShortPath: String;
begin
  Candidate := Trim(ActiveValue);
  if (Length(Candidate) >= 2) and (Copy(Candidate, 1, 1) = '"') and
     (Copy(Candidate, Length(Candidate), 1) = '"') then
    Candidate := Copy(Candidate, 2, Length(Candidate) - 2);
  Result := CompareText(Candidate, InstalledPath) = 0;
  if Result then Exit;
  if FileExists(InstalledPath) then
  begin
    ShortPath := GetShortName(InstalledPath);
    if ShortPath <> '' then
    begin
      Result := CompareText(Candidate, ShortPath) = 0;
      if not Result then Result := CompareText(Candidate, ExtractFileName(ShortPath)) = 0;
    end;
  end;
end;

function InitializeUninstall(): Boolean;
var CurrentSaver: String;
begin
  Result := True;
  // Fail closed: removing the per-user app first would strand the separately
  // managed System32 component. Remove it from Windows Installed Apps FIRST.
  if RegKeyExists(HKEY_LOCAL_MACHINE_64, HelperOwnerKey) then
  begin
    MsgBox(CustomMessage('UninstallHelperFirst'), mbError, MB_OK);
    Result := False;
    Exit;
  end;
  if RegQueryStringValue(HKCU, 'Control Panel\Desktop', 'SCRNSAVE.EXE', CurrentSaver) then
  begin
    if MatchesInstalledSaver(CurrentSaver, ExpandConstant('{app}\AppsAndGames-DigitalClock-Windows.scr')) or
       MatchesInstalledSaver(CurrentSaver, ExpandConstant('{sys}\AppsAndGames-DigitalClock-Windows.scr')) then
    begin
      MsgBox(CustomMessage('SelectOtherSaverFirst'), mbError, MB_OK);
      Result := False;
    end;
  end;
end;