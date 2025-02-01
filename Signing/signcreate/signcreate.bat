@echo off
del HydraDragonOS.pvk 2>nul
del HydraDragonOS.cer 2>nul

:: Remove previous certificates with the same name from the personal store
certutil -delstore my "HydraDragonOS" 2>nul

:: Create a self-signed certificate with makecert.exe
makecert.exe -r -pe -n "CN=HydraDragonOS, E=protonkral5668@proton.me" -sv HydraDragonOS.pvk HydraDragonOS.cer -len 2048 -b 01/01/2025 -e 01/01/2035

pause
