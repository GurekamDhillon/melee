@echo off
rem Build Melee Unlocked's no-Slippi launcher (MeleeUnlockedLauncher.exe) straight from a checkout with cl.exe, for
rem the Rust-engine seam test (docs/rust-port/notes/launcher-seam.md). No CMake: the same sources and defines as
rem port/CMakeLists.txt's melee_unlocked target under MELEE_NO_SLIPPI.
rem   build_mu_launcher.bat <melee-unlocked checkout> <output dir on E:>
setlocal
set SRC=%~1\port
set OUT=%~2
if not exist "%OUT%" mkdir "%OUT%"
call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" >nul || exit /b 1
set /p VER=<"%~1\VERSION"
cd /d "%OUT%"
rc /nologo /I "%SRC%\app" /D MELEE_PORT_VERSION=\"%VER%\" /D MELEE_VERSION_COMMAS=0,9,86,0 /fo launcher.res "%SRC%\app\launcher.rc" || exit /b 1
rem 05871a4a (0.9.990+): the RVZ/.slpz reader (port/app/disc_rvz.cmake: port_disc_rvz, zstd/lzma/bzip2) and the YouGame
rem web sign-in (melee_yougame_web: WebView2 static loader) are part of the launcher; older checkouts lack them and skip them.
set TP=%SRC%\third_party
set EXTRA=
set EXTRALIB=
if exist "%SRC%\app\disc_rvz.cpp" (
  cl /nologo /O2 /MP /c /W1 /DZSTD_DISABLE_ASM=1 /DXXH_NAMESPACE=ZSTD_ /DBZ_EXPORT /D_CRT_SECURE_NO_WARNINGS /DNOMINMAX /I "%SRC%\app" /I "%TP%" ^
    "%TP%\zstd\common\*.c" "%TP%\zstd\decompress\*.c" "%TP%\zstd\compress\*.c" "%TP%\lzma\LzmaDec.c" "%TP%\lzma\Lzma2Dec.c" ^
    "%TP%\bzip2\blocksort.c" "%TP%\bzip2\huffman.c" "%TP%\bzip2\crctable.c" "%TP%\bzip2\randtable.c" "%TP%\bzip2\compress.c" "%TP%\bzip2\decompress.c" "%TP%\bzip2\bzlib.c" > build_rvz.log 2>&1 || exit /b 1
  cl /nologo /O2 /MP /c /EHsc /std:c++17 /DZSTD_DISABLE_ASM=1 /DXXH_NAMESPACE=ZSTD_ /D_CRT_SECURE_NO_WARNINGS /DNOMINMAX /I "%SRC%\app" /I "%TP%" ^
    "%SRC%\app\disc_rvz.cpp" "%SRC%\app\replay_slpz.cpp" >> build_rvz.log 2>&1 || exit /b 1
  lib /nologo /OUT:port_disc_rvz.lib *.obj >> build_rvz.log 2>&1 || exit /b 1
  del /q *.obj 2>nul
  set EXTRALIB=port_disc_rvz.lib
)
if exist "%SRC%\app\launcher_yougame.cpp" (
  set EXTRA="%SRC%\app\launcher_yougame.cpp" "%SRC%\app\launcher_yougame_web.cpp" "%SRC%\app\launcher_yougame_app.cpp" "%SRC%\runtime\hle\yougame_channel.cpp" /I "%TP%\webview2\include"
  set EXTRALIB=port_disc_rvz.lib "%TP%\webview2\x64\WebView2LoaderStatic.lib" shlwapi.lib
)
cl /nologo /O2 /MP /EHsc /std:c++17 /DMELEE_NO_SLIPPI /DNOMINMAX /DUNICODE /D_UNICODE /D_CRT_SECURE_NO_WARNINGS /DMELEE_PORT_VERSION=\"%VER%\" /DMELEE_VERSION_COMMAS=0,9,86,0 ^
  /I "%SRC%\runtime\host" /I "%SRC%\runtime\ppc" /I "%SRC%\runtime\hle" /I "%SRC%\runtime\gx" /I "%SRC%\generated" /I "%SRC%\third_party" /I "%SRC%\third_party\dht" /I "%SRC%\third_party\monocypher" /I "%SRC%\app" ^
  "%SRC%\app\launcher_mod_catalog.cpp" "%SRC%\runtime\host\mod_scan.cpp" "%SRC%\runtime\host\mod_profile.cpp" "%SRC%\app\launcher.cpp" "%SRC%\app\launcher_replay_data.cpp" "%SRC%\app\launcher_lobby.cpp" "%SRC%\app\launcher_lobby_p2p.cpp" ^
  "%SRC%\third_party\dht\dht.c" "%SRC%\third_party\monocypher\monocypher.c" "%SRC%\third_party\monocypher\monocypher-ed25519.c" "%SRC%\runtime\host\updater.cpp" "%SRC%\runtime\host\net_trace_read.cpp" "%SRC%\app\launcher_trace_view.cpp" ^
  %EXTRA% launcher.res /FeMeleeUnlockedLauncher.exe /link /SUBSYSTEM:WINDOWS %EXTRALIB% comctl32.lib winhttp.lib ws2_32.lib bcrypt.lib user32.lib gdi32.lib shell32.lib comdlg32.lib msimg32.lib uxtheme.lib winmm.lib ole32.lib advapi32.lib version.lib dxgi.lib > build.log 2>&1
set RC=%ERRORLEVEL%
del /q *.obj 2>nul
exit /b %RC%
