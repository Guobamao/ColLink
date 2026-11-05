!define PRODUCT_NAME "ColLink"
!define PRODUCT_VERSION "1.0.0"
!define PRODUCT_PUBLISHER "ColLink Team"
!define PRODUCT_EXE "ColLink.exe"

SetCompressor /SOLID lzma

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "release\ColLink_Setup_v1.0.0.exe"
InstallDir "$PROGRAMFILES64\ColLink"
InstallDirRegKey HKLM "Software\ColLink" "InstallDir"
RequestExecutionLevel admin

Page directory
Page instfiles

UninstPage uninstConfirm
UninstPage instfiles

Section "MainSection" SEC01
  SetOutPath "$INSTDIR"
  SetOverwrite ifnewer
  File "release\ColLink-v1.0.0-Portable\ColLink.exe"
  File "release\ColLink-v1.0.0-Portable\collink-mcp.exe"
  File "app_icon.ico"
  
  CreateDirectory "$SMPROGRAMS\ColLink"
  CreateShortcut "$SMPROGRAMS\ColLink\ColLink.lnk" "$INSTDIR\ColLink.exe" "" "$INSTDIR\app_icon.ico"
  CreateShortcut "$DESKTOP\ColLink.lnk" "$INSTDIR\ColLink.exe" "" "$INSTDIR\app_icon.ico"
  
  WriteUninstaller "$INSTDIR\uninstall.exe"
  CreateShortcut "$SMPROGRAMS\ColLink\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ColLink" "DisplayName" "ColLink"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ColLink" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ColLink" "DisplayIcon" "$INSTDIR\app_icon.ico"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ColLink" "DisplayVersion" "1.0.0"
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\ColLink.lnk"
  Delete "$SMPROGRAMS\ColLink\ColLink.lnk"
  Delete "$SMPROGRAMS\ColLink\Uninstall.lnk"
  RMDir "$SMPROGRAMS\ColLink"
  
  Delete "$INSTDIR\ColLink.exe"
  Delete "$INSTDIR\collink-mcp.exe"
  Delete "$INSTDIR\app_icon.ico"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ColLink"
  DeleteRegKey HKLM "Software\ColLink"
SectionEnd
