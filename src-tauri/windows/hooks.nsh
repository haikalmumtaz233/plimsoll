!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    ReadRegStr $R0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
    ${If} $R0 == "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" "
    ${OrIf} $R0 == "$\"$INSTDIR\${MAINBINARYNAME}.exe$\""
      DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
    ${EndIf}
  ${EndIf}
!macroend
