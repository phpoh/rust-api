/// Perform an OS-level scroll event on the foreground window.
pub fn perform_scroll(delta: i64, step: i64, _speed: f64) {
    #[cfg(target_os = "windows")]
    {
        windows_scroll(delta, step);
    }
    #[cfg(target_os = "macos")]
    {
        macos_scroll(delta, step, _speed);
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (delta, step, _speed);
    }
}

/// One-time initialization for the scroll subsystem.
pub fn init() {
    // No init needed on any current platform.
}

// ==================== Windows ====================

#[cfg(target_os = "windows")]
fn windows_scroll(delta: i64, step: i64) {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_MOUSE, MOUSEEVENTF_WHEEL, MOUSEINPUT,
    };

    let wheel_delta: i32 = if delta >= 0 {
        (step as i32) * 120
    } else {
        -((step as i32) * 120)
    };

    let mut input: INPUT = unsafe { std::mem::zeroed() };
    input.r#type = INPUT_MOUSE;
    input.Anonymous.mi = MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: wheel_delta as u32,
        dwFlags: MOUSEEVENTF_WHEEL,
        time: 0,
        dwExtraInfo: 0,
    };

    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}

// ==================== macOS ====================

#[cfg(target_os = "macos")]
fn macos_scroll(delta: i64, step: i64, speed: f64) {
    use core_graphics::event::{CGEvent, CGEventTapLocation, ScrollEventUnit};
    use core_graphics::event_source::CGEventSource;
    use core_graphics::event_source::CGEventSourceStateID;

    let units = if speed > 0.5 {
        ScrollEventUnit::PIXEL
    } else {
        ScrollEventUnit::LINE
    };

    let scroll_count: i32 = if delta >= 0 {
        -(step as i32)
    } else {
        step as i32
    };

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok();

    if let Some(src) = source {
        if let Some(event) = CGEvent::new_scroll_event(&src, units, 1, scroll_count, 0, 0) {
            event.post(CGEventTapLocation::HID);
        }
    }
}
