@echo off
setlocal DisableDelayedExpansion
cd /d "%~dp0"
py -3 "%~dp0build_launcher_exe.py"
set "RESULT=%ERRORLEVEL%"
pause
exit /b %RESULT%
