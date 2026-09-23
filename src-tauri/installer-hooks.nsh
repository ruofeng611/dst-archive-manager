; Tauri NSIS custom hooks.
;
; The uninstaller only removes the files it installed and does not know about
; the data the app creates at runtime (dst_data.db / logs), so they must be
; cleaned up explicitly here.
;
; NOTE: comments are kept in ASCII on purpose - makensis reads this file with
; the machine's ANSI code page unless it has a BOM, and non-ASCII comments are
; not worth the encoding risk in a build-tool input file.
;
; Only exact paths are removed on purpose: users may install into a folder that
; also holds other files, so a recursive `RMDir /r "$INSTDIR"` would be unsafe.

!macro NSIS_HOOK_PREUNINSTALL
  ; data and logs next to the executable (the normal location)
  Delete "$INSTDIR\dst_data.db"
  RMDir /r "$INSTDIR\logs"

  ; fallback location when the install directory is not writable
  Delete "$LOCALAPPDATA\dst-archive-manager\dst_data.db"
  RMDir /r "$LOCALAPPDATA\dst-archive-manager\logs"
  RMDir "$LOCALAPPDATA\dst-archive-manager"
!macroend
