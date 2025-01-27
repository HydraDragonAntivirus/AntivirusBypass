@echo off
del UTKUDORUKBAYRAKTAR.pfx 2>nul

:: Convert the PVK and CER files to a PFX file
pvk2pfx.exe -pvk UTKUDORUKBAYRAKTAR.pvk -spc UTKUDORUKBAYRAKTAR.cer -pfx UTKUDORUKBAYRAKTAR.pfx -po UTKUDORUKBAYRAKTAR

echo PFX file created successfully: UTKUDORUKBAYRAKTAR.pfx
pause
