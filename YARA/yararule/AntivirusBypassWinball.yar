rule AntivirusBypassWinball {
    meta:
        description = "Detects antivirus bypass techniques related to Windows-based malware"
        author = "Emirhan Ucan"
        version = "0.2"
        category = "malware/ransomware/antivirus-bypass"
        reference = "https://github.com/HydraDragonAntivirus/AntivirusBypass"
        date = "2025-01-31"
    
    strings:
        // --- Safe Mode Bypass ---
        // Detects Safe Mode bypass checks in batch files
        $safe_mode_check = "bcdedit /enum {current} | findstr /i \"safeboot\""

        // --- Winlogon Registry Modification ---
        // Detects modification of Winlogon registry key to run custom batch file
        $winlogon_shell_mod = "reg delete \"HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon\" /v Shell /f"
        $winlogon_shell_add = "reg add \"HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon\" /v Shell /t REG_SZ /d \"explorer.exe, c:\\program files\\utkudrk2\\utkudrk2.bat\" /f"

        // --- Destructive Payload Execution ---
        // Detects execution of destructive payload
        $destructive_exec = "C:\\Program Files\\utkudrk2\\destructive.exe"

        // --- Antivirus File Deletions ---
        // Detects file deletions related to antivirus services
        $delete_antivirus_files = "del /f \"C:\\windows\\system32\\drivers\\wrkrn.sys\""
        $delete_webroot_files = "del /f \"C:\\windows\\system32\\wruser.dll\""
        $delete_wrcore_files = "del /f \"C:\\Program Files\\Webroot\\*.*\""
        $delete_wr_data = "del /f \"C:\\ProgramData\\WRCore\\*.*\""

        // --- Antivirus Registry Deletions ---
        // Detects deletion of antivirus registry entries
        $delete_registry_keys = "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\WinDefend\" /f"
        $delete_antivirus_service = "sc delete WRSkyClient"

        // Additional registry deletions related to antivirus services
        $delete_webroot_registry = "reg delete \"HKLM\\SOFTWARE\\WOW6432Node\\Webroot\" /f"
        $delete_avast_registry = "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\avast! Antivirus\" /f"
        $delete_mbam_registry = "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\MBAMService\" /f"
        $delete_vss_registry = "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\VSSERV\" /f"
        
        // --- Safe Mode Cleanup ---
        // Detects cleanup of Safe Mode setting
        $remove_safe_mode = "bcdedit /deletevalue {current} safeboot"

        // --- Other Antivirus-Related Registry and File Deletions ---
        $delete_av_registry = "reg delete \"HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run\" /f"
        $delete_aswbidsdriver = "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswbidsdriver\" /f"
        $delete_aswbidsdriver2 = "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswbidsdriver\" /f"
        $delete_aswbuniv = "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswbuniv\" /f"
        $delete_aswbuniv2 = "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswbuniv\" /f"
        $delete_aswApPct = "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswApPct\" /f"
        $delete_aswApPct2 = "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswApPct\" /f"
        $delete_aswMonFit = "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswMonFit\" /f"
        $delete_aswMonFit2 = "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswMonFit\" /f"
        $delete_aswKbd = "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswKbd\" /f"
        $delete_aswKbd2 = "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswKbd\" /f"
        $delete_aswbidsdriver_full = "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswbidsdriver\" /f"
        $delete_aswbidsdriver_full2 = "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswbidsdriver\" /f"

        // Webroot specific deletions
        $delete_webroot_folders = "rd /s /q \"C:\\Program Files\\Webroot\\\""
        $delete_webroot_folders_x86 = "rd /s /q \"C:\\Program Files (x86)\\Webroot\\\""
        $delete_wr_data_folder = "rd /s /q \"C:\\ProgramData\\WRData\\\""
        $delete_wr_core_folder = "rd /s /q \"C:\\ProgramData\\WRCore\\\""

    condition:
        // Trigger if any two of the specified behaviors are present
        (any of ($safe_mode_check, $winlogon_shell_mod, $winlogon_shell_add, $destructive_exec) and
         any of ($delete_antivirus_files, $delete_webroot_files, $delete_registry_keys, $delete_antivirus_service,
                $delete_wrcore_files, $delete_wr_data, $delete_webroot_registry, $delete_avast_registry, 
                $delete_mbam_registry, $delete_vss_registry, $remove_safe_mode,
                $delete_av_registry, $delete_aswbidsdriver, $delete_aswbidsdriver2, $delete_aswbuniv, $delete_aswbuniv2,
                $delete_aswApPct, $delete_aswApPct2, $delete_aswMonFit, $delete_aswMonFit2, $delete_aswKbd, $delete_aswKbd2,
                $delete_webroot_folders, $delete_webroot_folders_x86, $delete_wr_data_folder, $delete_wr_core_folder))
}
