# Reusable UI Patterns Extraction Guide

This document provides a practical guide for extracting and reusing the UI patterns from Halloy in your own project.

---

## Quick Start Guide

### 1. Essential Patterns to Extract First

| Pattern | Files to Extract | Priority | Dependencies |
|---------|------------------|----------|--------------|
| **Command Picker** | `src/widget/combo_box.rs` | High | `nucleo-matcher` |
| **Modal System** | `src/widget/modal.rs` | High | None |
| **Pane Grid** | `src/screen/dashboard.rs` (pane parts) | High | Iced's `pane_grid` |
| **Context Menu** | `src/widget/context_menu.rs` | Medium | None |
| **Theme System** | `src/appearance/theme.rs` | Medium | Iced's theme traits |

---

## Pattern 1: Command Picker (High Priority)

### Files to Extract

```bash
# Core widget
cp src/widget/combo_box.rs your_project/src/widget/

# Theme integration
cp src/appearance/theme.rs your_project/src/appearance/
cp -r src/appearance/theme your_project/src/appearance/

# Usage example
cp src/buffer/input_view/completion.rs your_project/src/examples/
```

### Minimal Implementation

**1. Create your message types:**

```rust
// src/messages.rs or your main message file
pub enum Message {
    CommandInput(String),
    CommandSelected(String),
    CommandPickerClosed,
}
```

**2. Create a simple command picker:**

```rust
// src/widgets/command_picker.rs
use crate::widget::combo_box;

pub struct CommandPicker {
    state: combo_box::State<String>,
}

impl CommandPicker {
    pub fn new(commands: Vec<String>) -> Self {
        Self {
            state: combo_box::State::new(commands),
        }
    }
    
    pub fn view(&self) -> iced::Element<Message> {
        combo_box::combo_box(
            &self.state,
            "> ",
            None,
            Message::CommandSelected,
        )
        .on_input(Message::CommandInput)
        .menu_class(crate::theme::combo_box::menu())
        .into()
    }
}
```

**3. Add to your app:**

```rust
// In your main view function
let command_picker = CommandPicker::new(vec![
    "/join #channel".to_string(),
    "/part #channel".to_string(),
    "/query nickname".to_string(),
]);

let content = column![
    command_picker.view(),
    // ... your other content
];
```

**4. Handle messages:**

```rust
match message {
    Message::CommandSelected(cmd) => {
        println!("Selected command: {}", cmd);
        // Execute the command
    }
    Message::CommandInput(input) => {
        // Filter commands based on input
        state.filter_commands(&input);
    }
    Message::CommandPickerClosed => {
        // Clean up
    }
}
```

### Dependencies to Add

```toml
[dependencies]
nucleo-matcher = "0.3"
iced = { version = "0.15", features = ["advanced"] }
```

### Customization Points

1. **Commands**: Replace the hardcoded commands with your own list
2. **Styling**: Implement `combo_box::Catalog` for your theme
3. **Behavior**: Modify filtering logic in `State::filter()`
4. **Selection**: Change what happens when a command is selected

---

## Pattern 2: Modal System (High Priority)

### Files to Extract

```bash
# Core modal widget
cp src/widget/modal.rs your_project/src/widget/

# Modal state management
cp src/modal.rs your_project/src/
```

### Minimal Implementation

**1. Define modal types:**

```rust
// src/modal.rs
pub enum Modal {
    Settings(SettingsModal),
    Confirmation(ConfirmationModal),
    // ... other modal types
}

pub enum Message {
    OpenModal(Modal),
    CloseModal,
    ModalAction(ModalAction),
}
```

**2. Create a simple modal:**

```rust
// src/widgets/settings_modal.rs
use crate::widget::modal;

pub struct SettingsModal {
    // Your settings state
}

impl SettingsModal {
    pub fn new() -> Self {
        Self { /* ... */ }
    }
    
    pub fn view(&self) -> iced::Element<Message> {
        // Create your modal content
        let content = container(
            column![
                text("Settings").size(24),
                // ... settings controls
                button("Close").on_press(Message::CloseModal),
            ]
        )
        .padding(20);
        
        content.into()
    }
    
    pub fn update(&mut self, message: ModalAction) {
        // Handle modal-specific actions
    }
}
```

**3. Add to your app:**

```rust
// In your main struct
pub struct App {
    modal: Option<Modal>,
    // ... other fields
}

// In your view function
let content = your_main_content();

let view = if let Some(modal) = &self.modal {
    widget::modal(
        content,
        modal.view(),
        || Message::CloseModal,
        0.7, // backdrop opacity
    )
} else {
    content
};
```

**4. Handle modal messages:**

```rust
match message {
    Message::OpenModal(modal) => {
        self.modal = Some(modal);
    }
    Message::CloseModal => {
        self.modal = None;
    }
    Message::ModalAction(action) => {
        if let Some(modal) = &mut self.modal {
            modal.update(action);
        }
    }
}
```

### Customization Points

1. **Modal Content**: Replace `SettingsModal` with your own modal types
2. **Backdrop**: Adjust the opacity (0.0-1.0)
3. **Close Behavior**: Modify what happens on Escape or backdrop click
4. **Positioning**: Change modal alignment (currently centered)

---

## Pattern 3: Pane Grid (Moveable Containers - High Priority)

### Files to Extract

```bash
# Extract pane-related logic from dashboard
# You'll need to extract relevant parts from:
# - src/screen/dashboard.rs (pane management)
# - src/screen/dashboard/pane.rs (Pane struct and view)

# Create your own pane module
mkdir -p your_project/src/widgets/pane
touch your_project/src/widgets/pane/mod.rs
```

### Minimal Implementation

**1. Define pane types:**

```rust
// src/widgets/pane/mod.rs
use iced::widget::pane_grid;

pub enum PaneContent {
    Chat,
    CommandPicker(CommandPicker),
    Settings,
    // ... other content types
}

pub struct Pane {
    content: PaneContent,
    size: iced::Size,
}

impl Pane {
    pub fn new(content: PaneContent) -> Self {
        Self {
            content,
            size: iced::Size::new(300.0, 400.0),
        }
    }
    
    pub fn view(&self, is_focused: bool) -> iced::Element<Message> {
        match &self.content {
            PaneContent::Chat => chat_view(is_focused),
            PaneContent::CommandPicker(cmd_picker) => cmd_picker.view(),
            PaneContent::Settings => settings_view(),
        }
    }
}
```

**2. Create pane grid state:**

```rust
// In your main app struct
use iced::widget::pane_grid;

pub struct App {
    panes: pane_grid::State<Pane>,
    // ... other fields
}

impl App {
    pub fn new() -> Self {
        let initial_pane = Pane::new(PaneContent::Chat);
        Self {
            panes: pane_grid::State::new(initial_pane),
            // ...
        }
    }
    
    pub fn split_pane(&mut self, axis: pane_grid::Axis) {
        if let Some((pane, _)) = self.panes.focused() {
            let new_pane = Pane::new(PaneContent::Chat);
            self.panes.split(axis, pane, new_pane);
        }
    }
}
```

**3. Create the view:**

```rust
// In your view function
let pane_grid = pane_grid::PaneGrid::new(
    &self.panes,
    |pane_id, pane, maximized| {
        let is_focused = self.panes.focused().map(|(id, _)| *id) == Some(pane_id);
        pane.view(is_focused).map(move |message| {
            Message::Pane(pane_id, message)
        })
    }
)
.on_drag(Message::PaneDragged)
.on_resize(Message::PaneResized);
```

**4. Handle pane messages:**

```rust
match message {
    Message::SplitPane(axis) => {
        self.split_pane(axis);
    }
    Message::ClosePane => {
        if let Some((pane, _)) = self.panes.focused() {
            self.panes.close(pane);
        }
    }
    Message::PaneDragged(event) => {
        match event {
            pane_grid::DragEvent::Dropped { pane, target } => {
                self.panes.drop(pane, target);
            }
            _ => {}
        }
    }
    Message::PaneResized(event) => {
        if let pane_grid::ResizeEvent::Resized(pane, size) = event {
            if let Some(pane_content) = self.panes.get_mut(pane) {
                pane_content.size = size;
            }
        }
    }
    // ... other messages
}
```

### Dependencies to Add

```toml
[dependencies]
iced = { version = "0.15", features = ["advanced"] }
```

### Customization Points

1. **Pane Content**: Replace `PaneContent` enum with your own types
2. **Initial Layout**: Change the initial pane configuration
3. **Pane Operations**: Add/remove operations as needed
4. **Styling**: Customize pane appearance via theme

---

## Pattern 4: Context Menu (Medium Priority)

### Files to Extract

```bash
cp src/widget/context_menu.rs your_project/src/widget/
```

### Minimal Implementation

**1. Define menu items:**

```rust
// src/widgets/context_menu_example.rs
use crate::widget::context_menu;

pub enum Message {
    ContextMenuEvent(ContextMenuEvent),
    MenuAction(MenuAction),
}

pub enum ContextMenuEvent {
    Opened,
    Closed,
}

pub enum MenuAction {
    Copy,
    Paste,
    Delete,
    Settings,
}
```

**2. Create a context menu:**

```rust
pub fn create_context_menu() -> context_menu::ContextMenu<'static, Message> {
    let trigger = button::text("Right-click me")
        .on_right_press(Message::ContextMenuEvent(ContextMenuEvent::Opened));
    
    context_menu::ContextMenu::new(
        trigger,
        vec![
            context_menu::MenuItem::Button {
                content: text("Copy").into(),
                on_press: Message::MenuAction(MenuAction::Copy),
                enabled: true,
            },
            context_menu::MenuItem::Button {
                content: text("Paste").into(),
                on_press: Message::MenuAction(MenuAction::Paste),
                enabled: true,
            },
            context_menu::MenuItem::Separator,
            context_menu::MenuItem::Button {
                content: text("Settings").into(),
                on_press: Message::MenuAction(MenuAction::Settings),
                enabled: true,
            },
        ],
        || Message::ContextMenuEvent(ContextMenuEvent::Opened),
    )
    .offset(iced::Vector::new(5.0, 0.0))
    .placement(context_menu::Placement::BottomRight)
}
```

**3. Handle messages:**

```rust
match message {
    Message::ContextMenuEvent(event) => match event {
        ContextMenuEvent::Opened => {
            // Menu opened, you might want to track this
        }
        ContextMenuEvent::Closed => {
            // Menu closed
        }
    },
    Message::MenuAction(action) => {
        match action {
            MenuAction::Copy => { /* handle copy */ }
            MenuAction::Paste => { /* handle paste */ }
            MenuAction::Settings => { /* open settings */ }
            MenuAction::Delete => { /* handle delete */ }
        }
    }
}
```

### Customization Points

1. **Menu Items**: Add/remove buttons and submenus as needed
2. **Positioning**: Change placement (Top, Bottom, Left, Right, etc.)
3. **Appearance**: Customize via theme implementation
4. **Behavior**: Modify what happens when items are selected

---

## Pattern 5: Theme System (Medium Priority)

### Files to Extract

```bash
# Theme structure
cp src/appearance/theme.rs your_project/src/appearance/
cp -r src/appearance/theme your_project/src/appearance/

# Theme catalogs
cp src/appearance/theme/*.rs your_project/src/appearance/theme/
```

### Minimal Implementation

**1. Create your theme:**

```rust
// src/appearance/theme.rs
use iced::{Color, Background, Border, Shadow};

#[derive(Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub border: Color,
    pub shadow: Shadow,
    // ... other theme properties
}

impl Theme {
    pub fn light() -> Self {
        Self {
            primary: Color::from_rgb(0.2, 0.5, 0.8),
            secondary: Color::from_rgb(0.8, 0.3, 0.3),
            background: Color::from_rgb8(245, 245, 245),
            surface: Color::WHITE,
            text: Color::from_rgb8(50, 50, 50),
            text_secondary: Color::from_rgb8(100, 100, 100),
            border: Color::from_rgb8(200, 200, 200),
            shadow: Shadow::default(),
        }
    }
    
    pub fn dark() -> Self {
        Self {
            primary: Color::from_rgb(0.3, 0.6, 0.9),
            secondary: Color::from_rgb(0.9, 0.4, 0.4),
            background: Color::from_rgb8(30, 30, 30),
            surface: Color::from_rgb8(50, 50, 50),
            text: Color::WHITE,
            text_secondary: Color::from_rgb8(200, 200, 200),
            border: Color::from_rgb8(100, 100, 100),
            shadow: Shadow {
                color: Color::BLACK.with_alpha(0.5),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
        }
    }
}
```

**2. Implement widget catalogs:**

```rust
// For combo_box
impl iced::widget::combo_box::Catalog for Theme {
    type Class<'a> = combo_box::Class<'a>;
    
    fn default_input(&self) -> Self::Class<'_> {
        combo_box::Class::default()
            .input(self.button_primary())
            .hovered(self.button_primary_hovered())
            .focused(self.button_primary_focused())
            .placeholder(self.text_secondary)
            .text(self.text)
            .selection(self.primary)
            .icon(self.primary)
    }
    
    fn default_menu(&self) -> Self::Class<'_> {
        combo_box::Class::default()
            .menu(self.surface)
            .item(self.menu_item_style())
            .selected(self.selected_menu_item_style())
            .hovered(self.hovered_menu_item_style())
    }
}

// Similar implementations for other widgets...
```

**3. Use the theme in your app:**

```rust
// In your main app
pub struct App {
    theme: Theme,
    // ... other fields
}

impl App {
    pub fn new() -> Self {
        Self {
            theme: Theme::light(),
            // ...
        }
    }
    
    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Light(_) => Theme::dark(),
            Theme::Dark(_) => Theme::light(),
        };
    }
}

// In your view function
fn view(&self) -> Element<Message> {
    let theme = &self.theme;
    
    // Use theme in your widgets
    let combo_box = combo_box::combo_box(
        &self.command_state,
        "> ",
        None,
        Message::CommandSelected,
    )
    .menu_class(theme);
    
    combo_box.into()
}
```

### Customization Points

1. **Colors**: Define your app's color scheme
2. **Widget Styles**: Customize each widget's appearance
3. **Dark/Light Mode**: Implement theme switching
4. **Spacing**: Adjust padding, margins, and spacing

---

## Integration Examples

### Example 1: Command Picker in a Pane

```rust
// Create a pane with a command picker
let command_picker_pane = Pane::new(PaneContent::CommandPicker(
    CommandPicker::new(vec![
        "/connect server".to_string(),
        "/disconnect".to_string(),
        "/join #channel".to_string(),
    ])
));

// Add to your pane grid
self.panes.split(
    pane_grid::Axis::Horizontal,
    current_pane,
    command_picker_pane
);
```

### Example 2: Modal with Settings

```rust
// Create settings modal
let settings_modal = SettingsModal::new(self.config.clone());

// Open modal
self.modal = Some(Modal::Settings(settings_modal));

// In view
let content = your_main_content();

let view = if let Some(modal) = &self.modal {
    match modal {
        Modal::Settings(settings) => {
            widget::modal(
                content,
                settings.view(),
                || Message::CloseModal,
                0.7,
            )
        }
        Modal::Confirmation(confirmation) => {
            widget::modal(
                content,
                confirmation.view(),
                || Message::CloseModal,
                0.7,
            )
        }
    }
} else {
    content
};
```

### Example 3: Context Menu on List Items

```rust
pub struct ListItem {
    id: String,
    name: String,
    // ... other data
}

pub fn list_item_view(item: &ListItem) -> Element<Message> {
    let context_menu = context_menu::ContextMenu::new(
        row![
            text(&item.name),
            horizontal_space(),
            icon_button("more_vert")
                .on_press(Message::ContextMenuEvent(ContextMenuEvent::Opened))
        ],
        vec![
            context_menu::MenuItem::Button {
                content: text("Edit").into(),
                on_press: Message::EditItem(item.id.clone()),
                enabled: true,
            },
            context_menu::MenuItem::Button {
                content: text("Delete").into(),
                on_press: Message::DeleteItem(item.id.clone()),
                enabled: true,
            },
        ],
        || Message::ContextMenuEvent(ContextMenuEvent::Opened),
    );
    
    context_menu.into()
}
```

---

## Testing Your Extracted Patterns

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_picker_filtering() {
        let mut picker = CommandPicker::new(vec![
            "/join #test".to_string(),
            "/part #test".to_string(),
        ]);
        
        // Test filtering
        picker.handle_input("join");
        assert_eq!(picker.filtered_commands().len(), 1);
        
        // Test selection
        let selected = picker.select(0);
        assert_eq!(selected, Some("/join #test".to_string()));
    }
    
    #[test]
    fn test_modal_close() {
        let mut app = App::new();
        
        // Open modal
        app.open_modal(Modal::Settings(SettingsModal::new()));
        assert!(app.modal.is_some());
        
        // Close modal
        app.handle_message(Message::CloseModal);
        assert!(app.modal.is_none());
    }
    
    #[test]
    fn test_pane_split() {
        let mut app = App::new();
        let initial_count = app.panes.len();
        
        // Split pane
        app.split_pane(pane_grid::Axis::Horizontal);
        assert_eq!(app.panes.len(), initial_count + 1);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_command_picker_workflow() {
    let mut app = App::new();
    
    // Simulate opening command picker
    app.handle_key_press(
        iced::keyboard::Key::Named(iced::keyboard::key::Named::P),
        iced::keyboard::Modifiers::CTRL,
    );
    assert!(app.command_picker_is_visible());
    
    // Simulate typing
    app.type_text("/join #");
    app.type_text("test");
    
    // Check filtered results
    assert!(app.filtered_commands().len() > 0);
    
    // Select command
    app.press_key(iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter));
    
    // Command should be executed
    assert_eq!(app.executed_command(), Some("/join #test"));
}
```

---

## Performance Optimization Checklist

### For Command Picker
- [ ] Use debounced input (300ms delay) for filtering
- [ ] Implement virtual scrolling for large lists (>100 items)
- [ ] Cache filtered results when possible
- [ ] Use `Arc` for shared command lists

### For Modal System
- [ ] Only render modal content when open
- [ ] Use `lazy` rendering for modal content
- [ ] Avoid heavy computations in modal render

### For Pane Grid
- [ ] Use `pane_grid::State` efficiently (don't clone unnecessarily)
- [ ] Implement layout caching
- [ ] Optimize pane content rendering
- [ ] Use `Task::none()` for operations that don't return tasks

### For Theme System
- [ ] Cache theme-derived styles
- [ ] Use `Rc<Theme>` for shared themes
- [ ] Avoid expensive color conversions in hot paths

---

## Common Pitfalls and Solutions

### Pitfall 1: Message Bloat
**Problem**: Too many message variants make the code hard to maintain.

**Solution**:
```rust
// Instead of:
pub enum Message {
    CommandInput(String),
    CommandSelected(String),
    CommandPickerClosed,
    PaneSplitHorizontal,
    PaneSplitVertical,
    ClosePane,
    // ... 50 more variants
}

// Use nested messages:
pub enum Message {
    Command(command_picker::Message),
    Pane(pane::Message),
    Modal(modal::Message),
    // ... only a few top-level variants
}

// Then delegate:
match message {
    Message::Command(cmd_msg) => {
        // Handle command picker messages
        if let Some(response) = self.command_picker.update(cmd_msg) {
            // Handle response
        }
    }
    Message::Pane(pane_msg) => {
        // Handle pane messages
    }
    // ...
}
```

### Pitfall 2: State Duplication
**Problem**: Duplicate state across multiple widgets.

**Solution**: Use shared state with `Arc` or `Rc`:
```rust
use std::sync::Arc;

pub struct App {
    command_state: Arc<Mutex<combo_box::State<String>>>, // Shared state
    panes: pane_grid::State<Pane>,
}

// When creating new panes:
let command_state = Arc::clone(&self.command_state);
let new_pane = Pane::new(command_state);
```

### Pitfall 3: Overly Complex Widgets
**Problem**: Widgets become hard to test and maintain.

**Solution**: Break widgets into smaller components:
```rust
// Instead of one giant widget:
pub struct ComplexWidget {
    // 50 fields
}

// Use composition:
pub struct ComplexWidget {
    header: HeaderWidget,
    content: ContentWidget,
    footer: FooterWidget,
    sidebar: SidebarWidget,
}
```

### Pitfall 4: Performance Issues with Large Lists
**Problem**: ComboBox or list rendering slows down with many items.

**Solution**: Implement virtual scrolling:
```rust
pub struct VirtualList<'a, T> {
    items: &'a [T],
    visible_range: Range<usize>,
    item_height: f32,
    render_item: Box<dyn Fn(&T) -> Element<'a, Message>>,
}

impl<'a, T> VirtualList<'a, T> {
    pub fn new(
        items: &'a [T],
        item_height: f32,
        render_item: impl Fn(&T) -> Element<'a, Message> + 'static,
    ) -> Self {
        Self {
            items,
            visible_range: 0..10.min(items.len()),
            item_height,
            render_item: Box::new(render_item),
        }
    }
    
    pub fn with_visible_range(mut self, range: Range<usize>) -> Self {
        self.visible_range = range;
        self
    }
}
```

---

## Migration Checklist

### Before You Start
- [ ] Set up a new Rust project with Iced 0.15
- [ ] Add required dependencies
- [ ] Set up your project structure
- [ ] Create basic message types

### When Extracting Patterns
- [ ] Copy files incrementally (start with combo_box.rs)
- [ ] Update imports to match your project
- [ ] Replace IRC-specific code with your domain logic
- [ ] Implement your theme catalogs
- [ ] Test each component in isolation

### After Extraction
- [ ] Integrate components into your main app
- [ ] Test the complete flow
- [ ] Optimize performance
- [ ] Add documentation
- [ ] Write tests

### Example Migration Steps

```bash
# Step 1: Set up project
cargo new my-project
cd my-project

# Add dependencies to Cargo.toml
cat >> Cargo.toml << 'EOF'
[dependencies]
iced = { version = "0.15", features = ["advanced"] }
nucleo-matcher = "0.3"

[dependencies]
# Add other dependencies as needed
EOF

# Step 2: Create widget directory
mkdir -p src/widget

# Step 3: Copy combo_box.rs
cp /path/to/halloy/src/widget/combo_box.rs src/widget/

# Step 4: Update imports in combo_box.rs
# Replace:
#   use crate::Theme;
#   use crate::widget;
# With:
#   use iced::widget::text;
#   use iced::{Color, Background, Border, Shadow, Vector, Padding};

# Step 5: Create theme integration
touch src/theme.rs

# Step 6: Implement simple theme
cat > src/theme.rs << 'EOF'
use iced::widget::combo_box;

pub struct Theme;

impl combo_box::Catalog for Theme {
    type Class<'a> = combo_box::Class<'a>;
    
    fn default_input(&self) -> Self::Class<'_> {
        combo_box::Class::default()
            .input(iced::theme::Button::Primary)
    }
    
    fn default_menu(&self) -> Self::Class<'_> {
        combo_box::Class::default()
            .menu(iced::theme::Container::Box)
    }
}
EOF

# Step 7: Create message types
touch src/message.rs

# Step 8: Test compilation
cargo check

# Step 9: Integrate into main app
# Update src/main.rs to use the new combo box
```

---

## Recommended Project Structure

```
my-project/
├── src/
│   ├── main.rs              # Application entry
│   ├── app.rs               # Main app state
│   ├── messages.rs          # Message types
│   ├── theme.rs             # Theme definitions
│   ├── widgets/
│   │   ├── mod.rs           # Widget exports
│   │   ├── combo_box.rs     # Extracted combo box
│   │   ├── modal.rs         # Extracted modal
│   │   ├── pane/
│   │   │   ├── mod.rs       # Pane system
│   │   │   ├── state.rs     # Pane state management
│   │   │   └── grid.rs       # Pane grid rendering
│   │   └── context_menu.rs  # Extracted context menu
│   ├── screens/
│   │   └── main.rs          # Main screen
│   └── utils/
│       ├── fuzzy.rs         # Fuzzy matching utils
│       └── theme.rs         # Theme utilities
├── Cargo.toml
└── README.md
```

---

## Next Steps

### Short Term (1-2 weeks)
1. Extract and test the command picker system
2. Implement basic modal system
3. Create theme integration
4. Set up pane grid for moveable containers

### Medium Term (2-4 weeks)
1. Add context menu support
2. Implement advanced theme customization
3. Add keyboard shortcuts and commands
4. Test on multiple platforms

### Long Term (1+ months)
1. Add accessibility support
2. Implement plugin system
3. Add animations and transitions
4. Optimize performance for large datasets
5. Add visual regression testing

---

## Support and Resources

### Iced Documentation
- [Iced Book](https://book.iced.rs/) - Official documentation
- [Iced Examples](https://github.com/iced-rs/iced/tree/master/examples) - Official examples
- [Iced Widgets](https://docs.rs/iced/latest/iced/widget/) - Widget documentation

### Halloy Resources
- [Halloy GitHub](https://github.com/squidowl/halloy) - Source code
- [Halloy Docs](https://halloy.chat/docs) - User documentation
- [Halloy Discord](https://discord.gg/8b9Z6J2) - Community support

### Learning Resources
- [Elm Architecture Tutorial](https://guide.elm-lang.org/architecture/) - Understanding message-driven architecture
- [Rust GUI Development](https://rust-gui.github.io/) - GUI development in Rust
- [iced-rs Discord](https://discord.gg/iced) - Iced community

---

## Final Notes

The patterns in Halloy are production-tested and designed for extensibility. Start with the essential patterns (command picker, modal, pane grid) and build up from there.

**Remember:**
- Extract incrementally
- Test each component
- Customize for your domain
- Optimize as needed
- Document your changes

Good luck with your project!
