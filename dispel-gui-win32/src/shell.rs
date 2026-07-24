//! Win32 application shell: WinMain, message loop, WndProc, menu, toolbar,
//! status bar, tab control, and 3-pane layout.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::Com::*;
use windows::Win32::Graphics::Gdi::*;

const ID_FILE_OPEN: u16 = 1001;
const ID_FILE_SAVE: u16 = 1002;
const ID_FILE_SAVE_AS: u16 = 1003;
const ID_FILE_EXIT: u16 = 1004;
const ID_EDIT_UNDO: u16 = 1011;
const ID_EDIT_REDO: u16 = 1012;
const ID_EDIT_CUT: u16 = 1013;
const ID_EDIT_COPY: u16 = 1014;
const ID_EDIT_PASTE: u16 = 1015;
const ID_VIEW_REFRESH: u16 = 1021;
const ID_HELP_ABOUT: u16 = 1041;

const IDC_MAIN_TOOLBAR: i32 = 2001;
const IDC_MAIN_STATUSBAR: i32 = 2002;
const IDC_MAIN_TABCONTROL: i32 = 2003;

const SIDEBAR_WIDTH: i32 = 250;
const HISTORY_WIDTH: i32 = 200;
const TOOLBAR_HEIGHT: i32 = 28;
const STATUSBAR_HEIGHT: i32 = 24;

pub fn run() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
        InitCommonControlsEx(&INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_WIN95_CLASSES,
        });

        let instance = GetModuleHandleW(None)?;

        // DPI awareness
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        // Register window class
        let class_name: Vec<u16> = OsStr::new("DispelWin32Main")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(main_wnd_proc),
            hInstance: instance,
            hIcon: LoadIconW(None, IDI_APPLICATION)?,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: GetStockObject(WHITE_BRUSH) as HBRUSH,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };
        RegisterClassExW(&wc);

        // Create main window
        let title: Vec<u16> = OsStr::new("Dispel Extractor - Win32")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1100,
            800,
            None,
            None,
            instance,
            None,
        );

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        CoUninitialize();
    }
    Ok(())
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => on_create(hwnd),
        WM_SIZE => on_size(hwnd),
        WM_COMMAND => on_command(hwnd, wparam),
        WM_NOTIFY => on_notify(hwnd, wparam, lparam),
        WM_CLOSE => on_close(hwnd),
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_PAINT => on_paint(hwnd),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn on_create(hwnd: HWND) -> LRESULT {
    // Menu bar
    create_menu_bar(hwnd);

    // Toolbar
    create_toolbar(hwnd);

    // Status bar
    create_status_bar(hwnd);

    // Tab control (main content area placeholder)
    create_tab_control(hwnd);

    // Sidebar (TreeView placeholder)
    create_sidebar(hwnd);

    // History panel placeholder
    create_history_panel(hwnd);

    // Initial layout
    layout_panes(hwnd);

    LRESULT(0)
}

unsafe fn on_size(hwnd: HWND) -> LRESULT {
    layout_panes(hwnd);
    LRESULT(0)
}

unsafe fn on_command(hwnd: HWND, wparam: WPARAM) -> LRESULT {
    let id = LOWORD(wparam.0) as u16;
    match id {
        ID_FILE_OPEN => on_file_open(hwnd),
        ID_FILE_SAVE => on_file_save(hwnd),
        ID_FILE_EXIT => {
            DestroyWindow(hwnd);
        }
        ID_EDIT_UNDO => {}
        ID_EDIT_REDO => {}
        ID_EDIT_CUT => {}
        ID_EDIT_COPY => {}
        ID_EDIT_PASTE => {}
        ID_VIEW_REFRESH => {}
        ID_HELP_ABOUT => {
            MessageBoxW(
                hwnd,
                w!("Dispel Extractor Win32\nVersion 0.8.0"),
                w!("About"),
                MB_OK | MB_ICONINFORMATION,
            );
        }
        _ => {}
    }
    LRESULT(0)
}

unsafe fn on_notify(hwnd: HWND, _wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let nmhdr = &*(lparam.0 as *const NMHDR);
    match nmhdr.code {
        TCN_SELCHANGE => {
            // Tab selection changed
        }
        _ => {}
    }
    LRESULT(0)
}

unsafe fn on_close(hwnd: HWND) -> LRESULT {
    DestroyWindow(hwnd);
    LRESULT(0)
}

unsafe fn on_paint(hwnd: HWND) -> LRESULT {
    let mut ps = PAINTSTRUCT::default();
    let _hdc = BeginPaint(hwnd, &mut ps);
    EndPaint(hwnd, &ps);
    LRESULT(0)
}

unsafe fn on_file_open(hwnd: HWND) {
    let mut ofn = OPENFILENAMEW::default();
    let mut file_buf = [0u16; 260];
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = hwnd;
    ofn.lpstrFile = file_buf.as_mut_ptr();
    ofn.nMaxFile = file_buf.len() as u32;
    ofn.lpstrFilter = w!("All Files\0*.*\0Dispel DB\0*.db\0Dispel INI\0*.ini\0Dispel REF\0*.ref\0Dispel SCR\0*.scr\0Dispel MAP\0*.map\0Dispel SPR\0*.spr\0Dispel SNF\0*.snf\0\0");
    ofn.nFilterIndex = 1;
    ofn.Flags = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;

    if GetOpenFileNameW(&mut ofn).as_bool() {
        // File selected - will be wired to editor in future phases
        let path = wide_to_string(&file_buf);
        // TODO: open file in appropriate editor
        let _ = path;
    }
}

unsafe fn on_file_save(_hwnd: HWND) {
    // TODO: implement save
}

fn create_menu_bar(hwnd: HWND) {
    unsafe {
        let menu = CreateMenu();
        let submenu_file = CreatePopupMenu();
        AppendMenuW(submenu_file, MF_STRING, ID_FILE_OPEN as usize, w!("&Open"));
        AppendMenuW(submenu_file, MF_STRING, ID_FILE_SAVE as usize, w!("&Save"));
        AppendMenuW(submenu_file, MF_STRING, ID_FILE_SAVE_AS as usize, w!("Save &As..."));
        AppendMenuW(submenu_file, MF_SEPARATOR, 0, None);
        AppendMenuW(submenu_file, MF_STRING, ID_FILE_EXIT as usize, w!("E&xit"));
        AppendMenuW(menu, MF_POPUP, submenu_file.0 as usize, w!("&File"));

        let submenu_edit = CreatePopupMenu();
        AppendMenuW(submenu_edit, MF_STRING, ID_EDIT_UNDO as usize, w!("&Undo"));
        AppendMenuW(submenu_edit, MF_STRING, ID_EDIT_REDO as usize, w!("&Redo"));
        AppendMenuW(submenu_edit, MF_SEPARATOR, 0, None);
        AppendMenuW(submenu_edit, MF_STRING, ID_EDIT_CUT as usize, w!("Cu&t"));
        AppendMenuW(submenu_edit, MF_STRING, ID_EDIT_COPY as usize, w!("&Copy"));
        AppendMenuW(submenu_edit, MF_STRING, ID_EDIT_PASTE as usize, w!("&Paste"));
        AppendMenuW(menu, MF_POPUP, submenu_edit.0 as usize, w!("&Edit"));

        let submenu_view = CreatePopupMenu();
        AppendMenuW(submenu_view, MF_STRING, ID_VIEW_REFRESH as usize, w!("&Refresh"));
        AppendMenuW(menu, MF_POPUP, submenu_view.0 as usize, w!("&View"));

        let submenu_help = CreatePopupMenu();
        AppendMenuW(submenu_help, MF_STRING, ID_HELP_ABOUT as usize, w!("&About"));
        AppendMenuW(menu, MF_POPUP, submenu_help.0 as usize, w!("&Help"));

        SetMenu(hwnd, menu);
    }
}

fn create_toolbar(hwnd: HWND) {
    unsafe {
        let toolbar = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("ToolbarWindow32"),
            None,
            WS_CHILD | WS_VISIBLE | TBSTYLE_TOOLTIPS | CCS_NODIVIDER,
            0, 0, 0, 0,
            hwnd,
            HMENU(IDC_MAIN_TOOLBAR as usize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        SendMessageW(toolbar, TB_BUTTONSTRUCTSIZE, WPARAM(std::mem::size_of::<TBBUTTON>() as usize), LPARAM(0));

        let buttons = [
            TBBUTTON {
                iBitmap: 0,
                idCommand: ID_FILE_OPEN as i32,
                fsState: TBSTATE_ENABLED,
                fsStyle: BTNS_BUTTON,
                dwData: 0,
                iString: 0,
            },
            TBBUTTON {
                iBitmap: 1,
                idCommand: ID_FILE_SAVE as i32,
                fsState: TBSTATE_ENABLED,
                fsStyle: BTNS_BUTTON,
                dwData: 0,
                iString: 0,
            },
            TBBUTTON {
                iBitmap: 2,
                idCommand: ID_EDIT_UNDO as i32,
                fsState: TBSTATE_ENABLED,
                fsStyle: BTNS_BUTTON,
                dwData: 0,
                iString: 0,
            },
            TBBUTTON {
                iBitmap: 3,
                idCommand: ID_EDIT_REDO as i32,
                fsState: TBSTATE_ENABLED,
                fsStyle: BTNS_BUTTON,
                dwData: 0,
                iString: 0,
            },
        ];
        SendMessageW(toolbar, TB_ADDBUTTONSW, WPARAM(buttons.len()), LPARAM(buttons.as_ptr() as isize));
    }
}

fn create_status_bar(hwnd: HWND) {
    unsafe {
        let sb = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("msctls_statusbar32"),
            None,
            WS_CHILD | WS_VISIBLE | SBARS_SIZEGRIP,
            0, 0, 0, 0,
            hwnd,
            HMENU(IDC_MAIN_STATUSBAR as usize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
        let parts = [0i32, 200, 400];
        SendMessageW(sb, SB_SETPARTS, WPARAM(parts.len()), LPARAM(parts.as_ptr() as isize));
    }
}

fn create_tab_control(hwnd: HWND) {
    unsafe {
        let _tab = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("SysTabControl32"),
            None,
            WS_CHILD | WS_VISIBLE | TCS_TABS | TCS_BUTTONS,
            0, 0, 0, 0,
            hwnd,
            HMENU(IDC_MAIN_TABCONTROL as usize),
            GetModuleHandleW(None).unwrap(),
            None,
        );
    }
}

fn create_sidebar(hwnd: HWND) {
    unsafe {
        let _tree = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("SysTreeView32"),
            None,
            WS_CHILD | WS_VISIBLE | TVS_HASBUTTONS | TVS_HASLINES | TVS_LINESATROOT | TVS_SHOWSELALWAYS,
            0, 0, 0, 0,
            hwnd,
            None,
            GetModuleHandleW(None).unwrap(),
            None,
        );
    }
}

fn create_history_panel(hwnd: HWND) {
    unsafe {
        let _history = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("SysListView32"),
            None,
            WS_CHILD | WS_VISIBLE | LVS_REPORT,
            0, 0, 0, 0,
            hwnd,
            None,
            GetModuleHandleW(None).unwrap(),
            None,
        );
    }
}

fn layout_panes(hwnd: HWND) {
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect);

        let client_width = rect.right - rect.left;
        let client_height = rect.bottom - rect.top;

        // Menu bar is above everything
        let mut menu_bar_height = 0;
        let menu_bar = GetMenu(hwnd);
        if !menu_bar.0.is_null() {
            let mut mbi = MENUINFO::default();
            mbi.cbSize = std::mem::size_of::<MENUINFO>() as u32;
            mbi.fMask = MIM_HEIGHT;
            if GetMenuInfo(menu_bar, &mut mbi).as_bool() {
                menu_bar_height = mbi.cyBar as i32;
            }
        }

        // Toolbar
        let toolbar = GetDlgItem(hwnd, IDC_MAIN_TOOLBAR);
        if !toolbar.0.is_null() {
            MoveWindow(toolbar, 0, menu_bar_height, client_width, TOOLBAR_HEIGHT, true);
        }

        let content_top = menu_bar_height + TOOLBAR_HEIGHT;
        let content_height = client_height - content_top - STATUSBAR_HEIGHT;

        // Sidebar
        let sidebar = FindWindowExW(hwnd, None, w!("SysTreeView32"), None);
        if !sidebar.0.is_null() {
            MoveWindow(sidebar, 0, content_top, SIDEBAR_WIDTH, content_height, true);
        }

        // Tab control (main content)
        let tab = GetDlgItem(hwnd, IDC_MAIN_TABCONTROL);
        if !tab.0.is_null() {
            MoveWindow(
                tab,
                SIDEBAR_WIDTH,
                content_top,
                client_width - SIDEBAR_WIDTH - HISTORY_WIDTH,
                content_height,
                true,
            );
        }

        // History panel
        let history = FindWindowExW(hwnd, None, w!("SysListView32"), None);
        if !history.0.is_null() {
            MoveWindow(
                history,
                client_width - HISTORY_WIDTH,
                content_top,
                HISTORY_WIDTH,
                content_height,
                true,
            );
        }

        // Status bar
        let statusbar = GetDlgItem(hwnd, IDC_MAIN_STATUSBAR);
        if !statusbar.0.is_null() {
            MoveWindow(
                statusbar,
                0,
                client_height - STATUSBAR_HEIGHT,
                client_width,
                STATUSBAR_HEIGHT,
                true,
            );
        }
    }
}

fn wide_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}
