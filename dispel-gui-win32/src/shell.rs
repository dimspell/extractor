//! Win32 application shell: WinMain, message loop, WndProc, menu, toolbar,
//! status bar, tab control, and 3-pane layout.
// Integrates the FileTree and App state modules.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::Com::*;
use windows::Win32::Graphics::Gdi::*;

use crate::app::App;
use crate::editors::*;
use crate::file_tree::{FileTree, FileType};
use crate::spreadsheet::{Spreadsheet, Row};
use dispel_core::*;

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

const GWLP_USERDATA: i32 = -21;

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

        // Create and store App state
        let app = Box::new(App::new(hwnd));
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(app) as isize);

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Clean up app state
        let app_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if !app_ptr.is_null() {
            let _ = Box::from_raw(app_ptr as *mut App);
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
    let app_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    let app = if !app_ptr.is_null() {
        &mut *(app_ptr as *mut App)
    } else {
        &mut App::new(hwnd)
    };

    match msg {
        WM_CREATE => on_create(hwnd, app),
        WM_SIZE => on_size(hwnd),
        WM_COMMAND => on_command(hwnd, app, wparam),
        WM_NOTIFY => on_notify(hwnd, app, wparam, lparam),
        WM_CLOSE => on_close(hwnd),
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_PAINT => on_paint(hwnd),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn on_create(hwnd: HWND, app: &mut App) -> LRESULT {
    // Menu bar
    create_menu_bar(hwnd);

    // Toolbar
    create_toolbar(hwnd);

    // Status bar
    create_status_bar(hwnd);

    // Tab control (main content area placeholder)
    create_tab_control(hwnd);

    // Sidebar with file tree
    create_sidebar(hwnd, app);

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

unsafe fn on_command(hwnd: HWND, app: &mut App, wparam: WPARAM) -> LRESULT {
    let id = LOWORD(wparam.0) as u16;
    match id {
        ID_FILE_OPEN => on_file_open(hwnd, app),
        ID_FILE_SAVE => on_file_save(hwnd, app),
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

unsafe fn on_notify(hwnd: HWND, app: &mut App, _wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let nmhdr = &*(lparam.0 as *const NMHDR);
    match nmhdr.code {
        TCN_SELCHANGE => {
            // Tab selection changed
        }
        TVN_SELCHANGED => {
            // File tree selection changed
            if let Some(tree) = app.file_tree.as_ref() {
                let nmtv = &*(lparam.0 as *const NMTREEVIEWW);
                tree.on_sel_changed(nmtv.itemNew.hItem);
            }
        }
        TVN_ITEMEXPANDING => {
            // Lazy loading
            if let Some(tree) = app.file_tree.as_ref() {
                let nmtv = &*(lparam.0 as *const NMTREEVIEWW);
                tree.on_item_expanding(nmtv.itemNew.hItem);
            }
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

unsafe fn on_file_open(hwnd: HWND, app: &mut App) {
    let mut ofn = OPENFILENAMEW::default();
    let mut file_buf = [0u16; 260];
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = hwnd;
    ofn.lpstrFile = file_buf.as_mut_ptr();
    ofn.nMaxFile = file_buf.len() as u32;
    ofn.lpstrFilter = w!(
        "All Files\0*.*\0Dispel DB\0*.db\0Dispel INI\0*.ini\0Dispel REF\0*.ref\0Dispel SCR\0*.scr\0Dispel MAP\0*.map\0Dispel SPR\0*.spr\0Dispel SNF\0*.snf\0\0"
    );
    ofn.nFilterIndex = 1;
    ofn.Flags = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;

    if GetOpenFileNameW(&mut ofn).as_bool() {
        let path = wide_to_string(&file_buf);
        let path_buf = PathBuf::from(&path);
        app.set_status(&path);

        // Open file in appropriate editor based on extension
        if let Some(editor_type) = editor_type_for_path(&path_buf) {
            open_editor_for_type(hwnd, app, editor_type, &path_buf);
        }
    }
}

unsafe fn open_editor_for_type(hwnd: HWND, app: &mut App, editor_type: EditorTypeId, path: &PathBuf) {
    use std::io::Read;

    // Read raw file bytes for save round-tripping
    let raw_data = match std::fs::File::open(path).and_then(|mut f| {
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map(|_| buf)
    }) {
        Ok(data) => data,
        Err(_) => return,
    };

    let tab_id = app.next_tab_id;
    app.next_tab_id += 1;

    // Create spreadsheet with correct columns for the editor type
    let mut spreadsheet = match create_editor(editor_type, hwnd) {
        Ok(ss) => ss,
        Err(_) => return,
    };

    // Load data based on editor type
    let record_count = match editor_type {
        EditorTypeId::WeaponItem => {
            let items = WeaponItem::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(weapon_item_to_row).collect());
            count
        }
        EditorTypeId::Monster => {
            let items = Monster::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(monster_to_row).collect());
            count
        }
        EditorTypeId::HealItem => {
            let items = HealItem::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(heal_item_to_row).collect());
            count
        }
        EditorTypeId::MiscItem => {
            let items = MiscItem::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(misc_item_to_row).collect());
            count
        }
        EditorTypeId::EditItem => {
            let items = EditItem::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(edit_item_to_row).collect());
            count
        }
        EditorTypeId::EventItem => {
            let items = EventItem::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(event_item_to_row).collect());
            count
        }
        EditorTypeId::MagicSpell => {
            let items = MagicSpell::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(magic_spell_to_row).collect());
            count
        }
        EditorTypeId::ChData => {
            let items = ChData::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(chdata_to_row).collect());
            count
        }
        EditorTypeId::PartyIniNpc => {
            let items = PartyIniNpc::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(party_ini_npc_to_row).collect());
            count
        }
        EditorTypeId::MonsterRef => {
            let items = MonsterRef::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(monster_ref_to_row).collect());
            count
        }
        EditorTypeId::ExtraRef => {
            let items = ExtraRef::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(extra_ref_to_row).collect());
            count
        }
        EditorTypeId::MonsterIni => {
            let items = MonsterIni::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(monster_ini_to_row).collect());
            count
        }
        EditorTypeId::NpcIni => {
            let items = NpcIni::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(npc_ini_to_row).collect());
            count
        }
        EditorTypeId::EventIni => {
            let items = Event::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(event_ini_to_row).collect());
            count
        }
        EditorTypeId::ExtraIni => {
            let items = Extra::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(extra_ini_to_row).collect());
            count
        }
        EditorTypeId::MapIni => {
            let items = MapIni::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(map_ini_to_row).collect());
            count
        }
        EditorTypeId::WaveIni => {
            let items = WaveIni::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(wave_ini_to_row).collect());
            count
        }
        EditorTypeId::AllMapIni => {
            let items = Map::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(all_map_ini_to_row).collect());
            count
        }
        EditorTypeId::NpcRef => {
            let items = NPC::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(npc_ref_to_row).collect());
            count
        }
        EditorTypeId::PartyRef => {
            let items = PartyRef::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(party_ref_to_row).collect());
            count
        }
        EditorTypeId::DrawItem => {
            let items = DrawItem::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(draw_item_to_row).collect());
            count
        }
        EditorTypeId::Store => {
            let items = Store::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(store_to_row).collect());
            count
        }
        EditorTypeId::DialogueScript => {
            let items = DialogueScript::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(dialogue_script_to_row).collect());
            count
        }
        EditorTypeId::DialogueParagraph => {
            let items = DialogueParagraph::read_file(path).unwrap_or_default();
            let count = items.len();
            spreadsheet.load_rows(items.iter().map(dialogue_paragraph_to_row).collect());
            count
        }
        _ => 0,
    };

    spreadsheet.file_path = Some(path.clone());
    app.spreadsheets.insert(tab_id, spreadsheet);
    app.open_files.insert(tab_id, path.clone());
    app.editor_types.insert(tab_id, editor_type);
    app.original_file_data.insert(tab_id, raw_data);
    app.active_tab = Some(tab_id);
    app.set_status(&format!("Opened {} — {} records", path.file_name().unwrap_or_default().to_string_lossy(), record_count));
}

unsafe fn on_file_save(hwnd: HWND, app: &mut App) {
    let tab_id = match app.active_tab {
        Some(id) => id,
        None => { app.set_status("No active editor to save"); return; }
    };

    let path = match app.open_files.get(&tab_id) {
        Some(p) => p.clone(),
        None => { app.set_status("No file path for active tab"); return; }
    };

    let raw_data = match app.original_file_data.get(&tab_id) {
        Some(d) => d.clone(),
        None => { app.set_status("No original data to round-trip"); return; }
    };

    let editor_type = match app.editor_types.get(&tab_id) {
        Some(et) => *et,
        None => { app.set_status("Unknown editor type"); return; }
    };

    let rows: Vec<Row> = match app.spreadsheets.get(&tab_id) {
        Some(ss) => ss.rows().to_vec(),
        None => { app.set_status("No spreadsheet data"); return; }
    };

    let result = match editor_type {
        EditorTypeId::WeaponItem => save_weapon_items(&rows, &raw_data, &path),
        EditorTypeId::Monster => save_monsters(&rows, &raw_data, &path),
        EditorTypeId::HealItem => save_heal_items(&rows, &raw_data, &path),
        EditorTypeId::MiscItem => save_misc_items(&rows, &raw_data, &path),
        EditorTypeId::EditItem => save_edit_items(&rows, &raw_data, &path),
        EditorTypeId::EventItem => save_event_items(&rows, &raw_data, &path),
        EditorTypeId::MagicSpell => save_magic_spells(&rows, &raw_data, &path),
        EditorTypeId::ChData => save_chdata(&rows, &raw_data, &path),
        EditorTypeId::PartyIniNpc => save_party_ini_npcs(&rows, &raw_data, &path),
        EditorTypeId::MonsterRef => save_monster_refs(&rows, &raw_data, &path),
        EditorTypeId::ExtraRef => save_extra_refs(&rows, &raw_data, &path),
        EditorTypeId::MonsterIni => save_monster_ini(&rows, &raw_data, &path),
        EditorTypeId::NpcIni => save_npc_ini(&rows, &raw_data, &path),
        EditorTypeId::EventIni => save_event_ini(&rows, &raw_data, &path),
        EditorTypeId::ExtraIni => save_extra_ini(&rows, &raw_data, &path),
        EditorTypeId::MapIni => save_map_ini(&rows, &raw_data, &path),
        EditorTypeId::WaveIni => save_wave_ini(&rows, &raw_data, &path),
        EditorTypeId::AllMapIni => save_all_map_ini(&rows, &raw_data, &path),
        EditorTypeId::NpcRef => save_npc_refs(&rows, &raw_data, &path),
        EditorTypeId::PartyRef => save_party_refs(&rows, &raw_data, &path),
        EditorTypeId::DrawItem => save_draw_items(&rows, &raw_data, &path),
        EditorTypeId::Store => save_stores(&rows, &raw_data, &path),
        EditorTypeId::DialogueScript => save_dialogue_scripts(&rows, &raw_data, &path),
        EditorTypeId::DialogueParagraph => save_dialogue_paragraphs(&rows, &raw_data, &path),
        _ => {
            app.set_status("Save not yet implemented for this editor type");
            return;
        }
    };

    match result {
        Ok(()) => {
            // Update stored raw data to reflect saved state
            app.original_file_data.insert(tab_id, raw_data);
            app.set_status("Saved successfully");
        }
        Err(e) => {
            app.set_status(&format!("Save failed: {}", e));
        }
    }
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

fn create_sidebar(hwnd: HWND, app: &mut App) {
    unsafe {
        let tree = FileTree::new(hwnd).expect("Failed to create file tree");
        app.file_tree = Some(tree);
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
