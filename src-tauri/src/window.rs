use tauri::Window;

#[cfg(target_os = "linux")]
pub fn set_x11_hints(window: &Window) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let title = window.title()?;

    // Use wmctrl to set window type to dock (always-on-top panel)
    let output = Command::new("wmctrl")
        .args(&["-r", &title, "-b", "add,sticky,above"])
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
