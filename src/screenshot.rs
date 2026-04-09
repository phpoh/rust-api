/// Trigger a fullscreen screenshot via OS keyboard simulation.
/// macOS: Cmd+Shift+3 → saves to Desktop
/// Windows: PrintScreen → copies to clipboard
pub fn take_screenshot() {
    #[cfg(target_os = "windows")]
    {
        windows_screenshot();
    }
    #[cfg(target_os = "macos")]
    {
        macos_screenshot();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        eprintln!("[SCREENSHOT] Unsupported platform");
    }
}

// ==================== Windows ====================

#[cfg(target_os = "windows")]
fn windows_screenshot() {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_SNAPSHOT,
    };

    let mut input_down: INPUT = unsafe { std::mem::zeroed() };
    input_down.r#type = INPUT_KEYBOARD;
    input_down.Anonymous.ki = KEYBDINPUT {
        wVk: VK_SNAPSHOT,
        wScan: 0,
        dwFlags: 0,
        time: 0,
        dwExtraInfo: 0,
    };

    let mut input_up: INPUT = unsafe { std::mem::zeroed() };
    input_up.r#type = INPUT_KEYBOARD;
    input_up.Anonymous.ki = KEYBDINPUT {
        wVk: VK_SNAPSHOT,
        wScan: 0,
        dwFlags: KEYEVENTF_KEYUP,
        time: 0,
        dwExtraInfo: 0,
    };

    unsafe {
        SendInput(1, &input_down, std::mem::size_of::<INPUT>() as i32);
        SendInput(1, &input_up, std::mem::size_of::<INPUT>() as i32);
    }
}

// ==================== macOS ====================

#[cfg(target_os = "macos")]
fn macos_screenshot() {
    use core_graphics::event::{CGEvent, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::CGEventSource;
    use core_graphics::event_source::CGEventSourceStateID;

    // macOS virtual key codes
    const KVK_COMMAND: CGKeyCode = 55; // 0x37
    const KVK_SHIFT: CGKeyCode = 56;   // 0x38
    const KVK_ANSI_3: CGKeyCode = 20;  // 0x14

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);

    if let Ok(src) = source {
        let tap = CGEventTapLocation::HID;

        // Key down: Cmd → Shift → 3
        if let Ok(event) = CGEvent::new_keyboard_event(&src, KVK_COMMAND, true) {
            event.post(tap);
        }
        if let Ok(event) = CGEvent::new_keyboard_event(&src, KVK_SHIFT, true) {
            event.post(tap);
        }
        if let Ok(event) = CGEvent::new_keyboard_event(&src, KVK_ANSI_3, true) {
            event.post(tap);
        }

        // Key up: 3 → Shift → Cmd
        if let Ok(event) = CGEvent::new_keyboard_event(&src, KVK_ANSI_3, false) {
            event.post(tap);
        }
        if let Ok(event) = CGEvent::new_keyboard_event(&src, KVK_SHIFT, false) {
            event.post(tap);
        }
        if let Ok(event) = CGEvent::new_keyboard_event(&src, KVK_COMMAND, false) {
            event.post(tap);
        }
    }
}
