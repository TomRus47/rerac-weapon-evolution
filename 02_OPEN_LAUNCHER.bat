@echo off
setlocal DisableDelayedExpansion
cd /d "%~dp0"
where py >nul 2>&1
if not errorlevel 1 (
  py -3 "%~dp0launcher\launcher.py" %*
) else (
  python "%~dp0launcher\launcher.py" %*
)
set "RESULT=%ERRORLEVEL%"
if not "%RESULT%"=="0" pause
exit /b %RESULT%
