@echo off

:: Remove any existing signature from AntivirusBypass.exe
signtool.exe remove /s "AntivirusBypass.exe"

:: Sign the AntivirusBypass.exe file with the PFX password
signtool.exe sign /f "UTKUDORUKBAYRAKTAR.pfx" /p "UTKUDORUKBAYRAKTAR" /fd SHA256 /t http://timestamp.digicert.com /a "AntivirusBypass.exe"

echo Files signed successfully
pause
