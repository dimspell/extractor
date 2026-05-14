use super::Entry;

/// Description of a menu item for native rendering — strips the generic `Message`
/// so platform code doesn't need to know about Iced's message type.
#[derive(Debug)]
pub(crate) struct MenuItem {
    pub label: String,
    pub is_enabled: bool,
    pub is_separator: bool,
}

/// Result of showing a native context menu.
#[derive(Debug, Clone, Copy)]
pub(crate) enum NativeResult {
    /// User selected the item at this index.
    Selected(usize),
    /// Native menu was shown but the user cancelled (clicked outside).
    Cancelled,
}

/// Try to show a native OS context menu at the current cursor position.
///
/// Returns:
/// - `Some(NativeResult::Selected(idx))` — user picked item `idx`.
/// - `Some(NativeResult::Cancelled)` — native menu appeared but was dismissed
///   without a selection. Caller should **not** fall through to the custom overlay.
/// - `None` — native menus are not available on this platform. Caller should
///   fall back to the custom-rendered overlay.
pub(crate) fn try_show_native_menu<Message: Clone>(
    entries: &[Entry<Message>],
) -> Option<NativeResult> {
    // Set FORCE_CUSTOM_CONTEXT_MENU=1 to skip native menus and use
    // the Iced-rendered overlay instead (useful for debugging layout/style).
    // Bash: DISPEL_FORCE_CUSTOM_CONTEXT_MENU=1 cargo run -p dispel-gui
    if std::env::var("FORCE_CUSTOM_CONTEXT_MENU").is_ok() {
        return None;
    }

    let items: Vec<MenuItem> = entries
        .iter()
        .map(|e| match e {
            Entry::Item { label, .. } => MenuItem {
                label: label.clone(),
                is_enabled: true,
                is_separator: false,
            },
            Entry::Separator => MenuItem {
                label: String::new(),
                is_enabled: false,
                is_separator: true,
            },
            Entry::Disabled { label, .. } => MenuItem {
                label: label.clone(),
                is_enabled: false,
                is_separator: false,
            },
        })
        .collect();

    if items.is_empty() {
        return None;
    }

    #[cfg(target_os = "macos")]
    {
        Some(macos_show_menu(&items))
    }
    #[cfg(target_os = "windows")]
    {
        Some(windows_show_menu(&items))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = items;
        None
    }
}

// ── macOS (AppKit) ───────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn macos_show_menu(items: &[MenuItem]) -> NativeResult {
    use objc2::rc::Retained;
    use objc2::{sel, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSMenu, NSMenuItem, NSEvent};
    use objc2_foundation::NSString;

    let mtm = match MainThreadMarker::new() {
        Some(mtm) => mtm,
        None => return NativeResult::Cancelled,
    };

    let empty = NSString::from_str("");
    let menu: Retained<NSMenu> =
        NSMenu::initWithTitle(NSMenu::alloc(mtm), &empty);

    // Prevent AppKit from auto-disabling items with no valid responder target
    menu.setAutoenablesItems(false);

    for (idx, item) in items.iter().enumerate() {
        if item.is_separator {
            let sep = NSMenuItem::separatorItem(mtm);
            menu.addItem(&sep);
        } else {
            let title = NSString::from_str(&item.label);
            let menu_item = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(mtm),
                    &title,
                    Some(sel!(dummy:)), // dummy — we detect selection via highlightedItem tag
                    &empty,
                )
            };
            menu_item.setTag(idx as isize);
            menu_item.setEnabled(item.is_enabled);
            menu.addItem(&menu_item);
        }
    }

    let location = NSEvent::mouseLocation();
    let did_select = menu.popUpMenuPositioningItem_atLocation_inView(None, location, None);

    if did_select {
        if let Some(highlighted) = menu.highlightedItem() {
            let tag = highlighted.tag();
            if tag >= 0 {
                let idx = tag as usize;
                if idx < items.len() && items[idx].is_enabled {
                    return NativeResult::Selected(idx);
                }
            }
        }
    }

    NativeResult::Cancelled
}

// ── Windows (Win32) ──────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn windows_show_menu(items: &[MenuItem]) -> NativeResult {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    type HMENU = *mut std::ffi::c_void;
    type HWND = *mut std::ffi::c_void;
    type UINT = u32;

    const MF_STRING: UINT = 0;
    const MF_SEPARATOR: UINT = 0x800;
    const MF_GRAYED: UINT = 0x0001;
    const TPM_RETURNCMD: UINT = 0x0100;
    const TPM_RIGHTBUTTON: UINT = 0x0002;

    extern "system" {
        fn CreatePopupMenu() -> HMENU;
        fn AppendMenuW(
            hmenu: HMENU,
            flags: UINT,
            id_new_item: usize,
            new_item: *const u16,
        ) -> i32;
        fn TrackPopupMenu(
            hmenu: HMENU,
            flags: UINT,
            x: i32,
            y: i32,
            reserved: i32,
            hwnd: HWND,
            reserved_rect: *const std::ffi::c_void,
        ) -> i32;
        fn DestroyMenu(hmenu: HMENU) -> i32;
        fn GetCursorPos(point: *mut POINT) -> i32;
        fn GetActiveWindow() -> HWND;
    }

    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }

    unsafe {
        let hmenu = CreatePopupMenu();
        if hmenu.is_null() {
            return NativeResult::Cancelled;
        }

        for (idx, item) in items.iter().enumerate() {
            if item.is_separator {
                AppendMenuW(hmenu, MF_SEPARATOR, 0, ptr::null());
            } else {
                let flags = if item.is_enabled {
                    MF_STRING
                } else {
                    MF_STRING | MF_GRAYED
                };
                let wide: Vec<u16> = OsStr::new(&item.label)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                // Offset ID by 1 so 0 means "cancelled" (TrackPopupMenu returns 0
                // for both "no selection" and "item with ID 0").
                AppendMenuW(hmenu, flags, idx + 1, wide.as_ptr());
            }
        }

        let mut cursor = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut cursor) == 0 {
            DestroyMenu(hmenu);
            return NativeResult::Cancelled;
        }

        let hwnd = GetActiveWindow();
        let cmd = TrackPopupMenu(
            hmenu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            cursor.x,
            cursor.y,
            0,
            hwnd,
            ptr::null(),
        );

        DestroyMenu(hmenu);

        if cmd == 0 {
            return NativeResult::Cancelled;
        }

        let idx = (cmd - 1) as usize;
        if idx < items.len() && items[idx].is_enabled {
            return NativeResult::Selected(idx);
        }
    }

    NativeResult::Cancelled
}
