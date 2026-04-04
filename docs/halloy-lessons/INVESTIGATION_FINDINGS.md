# Halloy IRC Client - Iced-RS UI Design Patterns Investigation

## Project Overview
Halloy is a modern IRC client built with Rust and Iced (iced-rs), a cross-platform GUI library. This investigation focuses on identifying reusable UI patterns for:
- Command picker (Ctrl+P)
- Moveable widget containers (pane system)
- Listings and selections
- Modals and overlays

---

## 1. Project Structure

### Core Architecture
```
src/
├── main.rs              # Application entry point
├── screen/              # Screen management
│   ├── dashboard.rs     # Main dashboard with pane grid
│   ├── help.rs          # Help screen
│   └── welcome.rs       # Welcome screen
├── widget/              # Custom widgets
│   ├── modal.rs         # Modal overlay system
│   ├── combo_box.rs     # Command picker / searchable dropdown
│   ├── pane_grid/       # Pane management (moveable containers)
│   └── ...              # Other custom widgets
├── buffer/              # Message buffers
└── modal/               # Modal dialogs
```

### Key Dependencies
```toml
iced = { version = "0.15.0-dev", features = [
    "wgpu", "tiny-skia", "tokio", "advanced", "image", "svg"
] }
```

---

## 2. Iced-RS Design Patterns Identified

### 2.1 Message-Driven Architecture (Elm Architecture)

**Pattern**: Iced follows the Elm Architecture pattern with:
- **State**: Application state stored in structs
- **Messages**: Enum variants representing events
- **Update**: State transition functions
- **View**: UI rendering functions

**Example from main.rs:**
```rust
pub enum Message {
    Modal(modal::Message),
    Pane(pane::Message),
    Buffer(buffer::Message),
    // ... other messages
}

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Modal(msg) => self.handle_modal(msg),
        Message::Pane(msg) => self.handle_pane(msg),
        // ...
    }
}
```

**Key Characteristics:**
- State is immutable between updates
- Messages are processed sequentially
- Tasks allow for async operations
- Clear separation of concerns

---

### 2.2 Modal System (Overlay Pattern)

**Location**: `src/widget/modal.rs`

**Pattern**: Modal overlay with backdrop and centered content

**Key Components:**

```rust
// Modal widget
pub fn modal<'a, Message, Theme, Renderer>(
    base: Element<'a, Message, Theme, Renderer>,
    modal: Element<'a, Message, Theme, Renderer>,
    on_blur: impl Fn() -> Message + 'a,
    backdrop_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
```

**Usage Pattern:**
```rust
// In main.rs
self.modal = Some(Modal::ServerConnect { config, window });

// In view
widget::modal(
    base_content,  // Background content
    modal_content, // Modal overlay
    || Message::Modal(modal::Message::Cancel),
    0.7,           // Backdrop opacity
)
```

**Reusable Components:**
- `Modal::new()` - Creates modal instance
- `overlay::Element` - Handles rendering
- Escape key handling
- Click outside to close
- Backdrop with configurable opacity

**Design Pattern**: **Overlay Pattern** - Modal is rendered as an overlay on top of existing content

---

### 2.3 Pane Grid System (Moveable Widget Containers)

**Location**: `src/screen/dashboard.rs` and `src/screen/dashboard/pane.rs`

**Pattern**: Split-pane layout with draggable/resizable panes

**Key Components:**

```rust
use iced::widget::pane_grid::{self, PaneGrid};

// State management
pub struct Dashboard {
    panes: pane_grid::State<Pane>,  // Main pane grid
    popout: HashMap<window::Id, pane_grid::State<Pane>>,  // Popout windows
}

// Pane definition
pub struct Pane {
    pub buffer: Buffer,
    pub size: Size,
    title_bar: TitleBar,
    pub modal: Option<super::modal::Modal>,
}
```

**Pane Operations:**
- `new_pane(axis)` - Split current pane
- `split_pane(axis)` - Split specific pane
- `close_pane()` - Remove pane
- `maximize_pane()` - Toggle maximize
- `merge()` - Merge panes

**Message Handling:**
```rust
pub enum Message {
    PaneClicked(pane_grid::Pane),
    PaneResized(pane_grid::ResizeEvent),
    PaneDragged(pane_grid::DragEvent),
    SplitPane(pane_grid::Axis),
    ClosePane,
    MaximizePane,
    // ...
}
```

**Design Pattern**: **Split-Pane Layout with Drag-and-Drop**
- Uses Iced's built-in `pane_grid` widget
- Supports horizontal and vertical splits
- Drag events for reordering
- Resize events for dimension changes

**Reusable Components:**
- `pane_grid::State` - State management
- `PaneGrid::new()` - Rendering
- `pane_grid::DragEvent` - Drag handling
- `pane_grid::ResizeEvent` - Resize handling

---

### 2.4 Command Picker / ComboBox (Searchable Dropdown)

**Location**: `src/widget/combo_box.rs` and `src/buffer/input_view/completion.rs`

**Pattern**: Searchable dropdown with fuzzy matching

**Key Components:**

```rust
// ComboBox widget
pub struct ComboBox<'a, T, Message, Theme = crate::Theme, Renderer = super::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    state: &'a State<T>,
    text_input: TextInput<'a, TextInputEvent, Theme, Renderer>,
    // ...
}

// State management
pub struct State<T> {
    options: Vec<T>,
    filtered: Vec<T>,
    input_value: String,
    is_open: bool,
    // ...
}
```

**Usage in Command Picker:**
```rust
// In buffer/input_view.rs
let combo_box = combo_box::combo_box(
    &self.command_state,
    "/",
    None,
    Message::CommandSelected,
)
.on_input(Message::CommandInputChanged)
.menu_class(theme::combo_box::menu(theme));
```

**Fuzzy Matching:**
```rust
// Uses nucleo-matcher for efficient fuzzy search
use nucleo_matcher::{Config, Matcher, Utf32Str};

let matcher = Matcher::new(Config::DEFAULT);
let pattern = Pattern::new(input, CaseMatching::Smart, Normalization::Smart, AtomKind::Fuzzy);
let matches = matcher.fuzzy_match(&options, pattern);
```

**Design Pattern**: **Searchable Dropdown with Fuzzy Matching**

**Reusable Components:**
- `ComboBox::new()` - Create picker
- `State::new()` - Manage state
- `on_input()` - Handle text changes
- `on_selected()` - Handle selection
- Fuzzy matching with `nucleo-matcher`

---

### 2.5 Listing and Selection Patterns

**Location**: Multiple files including `src/widget/combo_box.rs` and custom list implementations

**Pattern**: Generic listing with selection and keyboard navigation

**Key Components:**

```rust
// In combo_box.rs
impl<'a, T, Message, Theme, Renderer> ComboBox<'a, T, Message, Theme, Renderer>
where
    T: std::fmt::Display + Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    // ... methods for selection
    pub fn selected(&self) -> Option<&T> { /* ... */ }
    pub fn focus(&mut self) { /* ... */ }
    pub fn unfocus(&mut self) { /* ... */ }
}
```

**Keyboard Navigation:**
- Arrow keys for navigation
- Enter for selection
- Escape for closing
- Tab for focus management

**Design Pattern**: **Generic List Component with Keyboard Navigation**

---

### 2.6 Context Menu Pattern

**Location**: `src/widget/context_menu.rs`

**Pattern**: Right-click context menus with submenus

**Key Components:**

```rust
pub struct ContextMenu<'a, Message, Theme = crate::Theme, Renderer = super::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    // ...
}

pub enum Event {
    Opened,
    Closed,
    ItemSelected(usize),
}
```

**Usage:**
```rust
let context_menu = context_menu::ContextMenu::new(
    trigger_element,
    menu_items,
    Message::ContextMenuEvent,
)
.offset(offset)
.placement(Placement::Bottom);
```

**Design Pattern**: **Overlay Menu with Positioning**

---

## 3. Reusable UI Patterns for Your Project

### 3.1 Command Picker System

**Files to Reuse:**
- `src/widget/combo_box.rs` - Core combo box widget
- `src/buffer/input_view/completion.rs` - Command processing logic
- `src/widget.rs` - Exports and utilities

**Key Features to Replicate:**
1. **Fuzzy Matching**: Use `nucleo-matcher` for efficient search
2. **State Management**: `State<T>` struct for tracking options
3. **Keyboard Navigation**: Arrow keys, Enter, Escape
4. **Custom Styling**: Theme integration
5. **Message Passing**: Elm-style message handling

**Minimal Implementation:**
```rust
// 1. Define your message types
pub enum Message {
    CommandPickerInput(String),
    CommandPickerSelected(String),
    CommandPickerClosed,
}

// 2. Create combo box
let combo_box = combo_box::combo_box(
    &state,
    "> ",
    None,
    Message::CommandPickerSelected,
)
.on_input(Message::CommandPickerInput)
.menu_class(your_theme::combo_box_menu());
```

---

### 3.2 Moveable Widget Containers

**Files to Reuse:**
- `src/screen/dashboard.rs` - Main dashboard with pane grid
- `src/screen/dashboard/pane.rs` - Pane management
- `src/widget.rs` - Pane grid exports

**Key Features to Replicate:**
1. **Pane Grid**: `pane_grid::State` and `PaneGrid` widget
2. **Drag Events**: `PaneDragged(pane_grid::DragEvent)`
3. **Resize Events**: `PaneResized(pane_grid::ResizeEvent)`
4. **Pane Operations**: Split, close, maximize
5. **State Management**: Track pane configurations

**Minimal Implementation:**
```rust
use iced::widget::pane_grid::{self, PaneGrid};

pub struct State {
    panes: pane_grid::State<PaneContent>,
}

pub enum Message {
    SplitPane(pane_grid::Axis),
    ClosePane,
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
}

// In view
let pane_grid = PaneGrid::new(&state.panes, |pane_id, content, maximized| {
    // Render each pane
    pane_content_view(content, maximized)
});
```

---

### 3.3 Modal System

**Files to Reuse:**
- `src/widget/modal.rs` - Core modal widget
- `src/modal.rs` - Modal state management
- `src/screen/dashboard/modal.rs` - Dashboard-specific modals

**Key Features to Replicate:**
1. **Modal Widget**: `modal()` function for creating modals
2. **Backdrop**: Configurable opacity
3. **Escape Handling**: Auto-close on Escape
4. **Click Outside**: Close on backdrop click
5. **Positioning**: Centered modals

**Minimal Implementation:**
```rust
use crate::widget::modal;

// In your app state
pub enum Message {
    OpenModal,
    CloseModal,
    ModalAction(ModalAction),
}

// In view
let content = your_main_content();

if let Some(modal_content) = &self.modal {
    modal::modal(
        content,
        modal_content,
        || Message::CloseModal,
        0.7, // backdrop alpha
    )
} else {
    content
}
```

---

### 3.4 Generic Listing Component

**Pattern**: Create a reusable list component similar to `ComboBox`

**Key Features:**
- Keyboard navigation
- Selection tracking
- Custom rendering per item
- Scrollable content

**Example Structure:**
```rust
pub struct List<'a, T, Message> {
    items: &'a [T],
    selected: Option<usize>,
    on_selected: Box<dyn Fn(T) -> Message>,
    // ... other fields
}

impl<T: Clone> List<'_, T, Message> {
    pub fn new(
        items: &'a [T],
        selected: Option<usize>,
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self { /* ... */ }
    
    pub fn select(&mut self, index: usize) -> Option<T> { /* ... */ }
    pub fn focus(&mut self) { /* ... */ }
}
```

---

## 4. Iced-RS Best Practices Observed

### 4.1 Widget Composition
- **Use `Element` type**: `type Element<'a, Message> = iced::Element<'a, Message, Theme, Renderer>`
- **Builder pattern**: Fluent API for widget configuration
- **Theme integration**: Custom themes per widget

### 4.2 State Management
- **Separate state**: Keep widget state separate from app state
- **Message passing**: All interactions via messages
- **Immutable updates**: State is immutable between updates

### 4.3 Rendering
- **Lazy evaluation**: Widgets only render when needed
- **Layout caching**: Iced handles layout optimization
- **Custom renderers**: Use `Renderer` trait for custom drawing

### 4.4 Event Handling
- **Event filtering**: Match specific events
- **Cursor tracking**: Handle mouse interactions
- **Keyboard shortcuts**: Global and local key bindings

---

## 5. Integration Patterns

### 5.1 Combining Patterns

**Example: Command Picker in Moveable Pane**
```rust
// Create a pane with command picker
let pane = Pane::new(CommandPickerPane::new());

// In pane view
PaneGrid::new(state, |pane_id, pane, maximized| {
    match pane {
        Pane::CommandPicker(cmd_picker) => {
            cmd_picker.view(theme).map(Message::CommandPicker)
        }
        // ... other pane types
    }
})
```

### 5.2 Theme Integration

**Custom Theme Example:**
```rust
// In your theme.rs
impl combo_box::Catalog for Theme {
    type Class<'a> = combo_box::Class<'a>;
    
    fn default_input(&self) -> Self::Class<'_> {
        combo_box::Class::default()
            .input(self.button_primary_style())
    }
    
    fn default_menu(&self) -> Self::Class<'_> {
        combo_box::Class::default()
            .menu(self.context_menu_style())
    }
}
```

---

## 6. Next Steps for Your Project

### 6.1 Recommended Files to Extract

1. **Command Picker System**
   - `src/widget/combo_box.rs` (Core widget)
   - `src/widget.rs` (Exports)
   - `src/theme.rs` (Theme integration)

2. **Moveable Panes**
   - `src/screen/dashboard.rs` (Main dashboard)
   - `src/widget.rs` (Pane grid exports)
   - `src/theme/pane_grid.rs` (Styling)

3. **Modal System**
   - `src/widget/modal.rs` (Core widget)
   - `src/modal.rs` (State management)

### 6.2 Customization Points

- **Styling**: Implement `Catalog` traits for your theme
- **Behavior**: Modify message handling
- **Content**: Replace buffer-specific logic with your content
- **Shortcuts**: Customize keyboard bindings

### 6.3 Dependencies to Include

```toml
[dependencies]
iced = { version = "0.15", features = ["advanced", "wgpu"] }
nucleo-matcher = "0.3"  # For fuzzy matching
```

---

## 7. Architecture Decision Records

### 7.1 Why Iced-RS?
- **Cross-platform**: Windows, macOS, Linux
- **GPU-accelerated**: Good performance
- **Rust-native**: Memory safety
- **Declarative**: Similar to React/Elm

### 7.2 Pattern Tradeoffs

| Pattern | Pros | Cons |
|---------|------|------|
| Elm Architecture | Predictable, testable | Boilerplate |
| Message Passing | Clear data flow | Indirection |
| Overlays | Clean modal system | Z-index management |
| Custom Widgets | Reusable | Learning curve |

### 7.3 Alternatives Considered
- **egui**: Immediate mode, simpler but less structured
- **gtk-rs**: More mature but complex
- **druid**: Discontinued
- **Slint**: Different paradigm

---

## 8. Testing Patterns

**Unit Tests**:
- Message handling
- State transitions
- Rendering logic

**Integration Tests**:
- Message flow
- Event handling
- State consistency

**Visual Tests**:
- Layout correctness
- Theme application
- Responsive design

---

## 9. Performance Considerations

### 9.1 Optimization Techniques
- **Layout caching**: Iced handles this automatically
- **Debounced input**: For search/filter operations
- **Virtual scrolling**: For large lists
- **Lazy rendering**: Only render visible items

### 9.2 Memory Management
- **Clone carefully**: Avoid unnecessary cloning
- **Reference counting**: Use `Rc` or `Arc` for shared state
- **Drop unused resources**: Clean up when done

---

## 10. Future Enhancements

### 10.1 Potential Improvements
1. **Accessibility**: Screen reader support
2. **Animations**: Smooth transitions
3. **Theming**: Dark/light mode
4. **Plugins**: Extensible architecture
5. **Multi-window**: Better window management

### 10.2 Advanced Patterns
- **Undo/Redo**: Command pattern
- **Drag-and-drop**: More sophisticated
- **Responsive design**: Adaptive layouts
- **Internationalization**: Localization support

---

## Summary

Halloy demonstrates several reusable UI patterns with Iced-RS:

1. **Message-driven architecture** - Elm-style state management
2. **Modal overlays** - Clean dialog system
3. **Moveable panes** - Split-pane layout with drag-and-drop
4. **Command picker** - Searchable dropdown with fuzzy matching
5. **Context menus** - Right-click interactions

**Key Takeaways:**
- Use `Element` type for widget composition
- Follow Elm architecture for state management
- Leverage Iced's built-in widgets (pane_grid, modal, etc.)
- Implement custom themes via `Catalog` traits
- Use message passing for clear data flow

**Recommended starting point**: Extract `combo_box.rs` and `modal.rs` first, then build your command picker and modal system around them.
