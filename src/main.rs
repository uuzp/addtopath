#![windows_subsystem = "windows"] // Hide console window on Windows

use std::env;
use std::process::exit;

// --- Windows Specific Code ---
#[cfg(windows)]
mod windows_ops {
    use std::env;
    use std::ffi::OsStr; // Removed unused OsString
    use std::os::windows::ffi::OsStrExt; // Removed unused OsStringExt
    use std::ptr::null_mut;
    use winreg::enums::*;
    use winreg::RegKey;
    use windows_sys::Win32::Foundation::{GetLastError, LPARAM, WPARAM, HWND};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE, HWND_BROADCAST, SW_HIDE,
    };
    // Corrected path for Shell items
    use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SHELLEXECUTEINFOW, SEE_MASK_DEFAULT};

    const ENV_KEY_PATH: &str = "Environment";
    const PATH_VALUE_NAME: &str = "Path";
    const REG_MENU_ROOT: &str = "Directory\\shell\\AddToPath";
    const REG_MENU_CMD: &str = "Directory\\shell\\AddToPath\\command";
    const REG_BG_MENU_ROOT: &str = "Directory\\Background\\shell\\AddToPath";
    const REG_BG_MENU_CMD: &str = "Directory\\Background\\shell\\AddToPath\\command";
    const MENU_TEXT: &str = "Add To Path";
    const MENU_ICON: &str = "imageres.dll,-5302";

    // --- Function to add paths to HKCU\Environment\Path ---
    pub fn add_to_user_path(paths_to_add: &[String]) -> Result<(), String> {
        if paths_to_add.is_empty() { return Err("No paths provided to add.".to_string()); }
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu.open_subkey_with_flags(ENV_KEY_PATH, KEY_READ | KEY_WRITE).map_err(|e| format!("Failed to open HKCU\\{}: {}", ENV_KEY_PATH, e))?;
        let current_path_val: String = env_key.get_value::<String, _>(PATH_VALUE_NAME).or_else(|e| { if e.kind() == std::io::ErrorKind::NotFound { Ok(String::new()) } else { env_key.get_value::<String, _>(PATH_VALUE_NAME).or_else(|e2| { if e2.kind() == std::io::ErrorKind::NotFound { Ok(String::new()) } else { Err(format!("Failed read '{}': {}/{}", PATH_VALUE_NAME, e, e2)) } }) } })?;
        let separator = ";";
        let mut existing_paths: Vec<String> = current_path_val.split(separator).filter(|s| !s.is_empty()).map(String::from).collect();
        let mut added_paths = Vec::new();
        let mut path_updated = false;
        for new_path_str in paths_to_add {
            if !new_path_str.is_empty() {
                let normalized_new_path = new_path_str.replace("/", "\\");
                let exists = existing_paths.iter().any(|existing| existing.eq_ignore_ascii_case(&normalized_new_path));
                if !exists { existing_paths.push(normalized_new_path.clone()); added_paths.push(normalized_new_path); path_updated = true; }
            }
        }
        if !path_updated { eprintln!("Info: Paths already exist or were empty."); return Ok(()); }
        let new_path_val = existing_paths.join(separator);
        env_key.set_value(PATH_VALUE_NAME, &new_path_val).map_err(|e| format!("Failed write '{}': {}", PATH_VALUE_NAME, e))?;
        eprintln!("Successfully added paths to user PATH:");
        for p in added_paths { eprintln!("- {}", p); }
        Ok(())
    }

    // --- Function to register context menu ---
    pub fn register_context_menu() -> Result<(), String> {
        eprintln!("Registering context menu...");
        let exe_path = env::current_exe().map_err(|e| format!("Failed get exe path: {}", e))?;
        let exe_path_str = exe_path.to_str().ok_or("Failed convert exe path")?;
        let command_folder = format!("\"{}\" \"%1\"", exe_path_str);
        let command_bg = format!("\"{}\" \"%V\"", exe_path_str);
        let hcr = RegKey::predef(HKEY_CLASSES_ROOT);
        let (key_folder, _) = hcr.create_subkey(REG_MENU_ROOT).map_err(|e| format!("Failed create {}: {}", REG_MENU_ROOT, e))?;
        key_folder.set_value("", &MENU_TEXT).map_err(|e| format!("Failed set text: {}", e))?;
        key_folder.set_value("Icon", &MENU_ICON).map_err(|e| format!("Failed set icon: {}", e))?;
        let (key_folder_cmd, _) = hcr.create_subkey(REG_MENU_CMD).map_err(|e| format!("Failed create {}: {}", REG_MENU_CMD, e))?;
        key_folder_cmd.set_value("", &command_folder).map_err(|e| format!("Failed set cmd: {}", e))?;
        let (key_bg, _) = hcr.create_subkey(REG_BG_MENU_ROOT).map_err(|e| format!("Failed create {}: {}", REG_BG_MENU_ROOT, e))?;
        key_bg.set_value("", &MENU_TEXT).map_err(|e| format!("Failed set bg text: {}", e))?;
        key_bg.set_value("Icon", &MENU_ICON).map_err(|e| format!("Failed set bg icon: {}", e))?;
        let (key_bg_cmd, _) = hcr.create_subkey(REG_BG_MENU_CMD).map_err(|e| format!("Failed create {}: {}", REG_BG_MENU_CMD, e))?;
        key_bg_cmd.set_value("", &command_bg).map_err(|e| format!("Failed set bg cmd: {}", e))?;
        eprintln!("Successfully registered context menu.");
        Ok(())
    }

    // --- Function to unregister context menu ---
    pub fn unregister_context_menu() -> Result<(), String> {
        eprintln!("Unregistering context menu...");
        let hcr = RegKey::predef(HKEY_CLASSES_ROOT);
        match hcr.delete_subkey_all(REG_MENU_ROOT) { Ok(_) => eprintln!("Deleted {}", REG_MENU_ROOT), Err(e) if e.kind() == std::io::ErrorKind::NotFound => eprintln!("{} not found", REG_MENU_ROOT), Err(e) => return Err(format!("Failed delete {}: {}", REG_MENU_ROOT, e)), }
        match hcr.delete_subkey_all(REG_BG_MENU_ROOT) { Ok(_) => eprintln!("Deleted {}", REG_BG_MENU_ROOT), Err(e) if e.kind() == std::io::ErrorKind::NotFound => eprintln!("{} not found", REG_BG_MENU_ROOT), Err(e) => return Err(format!("Failed delete {}: {}", REG_BG_MENU_ROOT, e)), }
        eprintln!("Successfully unregistered context menu.");
        Ok(())
    }

    // --- Helper to broadcast setting change ---
    pub fn broadcast_setting_change() -> Result<(), String> {
        let param_env = to_wide_null_terminated(OsStr::new("Environment"));
        let mut result = 0;
        let success = unsafe { SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0 as WPARAM, param_env.as_ptr() as LPARAM, SMTO_ABORTIFHUNG, 5000, &mut result,) };
        if success == 0 { let error_code = unsafe { GetLastError() }; Err(format!("Broadcast failed ({}). Restart needed?", error_code)) }
        else { eprintln!("Env change notification sent."); Ok(()) }
    }

    // --- Helper function to relaunch the process with elevation ---
    pub fn relaunch_with_elevation(arg: &str) -> Result<(), String> {
        let exe_path = env::current_exe().map_err(|e| format!("Failed get exe path: {}", e))?;
        let verb = to_wide_null_terminated(OsStr::new("runas"));
        let path = to_wide_null_terminated(exe_path.as_os_str());
        let params = to_wide_null_terminated(OsStr::new(arg));

        // Explicitly initialize all fields of SHELLEXECUTEINFOW, including the anonymous union
        let mut sei = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_DEFAULT,
            hwnd: 0 as HWND,
            lpVerb: verb.as_ptr(),
            lpFile: path.as_ptr(),
            lpParameters: params.as_ptr(),
            lpDirectory: null_mut(),
            nShow: SW_HIDE,
            hInstApp: 0 as _,
            lpIDList: null_mut(),
            lpClass: null_mut(),
            hkeyClass: 0 as _,
            dwHotKey: 0,
            // Correctly initialize the anonymous union by initializing one of its fields.
            // We need the definition of the union type, which is SHELLEXECUTEINFOW_0
            Anonymous: windows_sys::Win32::UI::Shell::SHELLEXECUTEINFOW_0 { hIcon: 0 as _ }, // Initialize hIcon within the union
            hProcess: 0 as _,
        };

        eprintln!("Relaunching with elevation for: {}", arg);
        // ShellExecuteExW returns BOOL (i32). Non-zero indicates success.
        let success_code: i32 = unsafe { ShellExecuteExW(&mut sei) };

        if success_code != 0 { // Check if the return code is non-zero
            eprintln!("Relaunch successful (UAC prompt). Exiting.");
            Ok(())
        } else {
            let error_code = unsafe { GetLastError() };
            Err(format!("ShellExecuteExW failed ({}), UAC cancelled?", error_code))
        }
    }

    // Helper function to convert OsStr to null-terminated wide string Vec<u16>
    fn to_wide_null_terminated(s: &OsStr) -> Vec<u16> {
        s.encode_wide().chain(std::iter::once(0)).collect()
    }
}

// --- Main Function ---
fn main() {
    let args: Vec<String> = env::args().collect();

    // --- Handle --reg / --unreg ---
    if args.len() == 2 {
        let command = args[1].as_str();
        if command == "--reg" || command == "--unreg" {
            #[cfg(windows)]
            {
                if !is_elevated::is_elevated() {
                    match windows_ops::relaunch_with_elevation(command) {
                        Ok(_) => exit(0),
                        Err(e) => { eprintln!("Elevation error: {}", e); exit(1); }
                    }
                } else {
                    eprintln!("Running elevated.");
                    let result = if command == "--reg" { windows_ops::register_context_menu() }
                                 else { windows_ops::unregister_context_menu() };
                    match result {
                        Ok(_) => exit(0),
                        Err(e) => { eprintln!("Registry op error: {}", e); exit(1); }
                    }
                }
            }
            #[cfg(not(windows))]
            { eprintln!("Error: --reg/--unreg only on Windows."); exit(1); }
        }
    }

    // --- Path Adding Logic ---
    let paths_to_process: Vec<String>;
    let mut is_double_click_mode = false;

    if args.len() > 1 {
        paths_to_process = args[1..].to_vec();
    } else {
        is_double_click_mode = true;
        #[cfg(windows)] {
            paths_to_process = match env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).and_then(|d| d.to_str().map(String::from)) {
                Some(dir) => vec![dir],
                None => { eprintln!("Error: Cannot get executable directory."); exit(1); }
            };
        }
        #[cfg(not(windows))] { eprintln!("Error: Double-click add only on Windows."); exit(1); }
    }

    // --- Platform Specific Path Adding ---
    #[cfg(windows)] {
        if is_double_click_mode { eprintln!("Adding exe dir to user PATH: {}", paths_to_process[0]); }
        match windows_ops::add_to_user_path(&paths_to_process) {
            Ok(_) => {
                eprintln!("PATH update done.");
                if let Err(e) = windows_ops::broadcast_setting_change() { eprintln!("{}", e); }
                exit(0);
            }
            Err(e) => { eprintln!("Error updating PATH: {}", e); exit(1); }
        }
    }
    #[cfg(not(windows))] {
         eprintln!("Warning: Modifying PATH only on Windows.");
         eprintln!("Printing combined path:");
         // ... non-windows path printing ...
         exit(0);
    }
}
