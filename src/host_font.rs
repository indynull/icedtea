//! OS preference lists for UI and fixed-pitch families.
//!
//! Names only — selection against the loaded font database lives in
//! [`crate::typo`]. Returns empty when the platform has no API here
//! (Linux trusts fontconfig's mapping already applied to the database).

/// Preferred UI (proportional) family names, most preferred first.
pub(crate) fn ui_preferences() -> Vec<String> {
    platform::ui_preferences()
}

/// Preferred fixed-pitch family names, most preferred first.
pub(crate) fn mono_preferences() -> Vec<String> {
    platform::mono_preferences()
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::c_void;
    use std::ptr;

    // Two frameworks share `kind = "framework"`; clippy flags that as duplicated.
    #[allow(clippy::duplicated_attributes)]
    #[link(name = "CoreText", kind = "framework")]
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CTFontCreateUIFontForLanguage(
            ui_type: u32,
            size: f64,
            language: *const c_void,
        ) -> *mut c_void;
        fn CTFontCopyFamilyName(font: *mut c_void) -> *mut c_void;
        fn CTFontCopyDefaultCascadeListForLanguages(
            font: *mut c_void,
            language_pref_list: *const c_void,
        ) -> *mut c_void;
        fn CTFontCreateWithFontDescriptor(
            descriptor: *mut c_void,
            size: f64,
            matrix: *const c_void,
        ) -> *mut c_void;
        fn CFRelease(cf: *mut c_void);
        fn CFStringGetLength(the_string: *mut c_void) -> isize;
        fn CFStringGetCString(
            the_string: *mut c_void,
            buffer: *mut u8,
            buffer_size: isize,
            encoding: u32,
        ) -> u8;
        fn CFArrayGetCount(the_array: *mut c_void) -> isize;
        fn CFArrayGetValueAtIndex(the_array: *mut c_void, idx: isize) -> *mut c_void;
    }

    const CT_FONT_UI_USER_FIXED_PITCH: u32 = 1;
    const CT_FONT_UI_SYSTEM: u32 = 2;
    const CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

    pub(super) fn ui_preferences() -> Vec<String> {
        // iced → cosmic-text → swash rasterizes outlines with grayscale AA.
        // Prefer a proportional family that has *discrete* Regular + Bold
        // (Helvetica Neue, Lucida Grande): UI_BOLD must not cascade into
        // Menlo. Core Text's system cascade lists Menlo and many script
        // fallbacks; multi-axis System Font (SFNS) without an opsz axis
        // also looks harsh at 12–18px in swash. SF stays as a later choice.
        let mut out: Vec<String> = [
            "Helvetica Neue",
            "Lucida Grande",
            "Avenir Next",
            ".SF NS",
            "System Font",
            "SF Pro Text",
            "SF Pro",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        for name in families_for_ui_type(CT_FONT_UI_SYSTEM) {
            if looks_fixed_pitch_name(&name) {
                continue;
            }
            if !out.iter().any(|s| s == &name) {
                out.push(name);
            }
        }
        out
    }

    pub(super) fn mono_preferences() -> Vec<String> {
        let mut out = families_for_ui_type(CT_FONT_UI_USER_FIXED_PITCH);
        for alias in [".SF NS Mono", "SF Mono", "Menlo", "Monaco"] {
            if !out.iter().any(|s| s == alias) {
                out.push(alias.into());
            }
        }
        out
    }

    /// Names that must never be the UI (SansSerif) bind.
    fn looks_fixed_pitch_name(name: &str) -> bool {
        let n = name.to_ascii_lowercase();
        n.contains("mono")
            || n.contains("menlo")
            || n.contains("monaco")
            || n.contains("courier")
            || n.contains("andale")
            || n.contains("fixed")
            || n.contains("nerd font")
    }

    fn families_for_ui_type(ui_type: u32) -> Vec<String> {
        let mut out = Vec::new();
        unsafe {
            let font = CTFontCreateUIFontForLanguage(ui_type, 0.0, ptr::null());
            if font.is_null() {
                return out;
            }
            push_family(font, &mut out);
            let cascade = CTFontCopyDefaultCascadeListForLanguages(font, ptr::null());
            if !cascade.is_null() {
                let n = CFArrayGetCount(cascade);
                for i in 0..n {
                    let desc = CFArrayGetValueAtIndex(cascade, i);
                    if desc.is_null() {
                        continue;
                    }
                    let face = CTFontCreateWithFontDescriptor(desc, 0.0, ptr::null());
                    if face.is_null() {
                        continue;
                    }
                    push_family(face, &mut out);
                    CFRelease(face);
                }
                CFRelease(cascade);
            }
            CFRelease(font);
        }
        out
    }

    unsafe fn push_family(font: *mut c_void, out: &mut Vec<String>) {
        let name_ref = CTFontCopyFamilyName(font);
        if name_ref.is_null() {
            return;
        }
        if let Some(name) = cfstring_to_string(name_ref) {
            if !name.is_empty() && !out.iter().any(|s| s == &name) {
                out.push(name);
            }
        }
        CFRelease(name_ref);
    }

    unsafe fn cfstring_to_string(s: *mut c_void) -> Option<String> {
        let len = CFStringGetLength(s);
        if len < 0 {
            return None;
        }
        // UTF-8 worst case 4 bytes/unit + NUL
        let cap = (len as usize).saturating_mul(4).saturating_add(1);
        let mut buf = vec![0u8; cap];
        let ok = CFStringGetCString(
            s,
            buf.as_mut_ptr(),
            buf.len() as isize,
            CF_STRING_ENCODING_UTF8,
        );
        if ok == 0 {
            return None;
        }
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8(buf[..nul].to_vec()).ok()
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::mem::{size_of, zeroed};

    #[repr(C)]
    struct LogFontW {
        lf_height: i32,
        lf_width: i32,
        lf_escapement: i32,
        lf_orientation: i32,
        lf_weight: i32,
        lf_italic: u8,
        lf_underline: u8,
        lf_strike_out: u8,
        lf_char_set: u8,
        lf_out_precision: u8,
        lf_clip_precision: u8,
        lf_quality: u8,
        lf_pitch_and_family: u8,
        lf_face_name: [u16; 32],
    }

    #[repr(C)]
    struct NonClientMetricsW {
        cb_size: u32,
        i_border_width: i32,
        i_scroll_width: i32,
        i_scroll_height: i32,
        i_caption_width: i32,
        i_caption_height: i32,
        lf_caption_font: LogFontW,
        i_sm_caption_width: i32,
        i_sm_caption_height: i32,
        lf_sm_caption_font: LogFontW,
        i_menu_width: i32,
        i_menu_height: i32,
        lf_menu_font: LogFontW,
        lf_status_font: LogFontW,
        lf_message_font: LogFontW,
        i_padded_border_width: i32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SystemParametersInfoW(
            action: u32,
            ui_param: u32,
            pv_param: *mut core::ffi::c_void,
            f_win_ini: u32,
        ) -> i32;
    }

    const SPI_GET_NON_CLIENT_METRICS: u32 = 0x0029;

    pub(super) fn ui_preferences() -> Vec<String> {
        unsafe {
            let mut metrics: NonClientMetricsW = zeroed();
            metrics.cb_size = size_of::<NonClientMetricsW>() as u32;
            let ok = SystemParametersInfoW(
                SPI_GET_NON_CLIENT_METRICS,
                metrics.cb_size,
                (&raw mut metrics).cast(),
                0,
            );
            if ok == 0 {
                return Vec::new();
            }
            let mut out = Vec::new();
            for lf in [
                &metrics.lf_message_font,
                &metrics.lf_menu_font,
                &metrics.lf_status_font,
                &metrics.lf_caption_font,
                &metrics.lf_sm_caption_font,
            ] {
                if let Some(name) = face_name(lf) {
                    if !out.iter().any(|s| s == &name) {
                        out.push(name);
                    }
                }
            }
            out
        }
    }

    pub(super) fn mono_preferences() -> Vec<String> {
        // Windows has no separate fixed-pitch UI metric; mono is chosen
        // from monospaced faces already loaded into the font database.
        Vec::new()
    }

    unsafe fn face_name(lf: &LogFontW) -> Option<String> {
        let len = lf
            .lf_face_name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(lf.lf_face_name.len());
        if len == 0 {
            return None;
        }
        String::from_utf16(&lf.lf_face_name[..len]).ok()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    /// fontconfig already rewrote generic families when the database
    /// loaded; preferences stay empty so selection keeps that mapping
    /// when it is usable.
    pub(super) fn ui_preferences() -> Vec<String> {
        Vec::new()
    }

    pub(super) fn mono_preferences() -> Vec<String> {
        Vec::new()
    }
}
