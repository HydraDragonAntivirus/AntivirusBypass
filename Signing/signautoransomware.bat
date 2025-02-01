@echo off

:: Remove any existing signature from destructive.exe
signtool.exe remove /s "destructive".exe"

:: Sign the destructive.exe file with the PFX password
signtool.exe sign /f "HydraDragonOS.pfx" /p "DEATHOFANTIVIRUSESBYHYDRADRAGON" /fd SHA256 /t http://timestamp.digicert.com /a "destructive.exe"

echo Files signed successfully
pause
