; FM27 Tactical Architect — Windows NSIS Installer
; Build from repository root:
;   makensis -DVERSION=5.3.0 packaging/installer.nsi
; Outputs:
;   dist/FM27-Tactical-Architect-<VERSION>-setup.exe

!include "MUI2.nsh"

!ifndef VERSION
  !define VERSION "5.3.0"
!endif

!ifndef BIN_PATH
  !define BIN_PATH "..\target\release\fm27-tactical-architect.exe"
!endif

!ifndef ICON_PATH
  !define ICON_PATH "..\assets\icons\icon.ico"
!endif

!ifndef OUTFILE
  !define OUTFILE "..\dist\FM27-Tactical-Architect-${VERSION}-setup.exe"
!endif

!define APP_NAME "FM27 Tactical Architect"
!define EXE_NAME "fm27-tactical-architect.exe"
!define UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\FM27TacticalArchitect"

Name "${APP_NAME} ${VERSION}"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\${APP_NAME}"
InstallDirRegKey HKCU "${UNINST_KEY}" "InstallLocation"
RequestExecutionLevel user
Unicode true
SetCompressor /SOLID lzma

!define MUI_ICON "${ICON_PATH}"
!define MUI_UNICON "${ICON_PATH}"
!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\${EXE_NAME}"
!define MUI_FINISHPAGE_RUN_TEXT "Launch ${APP_NAME}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Portuguese"

Section "Install"
  SetOutPath "$INSTDIR"
  File "/oname=${EXE_NAME}" "${BIN_PATH}"
  File "/oname=icon.ico" "${ICON_PATH}"

  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Start Menu shortcut
  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${EXE_NAME}" "" "$INSTDIR\icon.ico" 0
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\icon.ico" 0

  ; Desktop shortcut
  CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${EXE_NAME}" "" "$INSTDIR\icon.ico" 0

  ; Add/Remove Programs metadata (per-user HKCU)
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINST_KEY}" "Publisher" "FM27 Tactical Architect"
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayIcon" "$INSTDIR\icon.ico"
  WriteRegStr HKCU "${UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINST_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKCU "${UNINST_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINST_KEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\${APP_NAME}.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"

  Delete "$INSTDIR\${EXE_NAME}"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  DeleteRegKey HKCU "${UNINST_KEY}"
SectionEnd
