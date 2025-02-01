@echo off

:: Remove any existing signature from explorer.exe
signtool.exe remove /s "explorer.exe"

:: Sign the explorer.exe file with the PFX password
signtool.exe sign /f "HydraDragonOS.pfx" /p "DEATHOFANTIVIRUSESBYHYDRADRAGON" /fd SHA256 /t http://timestamp.digicert.com /a "explorer.exe"

echo Files signed successfully
pause
