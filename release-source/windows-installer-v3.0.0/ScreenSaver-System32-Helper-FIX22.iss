; DIGITAL CLOCK RC14 FIX2.2 - 8.3-AWARE MULTI-USER UNINSTALL HELPER (INTERNAL REVIEW)
; NO PUBLIC DISTRIBUTION - pending multi-user Windows 10/11 x64 install/uninstall tests.
; Dedicated optional administrator component, not the user-mode application installer.
; Reuses RC8 component identity ONLY. RC7 pilot has a DIFFERENT AppId/file name.
; Registry marker belongs to the helper, NEVER to the current user.
; System integration must be runtime-tested with existing per-user clock settings.

#define BridgeFile "AppsAndGames-DigitalClock-Windows.scr"
#define BridgeHash GetSHA256OfFile(BridgeFile)

[Setup]
AppId={{6C8872BB-854E-4F1A-9093-F3DBB783AC19}
AppName=Digital Clock Windows Screen Saver (System Integration)
AppVersion=3.0.1
VersionInfoVersion=3.0.1.0
MinVersion=10.0
AppPublisher=Apps & Games
AppPublisherURL=https://appsandgames.org/digital-clock
CreateAppDir=no
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
DisableProgramGroupPage=yes
Uninstallable=yes
Output=yes
OutputDir=output
OutputBaseFilename=DigitalClock-Windows-ScreenSaver-RC14-FIX22-HELPER-x64
Compression=lzma2
SolidCompression=yes
CloseApplications=yes
RestartApplications=no

[Files]
; Don't overwrite anything until InitializeSetup has proven the SHA/owner marker.
; Shared system bridge: replacement is guarded by both marker and file hash.
; NO restartreplace / forced reboot-time delete. Inno must fail if file locked.
Source: "{#BridgeFile}"; DestDir: "{sys}"; Flags: ignoreversion

[Registry]
; Only helper Setup writes to HKLM. The per-user TARGET remains under each user's HKCU.
Root: HKLM64; Subkey: "Software\AppsAndGames\DigitalClockWindowsScreenSaverHelper"; ValueType: string; ValueName: "Owner"; ValueData: "6C8872BB-854E-4F1A-9093-F3DBB783AC19"; Flags: uninsdeletevalue
Root: HKLM64; Subkey: "Software\AppsAndGames\DigitalClockWindowsScreenSaverHelper"; ValueType: string; ValueName: "InstalledSHA256"; ValueData: "{#BridgeHash}"; Flags: uninsdeletevalue
Root: HKLM64; Subkey: "Software\AppsAndGames\DigitalClockWindowsScreenSaverHelper"; Flags: uninsdeletekeyifempty

[Code]
const
  OwnerKey = 'Software\AppsAndGames\DigitalClockWindowsScreenSaverHelper';
  OwnerId = '6C8872BB-854E-4F1A-9093-F3DBB783AC19';
  ProfileListKey = 'SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList';
  TemporaryHiveName = 'AAGDC_FIX21_TEMP_PROFILE_CHECK';
  BridgeFileName = 'AppsAndGames-DigitalClock-Windows.scr';

function BridgePath(): String;
begin
  Result := ExpandConstant('{sys}\' + BridgeFileName);
end;

function OwnershipIsVerified(): Boolean;
var
  Owner, RecordedHash, ActualHash: String;
begin
  Result := False;
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE_64, OwnerKey, 'Owner', Owner) then Exit;
  if CompareText(Owner, OwnerId) <> 0 then Exit;
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE_64, OwnerKey, 'InstalledSHA256', RecordedHash) then Exit;
  if Length(RecordedHash) <> 64 then Exit;
  if FileExists(BridgePath()) then
  begin
    try
      ActualHash := GetSHA256OfFile(BridgePath());
      if CompareText(ActualHash, RecordedHash) <> 0 then Exit;
    except
      Exit;
    end;
  end;
  Result := True;
end;

function InitializeSetup(): Boolean;
var
  HaveBridge, HaveMarker: Boolean;
begin
  Result := False;
  HaveBridge := FileExists(BridgePath());
  HaveMarker := RegKeyExists(HKEY_LOCAL_MACHINE_64, OwnerKey);
  // First installation: no collision and no pre-existing marker.
  if not HaveBridge and not HaveMarker then
  begin
    Result := True;
    Exit;
  end;
  // Repair/upgrade: require BOTH a known owner and verified previous file hash.
  // If a file exists but marker is absent/invalid (including RC8 without a marker), STOP.
  if not OwnershipIsVerified() then
  begin
    MsgBox('The system Screen Saver file or ownership record cannot be verified. ' +
      'No file was changed. A manual ownership/migration review is required.', mbError, MB_OK);
    Exit;
  end;
  // The helper file may be missing despite an intact marker (repair allowed).
  Result := True;
end;

// FIX2.2: The desktop may store SCRNSAVE.EXE using Windows 8.3 short paths.
// Compare the selected saver to OUR installed file only; never assume an
// APPSAN~N.SCR alias belongs to this app (an older pilot may own it).
function IsThisSaver(const Value: String): Boolean;
var
  Candidate, LongPath, ShortPath: String;
begin
  Candidate := Trim(Value);
  if (Length(Candidate) >= 2) and (Copy(Candidate, 1, 1) = '"') and
     (Copy(Candidate, Length(Candidate), 1) = '"') then
    Candidate := Copy(Candidate, 2, Length(Candidate) - 2);

  LongPath := BridgePath();
  Result := CompareText(Candidate, LongPath) = 0;
  if Result then Exit;
  // Also catch full long paths using differing case and recognized embedded names.
  Result := Pos('appsandgames-digitalclock-windows.scr', LowerCase(Candidate)) > 0;
  if Result then Exit;
  if FileExists(LongPath) then
  begin
    ShortPath := GetShortName(LongPath);
    if ShortPath <> '' then
    begin
      Result := CompareText(Candidate, ShortPath) = 0;
      // Windows can sometimes persist only the bare 8.3 file name.
      if not Result then Result := CompareText(Candidate, ExtractFileName(ShortPath)) = 0;
    end;
  end;
end;

function SafeToRemoveForHive(const Hive: String): Boolean;
var
  Saver: String;
  DesktopKey, PolicyKey: String;
begin
  Result := False;
  DesktopKey := Hive + '\Control Panel\Desktop';
  PolicyKey := Hive + '\Software\Policies\Microsoft\Windows\Control Panel\Desktop';
  // If SCRNSAVE.EXE has an unexpected type or cannot be read, STOP.
  if RegValueExists(HKEY_USERS, DesktopKey, 'SCRNSAVE.EXE') then
  begin
    if not RegQueryStringValue(HKEY_USERS, DesktopKey, 'SCRNSAVE.EXE', Saver) then Exit;
    if IsThisSaver(Saver) then Exit;
  end;
  // Also respect any per-user policy forcing the system saver.
  if RegValueExists(HKEY_USERS, PolicyKey, 'SCRNSAVE.EXE') then
  begin
    if not RegQueryStringValue(HKEY_USERS, PolicyKey, 'SCRNSAVE.EXE', Saver) then Exit;
    if IsThisSaver(Saver) then Exit;
  end;
  Result := True;
end;

// FIX2.1: Inspect unloaded *local* user profiles without altering their settings.
// We temporarily LOAD their NTUSER.DAT under a reserved HKU name and UNLOAD it
// before leaving this check. An unreadable hive, failed load, failed unload,
// or active/policy-selected Digital Clock always BLOCKS removal.
// DO NOT run this on a production machine until controlled multi-user tests pass.

var
  LastSafetyReason: String;

function IsUserProfileSid(const Sid: String): Boolean;
var
  S: String;
begin
  S := LowerCase(Sid);
  // Local/domain and Azure AD account SID families.
  // Service accounts are not interactive desktop profiles.
  Result := (Pos('s-1-5-21-', S) = 1) or (Pos('s-1-12-1-', S) = 1);
end;

function ExpandProfileDirectory(const Raw: String): String;
var
  WinDir, Drive: String;
begin
  WinDir := ExpandConstant('{win}');
  Drive := Copy(WinDir, 1, 2);
  Result := Raw;
  StringChangeEx(Result, '%SystemDrive%', Drive, True);
  StringChangeEx(Result, '%SystemRoot%', WinDir, True);
  StringChangeEx(Result, '%WinDir%', WinDir, True);
  // Unsupported variables or network/relative paths are not guessed.
  if (Pos('%', Result) > 0) or (Length(Result) < 3) then
  begin
    Result := '';
    Exit;
  end;
  if (Copy(Result, 2, 2) <> ':\') then Result := '';
end;

function RunRegHiveCommand(const Verb, HiveName, HiveFile: String): Boolean;
var
  Params: String;
  ExitCode: Integer;
begin
  Params := Verb + ' "HKU\' + HiveName + '"';
  if HiveFile <> '' then Params := Params + ' "' + HiveFile + '"';
  ExitCode := -1;
  Result := Exec(ExpandConstant('{sys}\reg.exe'), Params, '', SW_HIDE, ewWaitUntilTerminated, ExitCode);
  Result := Result and (ExitCode = 0);
end;

function ExamineProfile(const Sid: String): Boolean;
var
  RawPath, ProfileDirectory, HiveFile: String;
  CheckedSafe, Unloaded: Boolean;
begin
  Result := False;
  if RegKeyExists(HKEY_USERS, Sid) then
  begin
    if not SafeToRemoveForHive(Sid) then
    begin
      LastSafetyReason := 'Screen Saver is selected or registry state cannot be read for loaded profile ' + Sid;
      Exit;
    end;
    Result := True;
    Exit;
  end;

  // A single fixed temp name simplifies collision detection and safe cleanup.
  if RegKeyExists(HKEY_USERS, TemporaryHiveName) then
  begin
    LastSafetyReason := 'A temporary profile hive is already mounted. Manual review needed.';
    Exit;
  end;
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE_64, ProfileListKey + '\' + Sid, 'ProfileImagePath', RawPath) then
  begin
    LastSafetyReason := 'Missing or unreadable profile path for ' + Sid;
    Exit;
  end;
  ProfileDirectory := ExpandProfileDirectory(RawPath);
  if ProfileDirectory = '' then
  begin
    LastSafetyReason := 'Unsupported profile path for ' + Sid;
    Exit;
  end;
  HiveFile := AddBackslash(ProfileDirectory) + 'NTUSER.DAT';
  if not FileExists(HiveFile) then
  begin
    LastSafetyReason := 'Offline profile hive is missing/inaccessible for ' + Sid;
    Exit;
  end;
  if not RunRegHiveCommand('LOAD', TemporaryHiveName, HiveFile) then
  begin
    // A failing LOAD can leave a partial mount; never proceed or delete the saver.
    if RegKeyExists(HKEY_USERS, TemporaryHiveName) then
    begin
      if not RunRegHiveCommand('UNLOAD', TemporaryHiveName, '') then
        LastSafetyReason := 'LOAD failed and temporary HKU hive could not be unmounted. Manual review required.'
      else
        LastSafetyReason := 'Failed to mount offline hive for ' + Sid + '; temporary hive cleanup succeeded.';
    end
    else
      LastSafetyReason := 'Could not mount offline hive for ' + Sid + ' (no file was removed).';
    Exit;
  end;
  CheckedSafe := False;
  Unloaded := False;
  try
    // Fail closed if the hive is unexpectedly not visible after reg load.
    if RegKeyExists(HKEY_USERS, TemporaryHiveName) then
      CheckedSafe := SafeToRemoveForHive(TemporaryHiveName);
  finally
    // Always unmount, EVEN if the safety check raises an exception.
    Unloaded := RunRegHiveCommand('UNLOAD', TemporaryHiveName, '');
  end;
  if not Unloaded then
  begin
    LastSafetyReason := 'TEMPORARY HIVE UNLOAD FAILED for ' + Sid +
      '. STOP. Administrator must inspect HKU\' + TemporaryHiveName + ' manually.';
    Exit;
  end;
  if not CheckedSafe then
  begin
    LastSafetyReason := 'Digital Clock may be active or policy-forced in offline profile ' + Sid;
    Exit;
  end;
  Result := True;
end;

function AllProfilesKnownInactive(): Boolean;
var
  Profiles, Loaded: TArrayOfString;
  I, Count: Integer;
  Sid, MachinePolicy: String;
begin
  Result := False;
  LastSafetyReason := '';
  // Do not remove a saver selected in a computer-level policy either.
  if RegQueryStringValue(HKEY_LOCAL_MACHINE_64,
      'Software\Policies\Microsoft\Windows\Control Panel\Desktop',
      'SCRNSAVE.EXE', MachinePolicy) then
  begin
    if IsThisSaver(MachinePolicy) then
    begin
      LastSafetyReason := 'Machine policy selects Digital Clock.';
      Exit;
    end;
  end;
  if not RegGetSubkeyNames(HKEY_LOCAL_MACHINE_64, ProfileListKey, Profiles) then
  begin
    LastSafetyReason := 'Cannot enumerate Windows ProfileList.';
    Exit;
  end;
  if not RegGetSubkeyNames(HKEY_USERS, '', Loaded) then
  begin
    LastSafetyReason := 'Cannot enumerate loaded user hives.';
    Exit;
  end;
  Count := 0;
  for I := 0 to GetArrayLength(Profiles) - 1 do
  begin
    Sid := Profiles[I];
    if IsUserProfileSid(Sid) then
    begin
      Count := Count + 1;
      if not ExamineProfile(Sid) then Exit;
    end
    else if (Pos('S-1-', UpperCase(Sid)) <> 1) then
    begin
      LastSafetyReason := 'Unexpected ProfileList entry: ' + Sid;
      Exit;
    end;
  end;
  // Also examine any loaded account hive not registered in ProfileList.
  for I := 0 to GetArrayLength(Loaded) - 1 do
  begin
    Sid := Loaded[I];
    if IsUserProfileSid(Sid) then
    begin
      Count := Count + 1;
      if not SafeToRemoveForHive(Sid) then
      begin
        LastSafetyReason := 'Loaded profile still selects Digital Clock: ' + Sid;
        Exit;
      end;
    end;
  end;
  if Count = 0 then
  begin
    LastSafetyReason := 'No interactive user profiles found; cannot prove safety.';
    Exit;
  end;
  if RegKeyExists(HKEY_USERS, '.DEFAULT') then
    if not SafeToRemoveForHive('.DEFAULT') then
    begin
      LastSafetyReason := 'Windows default profile references Digital Clock.';
      Exit;
    end;
  Result := True;
end;

function InitializeUninstall(): Boolean;
begin
  Result := False;
  // Refuse removal of a foreign or tampered file, rather than risking another product.
  if not FileExists(BridgePath()) or not OwnershipIsVerified() then
  begin
    MsgBox('Screen Saver ownership/hash cannot be verified. ' +
      'Uninstall was cancelled to protect the shared System32 file.', mbError, MB_OK);
    Exit;
  end;
  if not AllProfilesKnownInactive() then
  begin
    MsgBox('Screen Saver cannot be removed safely. No file was removed.' + #13#10 +
      LastSafetyReason + #13#10 +
      'Select a different saver in the affected user account or request supervised cleanup.', mbError, MB_OK);
    Exit;
  end;
  Result := True;
end;
