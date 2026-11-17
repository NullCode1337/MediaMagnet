#[cfg(windows)]
fn sys_is_24h() -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::Globalization::{GetLocaleInfoEx, LOCALE_SSHORTTIME};
    let mut buf = [0u16; 80];
    // `LOCALE_NAME_USER_DEFAULT` is NULL in Win32 headers.
    let n = unsafe { GetLocaleInfoEx(PCWSTR::null(), LOCALE_SSHORTTIME, Some(&mut buf)) };
    if n <= 0 {
        return true;
    }
    // "HH:mm" -> 24h, "h:mm tt" -> 12h
    String::from_utf16_lossy(&buf[..(n as usize - 1)]).contains('H')
}

#[cfg(target_os = "macos")]
fn sys_is_24h() -> bool {
    use objc2_foundation::{NSDateFormatter, NSLocale, NSString};
    let tpl = NSString::from_str("j");
    let fmt = unsafe {
        NSDateFormatter::dateFormatFromTemplate_options_locale(
            &tpl,
            0,
            Some(&NSLocale::currentLocale()),
        )
    };
    // 12h resolves to "h a", 24h to "HH"
    fmt.map(|s| !s.to_string().contains('a')).unwrap_or(true)
}

#[cfg(target_os = "linux")]
fn sys_is_24h() -> bool {
    // GNOME explicit toggle
    if let Ok(o) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "clock-format"])
        .output()
    {
        let s = String::from_utf8_lossy(&o.stdout);
        if s.contains("12h") {
            return false;
        }
        if s.contains("24h") {
            return true;
        }
    }
    // fallback: LC_TIME t_fmt
    unsafe {
        let p = libc::nl_langinfo(libc::T_FMT);
        let f = std::ffi::CStr::from_ptr(p).to_string_lossy();
        !(f.contains("%p") || f.contains("%I") || f.contains("%l") || f.contains("%r"))
    }
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn sys_is_24h() -> bool {
    true
}

#[tauri::command]
pub fn is_24h() -> bool {
    sys_is_24h()
}
