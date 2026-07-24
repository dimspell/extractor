//! File tree: TreeView control for game file browsing with directory scanning,
//! file type icons, lazy loading, and selection handling.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Storage::FileSystem::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

const IDM_FILETREE_FILTER_DB: u16 = 3001;
const IDM_FILETREE_FILTER_INI: u16 = 3002;
const IDM_FILETREE_FILTER_REF: u16 = 3003;
const IDM_FILETREE_FILTER_SCR: u16 = 3004;
const IDM_FILETREE_FILTER_ALL: u16 = 3005;

/// File type categories for icon selection and filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Database,
    Ini,
    Reference,
    Script,
    Map,
    Sprite,
    Audio,
    Save,
    Other,
}

impl FileType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "db" => FileType::Database,
            "ini" => FileType::Ini,
            "ref" => FileType::Reference,
            "scr" | "dlg" | "pgp" => FileType::Script,
            "map" => FileType::Map,
            "spr" => FileType::Sprite,
            "snf" => FileType::Audio,
            "sav" | "ifo" => FileType::Save,
            "gtl" | "btl" => FileType::Map,
            _ => FileType::Other,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            FileType::Database => "*.db",
            FileType::Ini => "*.ini",
            FileType::Reference => "*.ref",
            FileType::Script => "*.scr",
            FileType::Map => "*.map",
            FileType::Sprite => "*.spr",
            FileType::Audio => "*.snf",
            FileType::Save => "*.sav",
            FileType::Other => "*.*",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            FileType::Database => "Database files",
            FileType::Ini => "INI config files",
            FileType::Reference => "Reference files",
            FileType::Script => "Script files",
            FileType::Map => "Map files",
            FileType::Sprite => "Sprite files",
            FileType::Audio => "Audio files",
            FileType::Save => "Save files",
            FileType::Other => "All files",
        }
    }
}

/// File tree item data stored in TreeView node lParam.
#[derive(Debug)]
pub struct TreeItemData {
    pub path: PathBuf,
    pub is_directory: bool,
    pub file_type: Option<FileType>,
    pub expanded: bool,
}

/// The file tree control wrapper.
pub struct FileTree {
    pub hwnd: HWND,
    pub image_list: HIMAGELIST,
    pub selected_path: Option<PathBuf>,
    pub filter: Option<FileType>,
    pub on_file_selected: Option<Box<dyn Fn(&Path)>>,
}

impl FileTree {
    pub fn new(parent: HWND) -> Result<Self> {
        unsafe {
            // Create TreeView control
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("SysTreeView32"),
                None,
                WS_CHILD | WS_VISIBLE | TVS_HASBUTTONS | TVS_HASLINES
                    | TVS_LINESATROOT | TVS_SHOWSELALWAYS | TVS_DISABLEDRAGDROP
                    | TVS_TRACKSELECT,
                0, 0, 0, 0,
                parent,
                None,
                GetModuleHandleW(None)?,
                None,
            );

            // Create image list for file type icons
            let image_list = ImageList_Create(16, 16, ILC_COLOR32 | ILC_MASK, 4, 4);

            // Add default icons
            let default_icon = LoadIconW(None, IDI_APPLICATION)?;
            ImageList_AddIcon(image_list, default_icon)?;

            let folder_icon = LoadIconW(None, IDI_FOLDER)?;
            ImageList_AddIcon(image_list, folder_icon)?;

            let db_icon = LoadIconW(None, IDI_APPLICATION)?;
            ImageList_AddIcon(image_list, db_icon)?;

            let ini_icon = LoadIconW(None, IDI_APPLICATION)?;
            ImageList_AddIcon(image_list, ini_icon)?;

            let script_icon = LoadIconW(None, IDI_APPLICATION)?;
            ImageList_AddIcon(image_list, script_icon)?;

            let map_icon = LoadIconW(None, IDI_APPLICATION)?;
            ImageList_AddIcon(image_list, map_icon)?;

            let sprite_icon = LoadIconW(None, IDI_APPLICATION)?;
            ImageList_AddIcon(image_list, sprite_icon)?;

            let audio_icon = LoadIconW(None, IDI_APPLICATION)?;
            ImageList_AddIcon(image_list, audio_icon)?;

            let save_icon = LoadIconW(None, IDI_APPLICATION)?;
            ImageList_AddIcon(image_list, save_icon)?;

            let other_icon = LoadIconW(None, IDI_APPLICATION)?;
            ImageList_AddIcon(image_list, other_icon)?;

            TreeView_SetImageList(hwnd, image_list, TVSIL_NORMAL);

            Ok(Self {
                hwnd,
                image_list,
                selected_path: None,
                filter: None,
                on_file_selected: None,
            })
        }
    }

    /// Populate the tree with a root directory.
    pub fn populate(&self, root_path: &Path) -> Result<()> {
        unsafe {
            TreeView_DeleteAllItems(self.hwnd);

            let root_item = TreeView_InsertItemW(
                self.hwnd,
                &TVINSERTSTRUCTW {
                    hParent: HTREEITEM(0),
                    hInsertAfter: TVI_LAST,
                    item: TVITEMW {
                        mask: TVIF_TEXT | TVIF_IMAGE | TVIF_SELECTEDIMAGE | TVIF_PARAM,
                        iImage: 1, // folder icon
                        iSelectedImage: 1,
                        pszText: wide_from_os(root_path.file_name().unwrap_or_default())?.as_ptr() as PCWSTR,
                        lParam: 0,
                        ..Default::default()
                    },
                },
            );

            if !root_item.0.is_null() {
                // Store root path in item data
                let data = Box::new(TreeItemData {
                    path: root_path.to_path_buf(),
                    is_directory: true,
                    file_type: None,
                    expanded: false,
                });
                TreeView_SetItemParam(self.hwnd, root_item, data as usize as LPARAM);

                // Add a placeholder child to enable lazy loading
                self.add_placeholder(root_item)?;
            }

            Ok(())
        }
    }

    fn add_placeholder(&self, parent: HTREEITEM) -> Result<()> {
        unsafe {
            let placeholder = TreeView_InsertItemW(
                self.hwnd,
                &TVINSERTSTRUCTW {
                    hParent: parent,
                    hInsertAfter: TVI_LAST,
                    item: TVITEMW {
                        mask: TVIF_TEXT,
                        pszText: PCWSTR(w!("Loading...").as_ptr()),
                        ..Default::default()
                    },
                },
            );
            Ok(())
        }
    }

    /// Remove placeholder and load actual children.
    pub fn expand_node(&self, item: HTREEITEM) -> Result<()> {
        unsafe {
            // Remove placeholder children
            let child = TreeView_GetChild(self.hwnd, item);
            while !child.0.is_null() {
                let next = TreeView_GetNextSibling(self.hwnd, child);
                TreeView_DeleteItem(self.hwnd, child);
                if next.0.is_null() {
                    break;
                }
            }

            // Get item data
            let mut tvitem = TVITEMW {
                mask: TVIF_PARAM,
                hItem: item,
                ..Default::default()
            };
            TreeView_GetItem(self.hwnd, &mut tvitem);
            let data_ptr = tvitem.lParam as *mut TreeItemData;
            if data_ptr.is_null() {
                return Ok(());
            }
            let data = &*data_ptr;

            if !data.is_directory {
                return Ok(());
            }

            // Read directory entries
            let entries = self.read_directory(&data.path)?;

            // Separate directories and files
            let mut dirs: Vec<PathBuf> = Vec::new();
            let mut files: Vec<PathBuf> = Vec::new();

            for entry in &entries {
                if entry.is_dir() {
                    dirs.push(entry.clone());
                } else {
                    files.push(entry.clone());
                }
            }

            // Sort directories first, then files
            dirs.sort();
            files.sort();

            // Insert directories
            for dir in &dirs {
                let name = dir.file_name().unwrap_or_default().to_string_lossy();
                let wide = wide_from_os(&name)?;
                let child_item = TreeView_InsertItemW(
                    self.hwnd,
                    &TVINSERTSTRUCTW {
                        hParent: item,
                        hInsertAfter: TVI_LAST,
                        item: TVITEMW {
                            mask: TVIF_TEXT | TVIF_IMAGE | TVIF_SELECTEDIMAGE | TVIF_PARAM,
                            iImage: 1, // folder icon
                            iSelectedImage: 1,
                            pszText: wide.as_ptr() as PCWSTR,
                            lParam: 0,
                            ..Default::default()
                        },
                    },
                );

                if !child_item.0.is_null() {
                    let child_data = Box::new(TreeItemData {
                        path: dir.clone(),
                        is_directory: true,
                        file_type: None,
                        expanded: false,
                    });
                    TreeView_SetItemParam(self.hwnd, child_item, Box::into_raw(child_data) as usize as LPARAM);

                    // Add placeholder for lazy loading
                    self.add_placeholder(child_item)?;
                }
            }

            // Insert files (respecting filter)
            for file in &files {
                let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
                let file_type = FileType::from_extension(ext);

                // Apply filter
                if let Some(filter) = self.filter {
                    if file_type != filter && file_type != FileType::Other {
                        continue;
                    }
                }

                let name = file.file_name().unwrap_or_default().to_string_lossy();
                let wide = wide_from_os(&name)?;
                let icon_index = self.icon_for_file_type(&file_type);

                let child_item = TreeView_InsertItemW(
                    self.hwnd,
                    &TVINSERTSTRUCTW {
                        hParent: item,
                        hInsertAfter: TVI_LAST,
                        item: TVITEMW {
                            mask: TVIF_TEXT | TVIF_IMAGE | TVIF_SELECTEDIMAGE | TVIF_PARAM,
                            iImage: icon_index,
                            iSelectedImage: icon_index,
                            pszText: wide.as_ptr() as PCWSTR,
                            lParam: 0,
                            ..Default::default()
                        },
                    },
                );

                if !child_item.0.is_null() {
                    let child_data = Box::new(TreeItemData {
                        path: file.clone(),
                        is_directory: false,
                        file_type: Some(file_type),
                        expanded: false,
                    });
                    TreeView_SetItemParam(self.hwnd, child_item, Box::into_raw(child_data) as usize as LPARAM);
                }
            }

            // Mark as expanded
            let mut tvitem = TVITEMW {
                mask: TVIF_PARAM,
                hItem: item,
                ..Default::default()
            };
            TreeView_GetItem(self.hwnd, &mut tvitem);
            let data_ptr = tvitem.lParam as *mut TreeItemData;
            if !data_ptr.is_null() {
                (*data_ptr).expanded = true;
            }

            Ok(())
        }
    }

    fn read_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        if let Ok(read_dir) = std::fs::read_dir(path) {
            for entry in read_dir.flatten() {
                entries.push(entry.path());
            }
        }
        Ok(entries)
    }

    fn icon_for_file_type(&self, file_type: &FileType) -> i32 {
        match file_type {
            FileType::Database => 2,
            FileType::Ini => 3,
            FileType::Reference => 2,
            FileType::Script => 4,
            FileType::Map => 5,
            FileType::Sprite => 6,
            FileType::Audio => 7,
            FileType::Save => 8,
            FileType::Other => 9,
        }
    }

    /// Set a file type filter.
    pub fn set_filter(&mut self, filter: Option<FileType>) {
        self.filter = filter;
    }

    /// Get the currently selected file path.
    pub fn selected_path(&self) -> Option<PathBuf> {
        self.selected_path.clone()
    }

    /// Handle TVN_SELCHANGED notification.
    pub fn on_sel_changed(&mut self, item: HTREEITEM) {
        unsafe {
            let mut tvitem = TVITEMW {
                mask: TVIF_PARAM,
                hItem: item,
                ..Default::default()
            };
            TreeView_GetItem(self.hwnd, &mut tvitem);

            let data_ptr = tvitem.lParam as *mut TreeItemData;
            if data_ptr.is_null() {
                self.selected_path = None;
                return;
            }

            let data = &*data_ptr;
            self.selected_path = Some(data.path.clone());

            // Only open files, not directories
            if !data.is_directory {
                if let Some(ref callback) = self.on_file_selected {
                    callback(&data.path);
                }
            }
        }
    }

    /// Handle TVN_ITEMEXPANDING for lazy loading.
    pub fn on_item_expanding(&self, item: HTREEITEM) -> bool {
        unsafe {
            let mut tvitem = TVITEMW {
                mask: TVIF_PARAM | TVIF_STATE,
                hItem: item,
                stateMask: TVIS_EXPANDEDONCE,
                ..Default::default()
            };
            TreeView_GetItem(self.hwnd, &mut tvitem);

            // If not yet expanded once, populate children
            if tvitem.state & TVIS_EXPANDEDONCE == 0 {
                // Check if this is a placeholder node
                let child = TreeView_GetChild(self.hwnd, item);
                if !child.0.is_null() {
                    let mut child_item = TVITEMW {
                        mask: TVIF_TEXT,
                        hItem: child,
                        ..Default::default()
                    };
                    TreeView_GetItem(self.hwnd, &mut child_item);

                    let text = wide_to_string(child_item.pszText);
                    if text == "Loading..." {
                        // This is a placeholder - expand it
                        let _ = self.expand_node(item);
                        return true; // Allow expansion
                    }
                }
            }

            false
        }
    }
}

fn wide_from_os(s: &std::ffi::OsStr) -> Result<Vec<u16>> {
    use std::os::windows::ffi::OsStrExt;
    let mut v: Vec<u16> = s.encode_wide().collect();
    v.push(0);
    Ok(v)
}

fn wide_to_string(wide: &[u16]) -> String {
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..len])
}

/// Recursively scan a directory for game files.
pub fn scan_game_files(root: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    scan_directory(root, 0, max_depth, &mut results);
    results
}

fn scan_directory(dir: &Path, depth: usize, max_depth: usize, results: &mut Vec<PathBuf>) {
    if depth > max_depth {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_directory(&path, depth + 1, max_depth, results);
            } else if is_game_file(&path) {
                results.push(path);
            }
        }
    }
}

fn is_game_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "db" | "ini" | "ref" | "scr" | "dlg" | "pgp" | "map" | "spr" | "snf" | "gtl" | "btl" | "sav" | "ifo"
        ),
        None => false,
    }
}
