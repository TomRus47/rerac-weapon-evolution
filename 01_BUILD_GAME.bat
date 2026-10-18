@echo off
setlocal DisableDelayedExpansion
cd /d "%~dp0"
where py >nul 2>&1
if not errorlevel 1 (
  py -3 "%~dp0launcher\editabile.py" build %*
) else (
  python "%~dp0launcher\editabile.py" build %*
)
set "RESULT=%ERRORLEVEL%"
if not "%RESULT%"=="0" pause
exit /b %RESULT%
