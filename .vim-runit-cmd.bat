@ECHO OFF
SETLOCAL

set TAGSINATE_BINARY=%~dp0\target\debug\tagsinate.exe
pushd h:\Goals\Game

IF /i "%1" == "make" GOTO MAKE
IF /i "%1" == "test" GOTO TEST

REM === MAKE ===
:MAKE
cargo build
GOTO Exit
REM ============

REM === TEST ===
:TEST
for /f "tokens=1,* delims= " %%a in ("%*") do set ALL_BUT_FIRST_ARG=%%b
%TAGSINATE_BINARY% %ALL_BUT_FIRST_ARG%
GOTO Exit

:Exit
popd
EXIT /B %ERRORLEVEL%
