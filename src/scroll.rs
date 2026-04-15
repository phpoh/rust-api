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
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetForegroundWindow, PostMessageW, WM_MOUSEWHEEL,
    };

    // Clamp step to avoid i32 overflow when multiplying by 120
    let clamped_step = step.min(i32::MAX as i64 / 120);
    let wheel_delta: i32 = if delta >= 0 {
        (clamped_step as i32) * 120
    } else {
        -((clamped_step as i32) * 120)
    };

    let hwnd = unsafe { GetForegroundWindow() };
    let mut point = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut point) };

    let w_param = (wheel_delta as isize) << 16;
    let l_param = ((point.y as isize) << 16) | (point.x as isize & 0xFFFF);

    unsafe {
        PostMessageW(hwnd, WM_MOUSEWHEEL, w_param as usize, l_param as isize);
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
        if let Ok(event) = CGEvent::new_scroll_event(src, units, 1, scroll_count, 0, 0) {
            event.post(CGEventTapLocation::HID);
        }
    }
}
