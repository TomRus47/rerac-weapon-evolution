@echo off
if not exist "%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" exit /b 1
for /f "usebackq tokens=*" %%i in (`"%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do call "%%i\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64
where cl >nul 2>&1
exit /b %ERRORLEVEL%
