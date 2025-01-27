@echo off
del UTKUDORUKBAYRAKTAR.pvk 2>nul
del UTKUDORUKBAYRAKTAR.cer 2>nul

:: Remove previous certificates with the same name from the personal store
certutil -delstore my "UTKU DORUK BAYRAKTAR" 2>nul

:: Create a self-signed certificate with makecert.exe
makecert.exe -r -pe -n "CN=UTKU DORUK BAYRAKTAR, E=satranckurslari1@gmail.com" -sv UTKUDORUKBAYRAKTAR.pvk UTKUDORUKBAYRAKTAR.cer -len 2048 -b 01/01/2025 -e 01/01/2035

pause
