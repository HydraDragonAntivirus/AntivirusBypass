@echo off
del HydraDragonOS.pfx 2>nul

:: Convert the PVK and CER files to a PFX file
pvk2pfx.exe -pvk HydraDragonOS.pvk -spc HydraDragonOS.cer -pfx HydraDragonOS.pfx -po DEATHOFANTIVIRUSESBYHYDRADRAGON

echo PFX file created successfully: HydraDragonOS.pfx
pause
