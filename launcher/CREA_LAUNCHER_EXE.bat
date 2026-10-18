@echo off
setlocal DisableDelayedExpansion
cd /d "%~dp0"
where py >nul 2>&1
if not errorlevel 1 (
  py -3 "%~dp0build_launcher_exe.py" %*
) else (
  python "%~dp0build_launcher_exe.py" %*
)
set "RESULT=%ERRORLEVEL%"
if not "%RESULT%"=="0" echo Build stopped. Read the error above.
pause
exit /b %RESULT%
