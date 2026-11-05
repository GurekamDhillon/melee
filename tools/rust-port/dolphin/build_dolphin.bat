@echo off
rem Dolphin oracle: build the patched Slippi Dolphin (dolphin-emu-nogui only) under _build\dolphin-oracle.
rem First time: git clone --depth 1 https://github.com/project-slippi/dolphin.git _build\dolphin-oracle\src,
rem   git -C src checkout 0653df8194eac2392dea9fe26276b70f3c87fd89 (or the shallow tip of that date),
rem   git -C src submodule update --init --recursive --depth 1, git -C src apply tools\rust-port\dolphin\slippi-dolphin-nhv1.patch
rem Needs VS 2022 Build Tools (MSVC 14.44, Windows SDK 10.0.26100) and rustup (the Slippi Rust extensions pin 1.88.0).
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
set PATH=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin;%PATH%
set CARGO_TARGET_DIR=
set D=E:\Projects\Melee Workspace\_build\dolphin-oracle
cd /d "%D%"
if not exist build mkdir build
cd build
if not exist build.ninja cmake -G Ninja -DCMAKE_BUILD_TYPE=Release -DENABLE_QT=OFF -DENABLE_NOGUI=ON -DENABLE_TESTS=OFF -DENABLE_AUTOUPDATE=OFF -DUSE_DISCORD_PRESENCE=OFF -DENABLE_CLI_TOOL=OFF "-DCMAKE_EXE_LINKER_FLAGS=/MAP" ..\src || exit /b 1
cmake --build . --target dolphin-nogui -- -j 8 || exit /b 1
rem runtime files next to the exe: Sys (with Slippi's playback Gecko codes as the game settings) and the Rust extension DLL
set B=%D%\src\Binary\x64
if not exist "%B%\Sys" xcopy /E /I /Q "%D%\src\Data\Sys" "%B%\Sys" >nul
copy /Y "%D%\src\Data\PlaybackGeckoCodes\*" "%B%\Sys\GameSettings\" >nul
copy /Y "%D%\build\cargo\build\x86_64-pc-windows-msvc\release\slippi_rust_extensions.dll" "%B%\" >nul
rem the private user directory: Null video, no audio, unthrottled, no analytics (tools\rust-port\dolphin\Dolphin.ini)
if not exist "%D%\user\Config" mkdir "%D%\user\Config"
if not exist "%D%\user\Logs" mkdir "%D%\user\Logs"
copy /Y "%~dp0Dolphin.ini" "%D%\user\Config\Dolphin.ini" >nul
copy /Y "%~dp0Logger.ini" "%D%\user\Config\Logger.ini" >nul
echo dolphin oracle built: %B%\dolphin-emu-nogui.exe
