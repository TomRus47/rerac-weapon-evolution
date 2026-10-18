@echo off
setlocal DisableDelayedExpansion
cd /d "%~dp0"
where py >nul 2>&1
if not errorlevel 1 (
  py -3 "%~dp0final_release.py" %*
) else (
  python "%~dp0final_release.py" %*
)
set "RESULT=%ERRORLEVEL%"
if not "%RESULT%"=="0" echo Build stopped. Read the error above.
pause
exit /b %RESULT%
