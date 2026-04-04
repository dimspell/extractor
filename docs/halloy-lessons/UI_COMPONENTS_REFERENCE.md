# Halloy UI Components Reference - Detailed Breakdown

This document provides a deep dive into the specific UI components used in Halloy, focusing on the patterns you can reuse in your project.

---

## Table of Contents

1. [Command Picker System](#1-command-picker-system)
   - [ComboBox Widget](#combobox-widget)
   - [Command Processing](#command-processing)
   - [Fuzzy Matching](#fuzzy-matching)
   - [Keyboard Navigation](#keyboard-navigation)

2. [Moveable Widget Containers](#2-moveable-widget-containers)
   - [Pane Grid System](#pane-grid-system)
   - [Pane State Management](#pane-state-management)
   - [Drag and Drop](#drag-and-drop)
   - [Resize Operations](#resize-operations)

3. [Modal System](#3-modal-system)
   - [Modal Widget](#modal-widget)
   - [Overlay Pattern](#overlay-pattern)
   - [Backdrop Handling](#backdrop-handling)
   - [Escape Key Handling](#escape-key-handling)

4. [Listing Components](#4-listing-components)
   - [Generic List Pattern](#generic-list-pattern)
   - [Selection Tracking](#selection-tracking)
   - [Keyboard Navigation](#keyboard-navigation-1)

5. [Context Menu System](#5-context-menu-system)
   - [Right-click Menus](#right-click-menus)
   - [Submenus](#submenus)
   - [Positioning](#positioning)

6. [Integration Examples](#6-integration-examples)
   - [Command Picker in Pane](#command-picker-in-pane)
   - [Modal with Context Menu](#modal-with-context-menu)
   - [Responsive Layouts](#responsive-layouts)

7. [Theme Integration](#7-theme-integration)
   - [Custom Themes](#custom-themes)
   - [Styling Hooks](#styling-hooks)
   - [Color Schemes](#color-schemes)

8. [Performance Considerations](#8-performance-considerations)
   - [Rendering Optimization](#rendering-optimization)
   - [Event Handling](#event-handling)
   - [State Management](#state-management)

---

## 1. Command Picker System

The command picker is a searchable dropdown that appears when the user presses Ctrl+P (or similar shortcut). It allows users to search through available commands and select one.

### 1.1 ComboBox Widget

**File**: `fixtures/halloy/src/widget/combo_box.rs`

**Core Structure**:
```rust
pub struct ComboBox<'a, T, Message, Theme = crate::Theme, Renderer = super::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    state: &'a State<T>,           // State management
    text_input: TextInput<'a, TextInputEvent, Theme, Renderer>, // Input field
    selection: text_input::Value,  // Current selection text
    on_selected: Box<dyn Fn(T) -> Message>, // Selection handler
    on_option_hovered: Option<Box<dyn Fn(T) -> Message>>, // Hover handler
    on_close: Option<Message>,     // Close handler
    on_input: Option<Box<dyn Fn(String) -> Message>>, // Input handler
    menu_class: <Theme as menu::Catalog>::Class<'a>, // Menu styling
    padding: Padding,              // Padding settings
    size: Option<f32>,             // Font size
}
```

**Key Methods**:

```rust
impl<'a, T, Message, Theme, Renderer> ComboBox<'a, T, Message, Theme, Renderer>
where
    T: std::fmt::Display + Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new ComboBox
    pub fn new(
        state: &'a State<T>,
        placeholder: &str,
        selection: Option<&T>,
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self { /* ... */ }
    
    /// Sets the message for text input changes
    pub fn on_input(
        mut self,
        on_input: impl Fn(String) -> Message + 'static,
    ) -> Self { /* ... */ }
    
    /// Sets the message for option hover
    pub fn on_option_hovered(
        mut self,
        on_selection: impl Fn(T) -> Message + 'static,
    ) -> Self { /* ... */ }
    
    /// Sets the message when the menu closes
    pub fn on_close(
        mut self,
        on_close: impl Fn() -> Message + 'static,
    ) -> Self { /* ... */ }
    
    /// Sets the menu style class
    pub fn menu_class(
        mut self,
        menu_class: impl Into<<Theme as menu::Catalog>::Class<'a>>,
    ) -> Self { /* ... */ }
    
    /// Sets the padding
    pub fn padding(mut self, padding: Padding) -> Self { /* ... */ }
    
    /// Sets the font size
    pub fn size(mut self, size: f32) -> Self { /* ... */ }
}
```

**Usage Example**:
```rust
let combo_box = combo_box::combo_box(
    &state,
    "/",
    None,
    Message::CommandSelected,
)
.on_input(Message::CommandInputChanged)
.menu_class(theme::combo_box::menu(theme));
```

### 1.2 State Management

**File**: `fixtures/halloy/src/widget/combo_box.rs` (State implementation)

**State Structure**:
```rust
pub struct State<T> {
    options: Vec<T>,           // All available options
    filtered: Vec<T>,          // Currently filtered options
    input_value: String,       // Current input text
    is_open: bool,             // Is the dropdown open?
    focused_index: Option<usize>, // Currently focused option
    scroll_offset: f32,        // Scroll position
    text_input_state: text_input::State, // Text input state
}
```

**State Methods**:

```rust
impl<T> State<T> {
    /// Creates a new State with options
    pub fn new(options: Vec<T>) -> Self { /* ... */ }
    
    /// Filters options based on input
    pub fn filter(&mut self, input: &str) { /* ... */ }
    
    /// Gets the current value to display
    pub fn value(&self) -> &str { /* ... */ }
    
    /// Focuses the combo box
    pub fn focus(&mut self) { /* ... */ }
    
    /// Unfocuses the combo box
    pub fn unfocus(&mut self) { /* ... */ }
    
    /// Selects an option by index
    pub fn select(&mut self, index: usize) -> Option<T> { /* ... */ }
    
    /// Clears the selection
    pub fn clear(&mut self) { /* ... */ }
    
    /// Toggles the dropdown open/closed
    pub fn toggle(&mut self) { /* ... */ }
}
```

**Example State Creation**:
```rust
let command_state = combo_box::State::new(
    vec![
        "/join #channel",
        "/part #channel",
        "/query nickname",
        "/msg nickname message",
    ]
);
```

### 1.3 Command Processing

**File**: `fixtures/halloy/src/buffer/input_view/completion.rs`

**Command Processing Logic**:

```rust
#[derive(Debug, Clone, Default)]
pub struct Completion {
    commands: Commands,    // Command-specific state
    text: Text,            // Text completion state
    emojis: Emojis,        // Emoji completion state
}

impl Completion {
    /// Process input and update completion state
    pub fn process<'a>(
        &mut self,
        input: &str,
        cursor_position: usize,
        // ... other parameters
        config: &Config,
    ) {
        let is_command = input.starts_with('/');
        
        if is_command {
            self.commands.process(
                input,
                // ... other parameters
                config,
            );
            
            // Disallow other completions when selecting a command
            if matches!(self.commands, Commands::Selecting { .. }) {
                self.text = Text::default();
                self.emojis = Emojis::default();
                return;
            }
        } else {
            self.commands = Commands::default();
        }
        
        // Handle emoji picker
        if let Some(shortcode) = config.buffer.emojis.show_picker
            .then(|| get_word(input, cursor_position))
            .flatten()
            .filter(|word| word.starts_with(':'))
        {
            self.emojis.process(shortcode, config);
            self.text = Text::default();
        } else {
            // Handle text completion
            self.text.process(
                input,
                cursor_position,
                // ... other parameters
                config,
            );
            self.emojis = Emojis::default();
        }
    }
    
    /// Select the current completion
    pub fn select(&mut self, config: &Config) -> Option<Entry> {
        self.commands
            .select()
            .map(Entry::Command)
            .or(self.emojis.select(config).map(Entry::Emoji))
    }
}
```

**Command State Machine**:
```rust
pub enum Commands {
    Default,
    Selecting {
        selected: usize,
        commands: Vec<String>,
    },
    // ... other states
}

impl Commands {
    pub fn process(
        &mut self,
        input: &str,
        // ... other parameters
        config: &Config,
    ) {
        // Filter and select commands
        // Update state based on user input
    }
    
    pub fn select(&mut self) -> Option<String> {
        // Handle selection logic
    }
}
```

### 1.4 Fuzzy Matching

**File**: `fixtures/halloy/src/buffer/input_view/completion.rs`

**Fuzzy Matching Implementation**:

```rust
use nucleo_matcher::{Config, Matcher, Utf32Str};

pub fn fuzzy_match_options(
    options: &[String],
    input: &str,
    config: &Config,
) -> Vec<String> {
    let matcher = Matcher::new(config);
    
    let pattern = Pattern::new(
        input,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    
    let mut matches: Vec<_> = options
        .iter()
        .filter_map(|option| {
            matcher.fuzzy_match(option, pattern)
                .map(|score| (score, option))
        })
        .collect();
    
    // Sort by score (higher is better)
    matches.sort_by(|a, b| b.0.cmp(&a.0));
    
    matches.into_iter()
        .map(|(_, option)| option.clone())
        .collect()
}
```

**Pattern Configuration**:
```rust
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization};

let config = Config::DEFAULT;
let pattern = Pattern::new(
    "irc",
    CaseMatching::Smart,      // Smart case matching
    Normalization::Smart,    // Smart normalization
    AtomKind::Fuzzy,          // Fuzzy matching
);
```

### 1.5 Keyboard Navigation

**File**: `fixtures/halloy/src/widget/combo_box.rs`

**Keyboard Event Handling**:

```rust
// In the ComboBox widget update method
fn update(
    &mut self,
    state: &mut widget::Tree,
    event: &Event,
    // ... other parameters
) {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers }) => {
            match key {
                keyboard::Key::Named(key::Named::ArrowDown) => {
                    state.focused_index = Some(
                        state.focused_index.map_or(0, |i| i.saturating_add(1))
                            .min(state.filtered.len().saturating_sub(1))
                    );
                    return;
                }
                keyboard::Key::Named(key::Named::ArrowUp) => {
                    state.focused_index = state.focused_index
                        .map(|i| i.saturating_sub(1))
                        .filter(|&i| i < state.filtered.len());
                    return;
                }
                keyboard::Key::Named(key::Named::Enter) => {
                    if let Some(index) = state.focused_index {
                        if let Some(selected) = state.select(index) {
                            shell.publish((self.on_selected)(selected));
                        }
                    }
                    return;
                }
                keyboard::Key::Named(key::Named::Escape) => {
                    state.unfocus();
                    if let Some(on_close) = &self.on_close {
                        shell.publish(on_close.clone());
                    }
                    return;
                }
                _ => {}
            }
        }
        _ => {}
    }
}
```

**Navigation Logic**:
- **Arrow Down**: Move focus down, wrap around if at end
- **Arrow Up**: Move focus up, wrap around if at start
- **Enter**: Select focused item
- **Escape**: Close dropdown
- **Tab**: Cycle focus to next element

---

## 2. Moveable Widget Containers

The pane grid system allows users to split the screen into multiple resizable and draggable panes, each containing different content.

### 2.1 Pane Grid System

**File**: `fixtures/halloy/src/screen/dashboard.rs`

**Core Components**:

```rust
use iced::widget::pane_grid::{self, PaneGrid};

pub struct Dashboard {
    panes: pane_grid::State<Pane>,          // Main pane grid state
    popout: HashMap<window::Id, pane_grid::State<Pane>>, // Popout windows
    focus_history: VecDeque<pane_grid::Pane>, // Focus tracking
}

pub enum Message {
    PaneClicked(pane_grid::Pane),
    PaneResized(pane_grid::ResizeEvent),
    PaneDragged(pane_grid::DragEvent),
    SplitPane(pane_grid::Axis),
    ClosePane,
    MaximizePane,
    // ... other messages
}
```

### 2.2 Pane State Management

**File**: `fixtures/halloy/src/screen/dashboard/pane.rs`

**Pane Structure**:
```rust
pub struct Pane {
    pub buffer: Buffer,           // Content of the pane
    pub size: Size,               // Current size
    title_bar: TitleBar,          // Title bar customization
    pub modal: Option<super::modal::Modal>, // Modal overlay
}

impl Pane {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            size: Size::default(),
            title_bar: TitleBar::default(),
            modal: None,
        }
    }
    
    pub fn view<'a>(
        &'a self,
        id: pane_grid::Pane,
        panes: usize,
        is_focused: bool,
        maximized: bool,
        // ... other parameters
    ) -> widget::Content<'a, Message> {
        // Render the pane content
        let content = self.buffer.view(
            id,
            is_focused,
            // ... other parameters
        );
        
        // Add title bar if needed
        if !maximized && panes > 1 {
            let title_bar = self.title_bar.view(id, is_focused);
            column![title_bar, content].into()
        } else {
            content
        }
    }
}
```

### 2.3 Drag and Drop

**File**: `fixtures/halloy/src/screen/dashboard/pane.rs`

**Drag Event Handling**:

```rust
pub enum Message {
    PaneDragged(pane_grid::DragEvent),
    // ... other messages
}

// In update method
match message {
    Message::PaneDragged(event) => {
        match event {
            pane_grid::DragEvent::Dragged(_, to) => {
                // Handle drag to new position
                self.panes.drag(to);
            }
            pane_grid::DragEvent::Dropped { pane, target } => {
                // Handle drop on target
                self.panes.drop(pane, target);
            }
            pane_grid::DragEvent::Cancelled => {
                // Handle drag cancellation
                self.panes.cancel();
            }
        }
        Task::none()
    }
    // ... other message handling
}
```

**Pane Grid Drag API**:
```rust
// Start dragging a pane
pane_grid::State::drag(&mut self.panes, source_pane);

// Drop pane on target
pane_grid::State::drop(&mut self.panes, source_pane, target_pane);

// Cancel drag operation
pane_grid::State::cancel(&mut self.panes);
```

### 2.4 Resize Operations

**File**: `fixtures/halloy/src/screen/dashboard/pane.rs`

**Resize Event Handling**:

```rust
pub enum Message {
    PaneResized(pane_grid::ResizeEvent),
    // ... other messages
}

// In update method
Message::PaneResized(event) => {
    match event {
        pane_grid::ResizeEvent::Resized(pane, new_size) => {
            if let Some(pane_content) = self.panes.get_mut(pane) {
                pane_content.size = new_size;
            }
        }
        pane_grid::ResizeEvent::ResizeStarted(pane) => {
            // Handle resize start
        }
        pane_grid::ResizeEvent::ResizeEnded(pane) => {
            // Handle resize end
        }
    }
    Task::none()
}
```

**Pane Grid Resize API**:
```rust
// Start resize
pane_grid::State::resize_start(&mut self.panes, pane);

// Update resize
pane_grid::State::resize_update(&mut self.panes, new_size);

// End resize
pane_grid::State::resize_end(&mut self.panes);
```

### 2.5 Pane Operations

**File**: `fixtures/halloy/src/screen/dashboard.rs`

**Common Pane Operations**:

```rust
impl Dashboard {
    /// Split current pane
    fn new_pane(&mut self, axis: pane_grid::Axis) -> Task<Message> {
        if let Some((pane, _)) = self.panes.focused() {
            self.panes.split(axis, pane, Pane::new(Buffer::Empty))
                .map(|pane| Message::PaneCreated(pane))
        } else {
            Task::none()
        }
    }
    
    /// Close current pane
    fn close_pane(&mut self) -> Task<Message> {
        if self.panes.len() > 1 {
            if let Some((pane, _)) = self.panes.focused() {
                self.panes.close(pane)
                    .map(|_| Message::PaneClosed)
            } else {
                Task::none()
            }
        } else {
            Task::none()
        }
    }
    
    /// Maximize current pane
    fn maximize_pane(&mut self) -> Task<Message> {
        self.panes.maximize()
            .map(|_| Message::PaneMaximized)
    }
    
    /// Split specific pane
    fn split_pane(&mut self, axis: pane_grid::Axis, pane: pane_grid::Pane) -> Task<Message> {
        self.panes.split(axis, pane, Pane::new(Buffer::Empty))
            .map(|new_pane| Message::PaneSplit(new_pane))
    }
}
```

---

## 3. Modal System

The modal system provides overlay dialogs that appear on top of the main content, with configurable backdrop opacity and automatic closing on Escape key.

### 3.1 Modal Widget

**File**: `fixtures/halloy/src/widget/modal.rs`

**Core Modal Structure**:

```rust
pub struct Modal<'a, Message, Theme, Renderer> {
    base: Element<'a, Message, Theme, Renderer>,  // Background content
    modal: Element<'a, Message, Theme, Renderer>,  // Modal content
    on_blur: Box<dyn Fn() -> Message + 'a>,       // Close handler
    backdrop: Color,                               // Backdrop color/alpha
    shadow: Shadow,                                // Modal shadow
}

pub fn modal<'a, Message, Theme, Renderer>(
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    modal: impl Into<Element<'a, Message, Theme, Renderer>>,
    on_blur: impl Fn() -> Message + 'a,
    backdrop_alpha: f32,
) -> Element<'a, Message, Theme, Renderer> {
    Modal::new(base, modal, on_blur, backdrop_alpha).into()
}
```

**Modal Creation**:
```rust
let modal = widget::modal(
    base_content,  // Background content
    modal_content, // Modal overlay
    || Message::CloseModal, // Close handler
    0.7,           // Backdrop alpha (0.0-1.0)
);
```

### 3.2 Overlay Pattern

**File**: `fixtures/halloy/src/widget/modal.rs`

**Overlay Implementation**:

```rust
impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Modal<'_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    // ... other trait methods
    
    fn overlay<'b>(
        &'b mut self,
        state: &'b mut widget::Tree,
        layout: Layout<'b>,
        _renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        Some(overlay::Element::new(Box::new(Overlay {
            position: layout.position() + translation,
            content: &mut self.modal,
            tree: &mut state.children[1],
            size: layout.bounds().size(),
            on_blur: &self.on_blur,
            backdrop: self.backdrop,
            shadow: self.shadow,
            viewport: *viewport,
        })))
    }
}

struct Overlay<'a, 'b, Message, Theme, Renderer> {
    position: Point,
    content: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut widget::Tree,
    size: Size,
    on_blur: &'b dyn Fn() -> Message,
    backdrop: Color,
    shadow: Shadow,
    viewport: Rectangle,
}
```

### 3.3 Backdrop Handling

**File**: `fixtures/halloy/src/widget/modal.rs`

**Backdrop Rendering**:

```rust
impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        // Draw backdrop
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                ..renderer::Quad::default()
            },
            self.backdrop,
        );
        
        // Draw modal shadow
        let bounds = layout.children().next().unwrap().bounds();
        
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..iced::Border::default()
                },
                shadow: self.shadow,
                ..renderer::Quad::default()
            },
            Color::TRANSPARENT,
        );
        
        // Draw modal content
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            &layout.bounds(),
        );
    }
}
```

**Backdrop Configuration**:
```rust
// Create modal with semi-transparent backdrop
widget::modal(
    base_content,
    modal_content,
    || Message::CloseModal,
    0.7, // 70% opacity
);

// Create modal with solid backdrop
widget::modal(
    base_content,
    modal_content,
    || Message::CloseModal,
    1.0, // 100% opacity
);
```

### 3.4 Escape Key Handling

**File**: `fixtures/halloy/src/widget/modal.rs`

**Keyboard Event Processing**:

```rust
impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
{
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            }) => {
                // Close modal on Escape
                shell.publish((self.on_blur)());
                shell.capture_event();
                return;
            }
            _ => {}
        }
        
        // ... other event handling
    }
}
```

### 3.5 Click Outside Handling

**File**: `fixtures/halloy/src/widget/modal.rs`

**Mouse Event Processing**:

```rust
fn update(
    &mut self,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    // ... other parameters
) {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed {
            button: mouse::Button::Left,
            ..
        }) => {
            let bounds = layout.children().next().unwrap().bounds();
            
            // Close modal if clicked outside
            if !cursor.is_over(bounds) {
                shell.publish((self.on_blur)());
                shell.capture_event();
                return;
            }
        }
        _ => {}
    }
    
    // ... other event handling
}
```

---

## 4. Listing Components

Generic listing components with selection and keyboard navigation.

### 4.1 Generic List Pattern

**Pattern**: Create reusable list components similar to ComboBox

**Example List Structure**:

```rust
pub struct List<'a, T, Message> {
    items: &'a [T],
    selected: Option<usize>,
    on_selected: Box<dyn Fn(T) -> Message>,
    on_hovered: Option<Box<dyn Fn(T) -> Message>>,
    scrollable: bool,
    max_height: Option<f32>,
}

impl<'a, T: Clone> List<'a, T, Message> {
    pub fn new(
        items: &'a [T],
        selected: Option<usize>,
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self {
        Self {
            items,
            selected,
            on_selected: Box::new(on_selected),
            on_hovered: None,
            scrollable: true,
            max_height: Some(300.0),
        }
    }
    
    pub fn with_hover(
        mut self,
        on_hovered: impl Fn(T) -> Message + 'static,
    ) -> Self {
        self.on_hovered = Some(Box::new(on_hovered));
        self
    }
    
    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }
    
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }
    
    pub fn select(&mut self, index: usize) -> Option<T> {
        if index < self.items.len() {
            self.selected = Some(index);
            Some(self.items[index].clone())
        } else {
            None
        }
    }
    
    pub fn focus(&mut self) {
        // Focus logic
    }
    
    pub fn unfocus(&mut self) {
        // Unfocus logic
    }
}
```

### 4.2 Selection Tracking

**Selection Management**:

```rust
impl<'a, T, Message> List<'a, T, Message> {
    pub fn selected(&self) -> Option<&T> {
        self.selected.and_then(|i| self.items.get(i))
    }
    
    pub fn set_selected(&mut self, index: Option<usize>) {
        self.selected = index;
    }
    
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected == Some(index)
    }
    
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }
}
```

### 4.3 Keyboard Navigation

**Navigation Logic**:

```rust
fn handle_keyboard_event(
    &mut self,
    event: &Event,
    shell: &mut Shell<'_, Message>,
) {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
            match key {
                keyboard::Key::Named(key::Named::ArrowDown) => {
                    self.selected = self.selected
                        .map(|i| (i + 1).min(self.items.len().saturating_sub(1)))
                        .or(Some(0));
                    
                    if let Some(item) = self.selected.and_then(|i| self.items.get(i)) {
                        if let Some(on_hovered) = &self.on_hovered {
                            shell.publish(on_hovered(item.clone()));
                        }
                    }
                    return;
                }
                keyboard::Key::Named(key::Named::ArrowUp) => {
                    self.selected = self.selected
                        .map(|i| i.saturating_sub(1))
                        .filter(|&i| i < self.items.len());
                    
                    if let Some(item) = self.selected.and_then(|i| self.items.get(i)) {
                        if let Some(on_hovered) = &self.on_hovered {
                            shell.publish(on_hovered(item.clone()));
                        }
                    }
                    return;
                }
                keyboard::Key::Named(key::Named::Enter) => {
                    if let Some(index) = self.selected {
                        if let Some(item) = self.items.get(index) {
                            shell.publish((self.on_selected)(item.clone()));
                        }
                    }
                    return;
                }
                keyboard::Key::Named(key::Named::Escape) => {
                    self.clear_selection();
                    return;
                }
                _ => {}
            }
        }
        _ => {}
    }
}
```

---

## 5. Context Menu System

Right-click context menus with positioning and submenu support.

### 5.1 Right-click Menus

**File**: `fixtures/halloy/src/widget/context_menu.rs`

**ContextMenu Structure**:

```rust
pub struct ContextMenu<'a, Message, Theme = crate::Theme, Renderer = super::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    trigger: Element<'a, Message, Theme, Renderer>,
    menu: Vec<MenuItem<'a, Message, Theme, Renderer>>,
    on_menu: Box<dyn Fn() -> Message>,
    on_close: Option<Box<dyn Fn() -> Message>>,
    offset: Vector,
    placement: Placement,
    open_on: OpenOn,
}

pub enum MenuItem<'a, Message, Theme = crate::Theme, Renderer = super::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    Button {
        content: Element<'a, Message, Theme, Renderer>,
        on_press: Message,
        enabled: bool,
    },
    Submenu {
        label: String,
        items: Vec<MenuItem<'a, Message, Theme, Renderer>>,
    },
    Separator,
}
```

**ContextMenu Creation**:
```rust
let context_menu = context_menu::ContextMenu::new(
    button::text("Right-click me"),
    vec![
        MenuItem::Button {
            content: text("Copy").into(),
            on_press: Message::Copy,
            enabled: true,
        },
        MenuItem::Button {
            content: text("Paste").into(),
            on_press: Message::Paste,
            enabled: true,
        },
        MenuItem::Separator,
        MenuItem::Submenu {
            label: "More options".to_string(),
            items: vec![
                MenuItem::Button {
                    content: text("Option 1").into(),
                    on_press: Message::Option1,
                    enabled: true,
                },
                MenuItem::Button {
                    content: text("Option 2").into(),
                    on_press: Message::Option2,
                    enabled: false,
                },
            ],
        },
    ],
    || Message::MenuOpened,
)
.offset(Vector::new(10.0, 5.0))
.placement(Placement::Bottom);
```

### 5.2 Submenus

**Submenu Implementation**:

```rust
impl<'a, Message, Theme, Renderer> MenuItem<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn is_submenu(&self) -> bool {
        matches!(self, MenuItem::Submenu { .. })
    }
    
    pub fn submenu_items(&self) -> Option<&[MenuItem<'a, Message, Theme, Renderer>]> {
        match self {
            MenuItem::Submenu { items, .. } => Some(items),
            _ => None,
        }
    }
}
```

### 5.3 Positioning

**Placement Options**:

```rust
pub enum Placement {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub enum OpenOn {
    Click,
    RightClick,
    Hover,
}
```

**Positioning Logic**:
```rust
fn calculate_position(
    trigger_bounds: Rectangle,
    menu_size: Size,
    placement: Placement,
    offset: Vector,
) -> Point {
    match placement {
        Placement::Top => Point {
            x: trigger_bounds.center_x() - menu_size.width / 2.0,
            y: trigger_bounds.y - menu_size.height - offset.y,
        },
        Placement::Bottom => Point {
            x: trigger_bounds.center_x() - menu_size.width / 2.0,
            y: trigger_bounds.y + trigger_bounds.height + offset.y,
        },
        Placement::Left => Point {
            x: trigger_bounds.x - menu_size.width - offset.x,
            y: trigger_bounds.center_y() - menu_size.height / 2.0,
        },
        Placement::Right => Point {
            x: trigger_bounds.x + trigger_bounds.width + offset.x,
            y: trigger_bounds.center_y() - menu_size.height / 2.0,
        },
        // ... other placements
    }
}
```

---

## 6. Integration Examples

### 6.1 Command Picker in Moveable Pane

**Complete Integration Example**:

```rust
// 1. Define your message types
pub enum Message {
    CommandPickerInput(String),
    CommandPickerSelected(String),
    PaneSplit(pane_grid::Axis),
    PaneClose,
    // ... other messages
}

// 2. Define pane content
pub enum PaneContent {
    CommandPicker(CommandPickerState),
    ChatBuffer(ChatBufferState),
    // ... other content types
}

// 3. Create command picker state
let command_options = vec![
    "/join #channel",
    "/part #channel",
    "/query nickname",
    "/msg nickname message",
    "/nick newnick",
];

let command_state = combo_box::State::new(command_options);

// 4. Create command picker widget
let command_picker = combo_box::combo_box(
    &command_state,
    "> ",
    None,
    Message::CommandPickerSelected,
)
.on_input(Message::CommandPickerInput)
.menu_class(your_theme::combo_box_menu());

// 5. Create pane content
let pane_content = match pane_content {
    PaneContent::CommandPicker(_) => command_picker.into(),
    PaneContent::ChatBuffer(state) => chat_buffer_view(state),
    // ...
};

// 6. Create pane grid
let pane_grid = PaneGrid::new(&state.panes, |pane_id, content, maximized| {
    match content {
        PaneContent::CommandPicker(state) => {
            let picker = combo_box::combo_box(
                &state.command_state,
                "> ",
                None,
                Message::CommandPickerSelected,
            )
            .on_input(Message::CommandPickerInput)
            .menu_class(your_theme::combo_box_menu());
            
            picker.into()
        }
        PaneContent::ChatBuffer(state) => {
            chat_buffer_view(state).map(|msg| Message::ChatBuffer(msg))
        }
    }
})
.on_drag(Message::PaneDragged)
.on_resize(Message::PaneResized);

// 7. Handle messages
match message {
    Message::CommandPickerSelected(cmd) => {
        // Handle command selection
        handle_command(cmd);
    }
    Message::PaneSplit(axis) => {
        // Split current pane
        state.split_current_pane(axis);
    }
    // ... other message handling
}
```

### 6.2 Modal with Context Menu

**Combining Modal and Context Menu**:

```rust
pub enum Message {
    OpenSettingsModal,
    CloseModal,
    SettingsAction(SettingsAction),
    ContextMenuEvent(ContextMenuEvent),
    // ... other messages
}

// In your view function
let settings_button = button::text("Settings")
    .on_press(Message::OpenSettingsModal);

let content = column![
    settings_button,
    // ... other content
];

// Create modal if needed
let view = if let Some(settings) = &self.settings_modal {
    widget::modal(
        content,
        settings_view(settings),
        || Message::CloseModal,
        0.7,
    )
} else {
    content
};

// Settings view with context menu
fn settings_view(settings: &Settings) -> Element<Message> {
    let context_menu = context_menu::ContextMenu::new(
        container(button::text("Options")),
        vec![
            MenuItem::Button {
                content: text("Save").into(),
                on_press: Message::SettingsAction(SettingsAction::Save),
                enabled: true,
            },
            MenuItem::Button {
                content: text("Reset").into(),
                on_press: Message::SettingsAction(SettingsAction::Reset),
                enabled: true,
            },
        ],
        || Message::ContextMenuEvent(ContextMenuEvent::Opened),
    )
    .offset(Vector::new(5.0, 0.0))
    .placement(Placement::BottomRight);
    
    context_menu.into()
}
```

### 6.3 Responsive Layouts

**Adaptive Layout Example**:

```rust
pub struct ResponsiveLayout {
    panes: pane_grid::State<PaneContent>,
    sidebar_collapsed: bool,
    show_toolbar: bool,
}

impl ResponsiveLayout {
    pub fn view(&self) -> Element<Message> {
        // Determine layout based on available space
        let available_width = self.calculate_available_width();
        
        if available_width > 1200.0 {
            // Wide layout: sidebar + main content
            row![
                sidebar_view(self.sidebar_collapsed),
                pane_grid_view(&self.panes),
            ]
            .spacing(10)
            .into()
        } else if available_width > 800.0 {
            // Medium layout: collapsible sidebar
            column![
                toolbar_view(self.show_toolbar),
                row![
                    if !self.sidebar_collapsed {
                        sidebar_view(false)
                    } else {
                        Space::new(0, 0).into()
                    },
                    pane_grid_view(&self.panes),
                ]
                .spacing(10),
            ]
            .into()
        } else {
            // Narrow layout: only main content
            column![
                toolbar_view(self.show_toolbar),
                pane_grid_view(&self.panes),
            ]
            .into()
        }
    }
    
    fn calculate_available_width(&self) -> f32 {
        // Calculate available width
        // ...
    }
}
```

---

## 7. Theme Integration

Custom theming for your widgets.

### 7.1 Custom Themes

**Implementing Theme Catalogs**:

```rust
// In your theme.rs file
use iced::widget::combo_box;

pub struct Theme {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub background: Color,
    pub text: Color,
    pub surface: Color,
    // ... other theme properties
}

impl combo_box::Catalog for Theme {
    type Class<'a> = combo_box::Class<'a>;
    
    fn default_input(&self) -> Self::Class<'_> {
        combo_box::Class::default()
            .input(self.button_style())
            .hovered(self.button_hovered_style())
            .focused(self.button_focused_style())
            .placeholder(self.placeholder_style())
            .text(self.text_style())
            .selection(self.selection_style())
            .icon(self.icon_style())
    }
    
    fn default_menu(&self) -> Self::Class<'_> {
        combo_box::Class::default()
            .menu(self.menu_style())
            .item(self.menu_item_style())
            .selected(self.selected_menu_item_style())
            .hovered(self.hovered_menu_item_style())
    }
}

// Similar implementations for other widgets
impl context_menu::Catalog for Theme {
    // ...
}

impl modal::Catalog for Theme {
    // ...
}
```

### 7.2 Styling Hooks

**Widget Styling Methods**:

```rust
// Example for combo box
impl Theme {
    pub fn combo_box_input_style(&self) -> combo_box::Style {
        combo_box::Style {
            background: Background::Color(self.surface),
            border: Border {
                width: 1.0,
                radius: 4.0.into(),
                color: self.primary_color.with_alpha(0.3),
            },
            // ... other style properties
        }
    }
    
    pub fn combo_box_menu_style(&self) -> combo_box::MenuStyle {
        combo_box::MenuStyle {
            background: Background::Color(self.surface),
            border: Border {
                width: 1.0,
                radius: 4.0.into(),
                color: self.primary_color.with_alpha(0.3),
            },
            // ... other style properties
        }
    }
    
    pub fn combo_box_item_style(&self) -> combo_box::ItemStyle {
        combo_box::ItemStyle {
            text_color: self.text,
            selected_text_color: self.primary_color,
            hovered_background: Background::Color(
                self.primary_color.with_alpha(0.1)
            ),
            // ... other style properties
        }
    }
}
```

### 7.3 Color Schemes

**Dark/Light Mode Support**:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorScheme {
    Light,
    Dark,
}

impl Theme {
    pub fn with_scheme(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Light => Self::light_theme(),
            ColorScheme::Dark => Self::dark_theme(),
        }
    }
    
    fn light_theme() -> Self {
        Self {
            primary_color: Color::from_rgb(0.2, 0.5, 0.8),
            secondary_color: Color::from_rgb(0.8, 0.3, 0.3),
            background: Color::from_rgb8(245, 245, 245),
            text: Color::from_rgb8(50, 50, 50),
            surface: Color::WHITE,
            // ...
        }
    }
    
    fn dark_theme() -> Self {
        Self {
            primary_color: Color::from_rgb(0.3, 0.6, 0.9),
            secondary_color: Color::from_rgb(0.9, 0.4, 0.4),
            background: Color::from_rgb8(30, 30, 30),
            text: Color::WHITE,
            surface: Color::from_rgb8(50, 50, 50),
            // ...
        }
    }
}
```

---

## 8. Performance Considerations

### 8.1 Rendering Optimization

**Lazy Rendering**:

```rust
// Only render when needed
fn should_render(&self) -> bool {
    self.dirty || self.needs_layout
}

// Use caching for expensive operations
fn cached_render(&mut self) -> Element<Message> {
    if let Some(cached) = &self.cached_view {
        cached.clone()
    } else {
        let rendered = self.render();
        self.cached_view = Some(rendered.clone());
        rendered
    }
}
```

**Virtual Scrolling for Lists**:

```rust
pub struct VirtualList<'a, T, Message> {
    items: &'a [T],
    visible_range: Range<usize>,
    item_height: f32,
    on_render_item: Box<dyn Fn(&T) -> Element<'a, Message>>,
}

impl<'a, T, Message> VirtualList<'a, T, Message> {
    pub fn new(
        items: &'a [T],
        item_height: f32,
        on_render_item: impl Fn(&T) -> Element<'a, Message> + 'static,
    ) -> Self {
        Self {
            items,
            visible_range: 0..0,
            item_height,
            on_render_item: Box::new(on_render_item),
        }
    }
    
    pub fn with_visible_range(mut self, range: Range<usize>) -> Self {
        self.visible_range = range;
        self
    }
}
```

### 8.2 Event Handling Optimization

**Debounced Input**:

```rust
use std::time::{Duration, Instant};

pub struct DebouncedInput {
    input: String,
    last_change: Instant,
    debounce_duration: Duration,
}

impl DebouncedInput {
    pub fn new(debounce_duration: Duration) -> Self {
        Self {
            input: String::new(),
            last_change: Instant::now(),
            debounce_duration,
        }
    }
    
    pub fn update(&mut self, new_input: String) {
        self.input = new_input;
        self.last_change = Instant::now();
    }
    
    pub fn should_process(&self) -> bool {
        self.last_change.elapsed() >= self.debounce_duration
    }
    
    pub fn clear(&mut self) {
        self.input.clear();
        self.last_change = Instant::now();
    }
}

// Usage in combo box
let debounced = DebouncedInput::new(Duration::from_millis(300));

if debounced.should_process() {
    // Process the input
    state.filter(&debounced.input);
}
```

### 8.3 State Management

**Efficient State Updates**:

```rust
// Use Clone::clone only when necessary
pub struct AppState {
    pub panes: PaneGridState,
    pub modal: Option<Modal>,
    pub command_state: CommandState,
    // Use Arc for shared state
    pub shared_config: Arc<Config>,
}

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CommandInputChanged(input) => {
                // Only clone if needed
                if input.len() > 3 || input.ends_with(' ') {
                    self.command_state.update_input(input);
                }
                Task::none()
            }
            // ... other messages
        }
    }
}
```

---

## 9. Testing Patterns

### 9.1 Unit Testing

**Message Handling Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_selection() {
        let mut state = combo_box::State::new(vec![
            "/join #test",
            "/part #test",
        ]);
        
        // Test filtering
        state.filter("join");
        assert_eq!(state.filtered.len(), 1);
        
        // Test selection
        if let Some(selected) = state.select(0) {
            assert_eq!(selected, "/join #test");
        }
    }
    
    #[test]
    fn test_pane_operations() {
        let mut panes = pane_grid::State::new(Pane::new(Buffer::Empty));
        
        // Test split
        if let Some((pane, _)) = panes.focused() {
            panes.split(pane_grid::Axis::Horizontal, pane, Pane::new(Buffer::Empty));
            assert_eq!(panes.len(), 2);
        }
        
        // Test close
        if let Some((pane, _)) = panes.focused() {
            panes.close(pane);
            assert_eq!(panes.len(), 1);
        }
    }
}
```

### 9.2 Integration Testing

**End-to-End Flow Tests**:

```rust
#[test]
fn test_command_picker_flow() {
    // Setup
    let mut app = App::new();
    
    // Open command picker
    app.handle_key_press(Key::Named(Named::P), Modifiers::CTRL);
    assert!(app.command_picker_is_open());
    
    // Type command
    app.type_text("/join #test");
    assert_eq!(app.filtered_commands().len(), 1);
    
    // Select command
    app.press_key(Key::Named(Named::Enter));
    assert!(!app.command_picker_is_open());
    assert_eq!(app.current_command(), Some("/join #test"));
}
```

### 9.3 Visual Regression Testing

**Screenshot Comparison**:

```rust
#[test]
fn test_modal_appearance() {
    let theme = Theme::dark();
    let modal = create_test_modal();
    
    let rendered = modal.view(&theme);
    
    // Compare with golden image
    assert_snapshot!(
        "modal_dark_theme",
        rendered,
        theme = theme
    );
}
```

---

## Summary

This document provides a comprehensive reference for the UI components and patterns used in Halloy. Key takeaways:

1. **Command Picker**: Use `combo_box.rs` with fuzzy matching via `nucleo-matcher`
2. **Moveable Panes**: Leverage `pane_grid` from Iced with drag-and-drop support
3. **Modals**: Implement overlay modals with configurable backdrop and escape handling
4. **Listings**: Create generic list components with keyboard navigation
5. **Context Menus**: Use right-click menus with positioning support
6. **Theming**: Implement custom themes via `Catalog` traits
7. **Performance**: Use lazy rendering, debounced input, and efficient state management

**Recommended starting points**:
- Extract `combo_box.rs` and `modal.rs` first for command picker and modal system
- Use `pane_grid` from Iced for moveable containers
- Implement custom themes early for styling consistency
