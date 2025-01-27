@echo off

:: Remove any existing signature from AntiBitdefender.exe
signtool.exe remove /s "AntiBitdefender.exe"

:: Sign the AntiBitdefender.exe file with the PFX password
signtool.exe sign /f "UTKUDORUKBAYRAKTAR.pfx" /p "UTKUDORUKBAYRAKTAR" /fd SHA256 /t http://timestamp.digicert.com /a "AntiBitdefender.exe"

echo Files signed successfully
pause
