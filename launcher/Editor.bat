@echo off
setlocal DisableDelayedExpansion
cd /d "%~dp0"
py -3 "%~dp0launcher.py"
set "RESULT=%ERRORLEVEL%"
pause
exit /b %RESULT%
