use tauri::Window;

#[cfg(target_os = "linux")]
pub fn set_x11_hints(window: &Window) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let window_id = window.hwnd()? as u32;

    // Use wmctrl to set window type to dock (always-on-top panel)
    let output = Command::new("wmctrl")
        .args(&["-i", "-r", &format!("0x{:x}", window_id), "-b", "add,sticky,above"])
        .output();

    match output {
        Ok(_) => {
            eprintln!("Set X11 window hints via wmctrl");
            Ok(())
        }
        Err(e) => {
            eprintln!("Warning: wmctrl not available ({}), window hints may not be applied", e);
            Ok(())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn set_x11_hints(_window: &Window) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
