# 🎹 Keyboard Shortcuts & Ctrl+P Implementation Patterns
> Based on Halloy IRC Client's Real-World Implementation

This document explains how **keyboard shortcuts** (including Ctrl+P) and **event subscriptions** work in Halloy, with patterns you can reuse in your own Rust/Iced applications.

---

## 📋 Table of Contents

1. [Core Architecture](#1-core-architecture)
2. [Command System](#2-command-system)
3. [Keyboard Shortcuts (Ctrl+P etc.)](#3-keyboard-shortcuts-ctrlp-etc)
4. [Event Subscription System](#4-event-subscription-system)
5. [Text Editor Key Bindings](#5-text-editor-key-bindings)
6. [Platform-Specific Bindings](#6-platform-specific-bindings)
7. [Configuration & Customization](#7-configuration--customization)
8. [Implementation Patterns](#8-implementation-patterns)
9. [Debugging & Testing](#9-debugging--testing)
10. [Common Mistakes & Solutions](#10-common-mistakes--solutions)
11. [Advanced Patterns](#11-advanced-patterns)
12. [Code Examples for Your App](#12-code-examples-for-your-app)

---

## 🎯 1. Core Architecture

Halloy implements a **3-layer keyboard shortcut system**:

```
Config Layer (data/src/shortcut.rs) 
  ↓
Shortcut System (data::shortcut module)
  ↓
Widget Layer (src/widget/shortcut.rs)
  ↓
Application Layer (src/main.rs)
```

### Key Components:

| Component | Location | Purpose |
|-----------|----------|---------|
| **Command Enum** | `data/src/shortcut.rs` | Defines all available commands |
| **KeyBind Struct** | `data/src/shortcut.rs` | Represents key combinations |
| **Shortcut Struct** | `data/src/shortcut.rs` | Binds KeyBind to Command |
| **Shortcut Widget** | `src/widget/shortcut.rs` | Handles key press events |
| **Config System** | `data/config/` | Loads keybindings from config |

---

## 🎓 2. Command System

### Command Enum Definition

**File**: `data/src/shortcut.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    NewHorizontalBuffer,
    NewVerticalBuffer,
    CloseBuffer,
    MaximizeBuffer,
    RestoreBuffer,
    CycleNextBuffer,
    CyclePreviousBuffer,
    LeaveBuffer,
    ToggleNicklist,
    ToggleTopic,
    ToggleSidebar,
    ToggleFullscreen,
    CommandBar,        // ← This is Ctrl+P!
    ReloadConfiguration,
    FileTransfers,
    Logs,
    ThemeEditor,
    Highlights,
    QuitApplication,
    ScrollUpPage,
    ScrollDownPage,
    ScrollToTop,
    ScrollToBottom,
    CycleNextUnreadBuffer,
    CyclePreviousUnreadBuffer,
    MarkAsRead,
    OpenConfigFile,
}
```

**Key Command**: `CommandBar` → This is what Ctrl+P triggers!

### Default Key Bindings

```rust
impl KeyBind {
    default!(command_bar, "k", COMMAND);  // Ctrl+K or Cmd+K
    
    // Other defaults...
    default!(close_buffer, "w", COMMAND);   // Ctrl+W or Cmd+W
    default!(reload_configuration, "r", COMMAND); // Ctrl+R or Cmd+R
}
```

---

## 🚀 3. Keyboard Shortcuts (Ctrl+P etc.)

### How Ctrl+P Works in Halloy

Halloy uses **Ctrl+K** for the command bar (similar to Slack/Discord), but the pattern applies to any key combination.

### The Complete Flow:

```
User presses: Ctrl+K
  ↓
Event::Keyboard(KeyPressed { key: Named(Enter), modifiers: COMMAND })
  ↓
Shortcut Widget catches the event
  ↓
Finds matching shortcut: KeyBind::command_bar()
  ↓
Executes: Command::CommandBar
  ↓
Main app receives: Message::OpenCommandBar
  ↓
Shows the command picker dropdown
```

### Key Components Explained:

#### 1. KeyBind Definition

```rust
// In data/src/shortcut.rs
pub fn command_bar() -> KeyBind {
    KeyBind::Bind {
        key_code: KeyCode(iced_core::keyboard::Key::Character("k".into())),
        modifiers: COMMAND,  // COMMAND = Ctrl on Windows/Linux, Cmd on macOS
    }
}
```

#### 2. Shortcut Matching

```rust
// In data/src/shortcut.rs
impl Shortcut {
    pub fn execute(&self, key_bind: &KeyBind) -> Option<Command> {
        (self.key_bind == *key_bind).then_some(self.command)
    }
}
```

#### 3. Widget-Level Key Handling

```rust
// In src/widget/shortcut.rs
pub fn shortcut<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    shortcuts: Vec<data::Shortcut>,
    on_press: impl Fn(Command) -> Message + 'a,
) -> Element<'a, Message> {
    decorate(base)
        .update(move |_modifiers, inner, tree, event, layout, cursor, 
                   renderer, clipboard, shell, viewport| {
            if let Event::Keyboard(keyboard::Event::KeyPressed { 
                key, 
                physical_key, 
                modifiers, 
                text, 
                ..
            }) = event {
                let key_bind = shortcut::KeyBind::from((key.clone(), *modifiers));
                
                if let Some(command) = shortcuts.iter()
                    .find_map(|shortcut| shortcut.execute(&key_bind))
                {
                    shell.publish((on_press)(command));
                    shell.capture_event();  // Prevent further handling
                    return;
                }
            }
            
            inner.as_widget_mut().update(/* ... */);
        })
        .into()
}
```

---

## 📡 4. Event Subscription System

### How Events Flow Through Halloy

**File**: `fixtures/halloy/src/main.rs`

```rust
fn subscription(&self) -> Subscription<Message> {
    let tick = iced::time::every(Duration::from_secs(1)).map(Message::Tick);
    let animation_tick = iced::time::every(Duration::from_millis(50))
        .map(Message::AnimationTick);

    let mut subscriptions = vec![
        url::listen().map(Message::RouteReceived),
        events().map(|(window, event)| Message::Event(window, event)),
        window::events()
            .map(|(window, event)| Message::Window(window, event)),
        tick,
        animation_tick,
        streams,
    ];

    Subscription::batch(subscriptions)
}
```

### The `events()` Function

**File**: `fixtures/halloy/src/event.rs`

```rust
pub fn events() -> Subscription<(window::Id, Event)> {
    event::listen_with(filtered_events)
}

fn filtered_events(
    event: iced::Event,
    status: iced::event::Status,
    window: window::Id,
) -> Option<(window::Id, Event)> {
    let event = match &event {
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        }) => Some(Event::Escape),
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Character(c),
            modifiers,
            ..
        }) if c.as_str() == "c" && modifiers.command() => Some(Event::Copy),
        // ... other event filtering
        _ => None,
    };

    event.map(|event| (window, event))
}
```

### Message Flow for Ctrl+P:

```rust
// 1. User presses Ctrl+K
// 2. Iced generates: Event::Keyboard(KeyPressed { key: Character("k"), modifiers: COMMAND })
// 3. events() subscription catches it
// 4. Main app receives: Message::Event(window_id, Event::Keyboard(...))
// 5. Shortcut widget processes it
// 6. Main app receives: Message::CommandBar
// 7. Main app shows command picker
```

---

## ⌨️ 5. Text Editor Key Bindings

Halloy has **two types of key bindings**:

1. **Global shortcuts** (handled by `widget/shortcut.rs`)
2. **Text editor bindings** (handled by `iced::widget::text_editor`)

### Text Editor Key Bindings

**File**: `fixtures/halloy/src/buffer/input_view.rs`

```rust
// In the input view setup
let key_bindings = config.buffer.text_input.key_bindings;

let text_editor = text_editor::TextEditor::new()
    .key_binding(move |key_press| {
        // Platform specific key bindings
        if matches!(key_bindings, KeyBindings::Emacs)
            && let Some(binding) = emacs_key_binding(key_press.clone())
        {
            return Some(binding);
        }

        // Default key bindings
        match key_press.key.as_ref() {
            iced::keyboard::Key::Character("e")
                if key_press.modifiers.control() =>
                Some(text_editor::Binding::Custom(Message::EditorAction(
                    text_editor::Action::Edit(text_editor::Edit::SelectAll)
                ))),
            // ... other text editor bindings
            _ => None,
        }
    })
    .on_action(Message::EditorAction);
```

### Common Text Editor Bindings:

```rust
// Ctrl+A - Select All
// Ctrl+X - Cut
// Ctrl+C - Copy
// Ctrl+V - Paste
// Ctrl+Z - Undo (handled by text_editor)
// Ctrl+Y - Redo (handled by text_editor)
// Ctrl+Enter - Send message
```

---

## 💻 6. Platform-Specific Bindings

Halloy handles different platforms (macOS, Windows, Linux) with conditional key bindings:

```rust
// In data/src/shortcut.rs
impl KeyBind {
    #[cfg(target_os = "macos")]
    default!(toggle_fullscreen, "f", COMMAND | CTRL);
    
    #[cfg(not(target_os = "macos"))]
    default!(toggle_fullscreen, F11);
    
    #[cfg(target_os = "macos")]
    default!(open_config_file, ",", COMMAND);
    
    #[cfg(not(target_os = "macos"))]
    default!(open_config_file, ",", CTRL);
    
    #[cfg(target_os = "linux")]
    default!(quit_application, "q", CTRL);
    
    #[cfg(not(target_os = "linux"))]
    default!(quit_application);
}
```

---

## ⚙️ 7. Configuration & Customization

### Config File Structure

**File**: `config.toml` (example)

```toml
[shortcuts]
# Override default key bindings
command_bar = "p"  # Change Ctrl+K to Ctrl+P
close_buffer = "q"
reload_configuration = "r"

# Add platform-specific bindings
[shortcuts.macos]
command_bar = "p"  # Cmd+P

[shortcuts.windows]
command_bar = "p"  # Ctrl+P
```

### Loading Key Bindings

**File**: `fixtures/halloy/src/config.rs` (simplified)

```rust
pub fn load_keybindings(config: &Config) -> Vec<data::Shortcut> {
    vec![
        data::shortcut(
            config.shortcuts.command_bar.clone(),
            data::Command::CommandBar,
        ),
        data::shortcut(
            config.shortcuts.close_buffer.clone(),
            data::Command::CloseBuffer,
        ),
        // ... other shortcuts
    ]
}
```

---

## 🏗️ 8. Implementation Patterns

### Pattern 1: Basic Shortcut System

```rust
// 1. Define your commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    OpenCommandBar,
    CloseWindow,
    ToggleTheme,
    // ...
}

// 2. Define key bindings
pub fn open_command_bar() -> KeyBind {
    KeyBind::Bind {
        key_code: KeyCode(iced_core::keyboard::Key::Character("p".into())),
        modifiers: COMMAND,  // Ctrl/Cmd
    }
}

// 3. Create shortcuts
let shortcuts = vec![
    data::shortcut(open_command_bar(), Command::OpenCommandBar),
    data::shortcut(
        KeyBind::Bind {
            key_code: KeyCode(iced_core::keyboard::Key::Named(
                iced_core::keyboard::key::Named::Escape
            )),
            modifiers: Modifiers::default(),
        },
        Command::CloseWindow,
    ),
];

// 4. Wrap your app with shortcut widget
let app = shortcut(
    your_main_content(),
    shortcuts,
    |command| Message::Shortcut(command),
);

// 5. Handle in update
match message {
    Message::Shortcut(Command::OpenCommandBar) => {
        self.show_command_picker = true;
        Task::none()
    }
    Message::Shortcut(Command::CloseWindow) => {
        window::close(window_id)
    }
    // ...
}
```

### Pattern 2: Command Bar Pattern (Ctrl+P)

```rust
// Command bar state
pub struct CommandBar {
    state: combo_box::State<Command>,
    is_open: bool,
}

impl CommandBar {
    pub fn new(commands: Vec<Command>) -> Self {
        Self {
            state: combo_box::State::new(commands),
            is_open: false,
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        if self.is_open {
            let combo_box = combo_box::combo_box(
                &self.state,
                "> ",
                None,
                Message::CommandSelected,
            )
            .on_input(Message::CommandInput)
            .on_close(Message::CommandBarClosed);
            
            combo_box.into()
        } else {
            space().into()
        }
    }
    
    pub fn update(&mut self, message: CommandBarMessage) {
        match message {
            CommandBarMessage::Open => self.is_open = true,
            CommandBarMessage::Close => self.is_open = false,
            CommandBarMessage::Select(cmd) => {
                self.is_open = false;
                // Execute command
            }
        }
    }
}

// Usage in main app
let command_bar = CommandBar::new(vec![
    Command::ConnectServer,
    Command::Disconnect,
    Command::JoinChannel("#general".to_string()),
]);

// In view
let content = if self.command_bar_is_open {
    overlay(
        main_content,
        command_bar.view(),
        || Message::CommandBarClosed,
        0.7,
    )
} else {
    main_content
};
```

### Pattern 3: Global Key Handler

```rust
pub struct KeyHandler {
    bindings: HashMap<KeyBind, Message>,
}

impl KeyHandler {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
    
    pub fn bind(&mut self, key_bind: KeyBind, message: Message) {
        self.bindings.insert(key_bind, message);
    }
    
    pub fn handle_event(&self, event: &iced::Event) -> Option<Message> {
        if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            ..
        }) = event {
            let key_bind = KeyBind::from((key.clone(), *modifiers));
            
            self.bindings.get(&key_bind).cloned()
        } else {
            None
        }
    }
}

// Usage
let mut key_handler = KeyHandler::new();
key_handler.bind(
    KeyBind::Bind {
        key_code: KeyCode(iced_core::keyboard::Key::Character("p".into())),
        modifiers: COMMAND,
    },
    Message::OpenCommandPicker,
);

// In subscription
subscription::events_with(|event, _| {
    if let Some(msg) = key_handler.handle_event(&event) {
        Some(msg)
    } else {
        None
    }
})
```

---

## 🔍 9. Debugging & Testing

### Debugging Key Events

```rust
// Add this to your app for debugging
fn debug_key_event(event: &iced::Event) {
    if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        modifiers,
        ..
    }) = event {
        println!("Key pressed: {:?} with modifiers: {:?}", key, modifiers);
    }
}

// Call it in your update or view
```

### Testing Key Bindings

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_bar_binding() {
        let binding = open_command_bar();
        
        // Test Ctrl+K (Windows/Linux)
        assert_eq!(
            binding,
            KeyBind::Bind {
                key_code: KeyCode(iced_core::keyboard::Key::Character("k".into())),
                modifiers: COMMAND,
            }
        );
        
        // Test Ctrl+P (your custom binding)
        let custom_binding = KeyBind::Bind {
            key_code: KeyCode(iced_core::keyboard::Key::Character("p".into())),
            modifiers: COMMAND,
        };
        
        assert!(matches!(custom_binding, KeyBind::Bind { .. }));
    }
    
    #[test]
    fn test_shortcut_execution() {
        let shortcut = data::shortcut(
            open_command_bar(),
            data::Command::CommandBar,
        );
        
        let key_bind = open_command_bar();
        assert_eq!(
            shortcut.execute(&key_bind),
            Some(data::Command::CommandBar)
        );
    }
}
```

---

## ❌ 10. Common Mistakes & Solutions

### Mistake 1: Key Event Not Being Caught

**❌ Problem**: Key events aren't being handled

**✅ Solution**: Make sure your widget is properly wrapped with the shortcut widget:

```rust
// Wrong - missing shortcut wrapper
let content = button("Click me");

// Right - wrap with shortcut widget
let content = shortcut(
    button("Click me"),
    shortcuts,
    |cmd| Message::Shortcut(cmd),
);
```

---

### Mistake 2: Command Not Executing

**❌ Problem**: Command is defined but doesn't execute

**✅ Solution**: Check key binding equality:

```rust
// Wrong - key comparison issues
if key == "p" && modifiers.command() { /* ... */ }

// Right - use KeyBind for proper comparison
let key_bind = KeyBind::from((key.clone(), modifiers));
if let Some(cmd) = shortcuts.iter().find_map(|s| s.execute(&key_bind)) {
    // Command found!
}
```

**Key Issue**: Character case sensitivity (`'p'` vs `'P'`)

**Solution**: KeyBind handles case-insensitive character comparison:

```rust
// In data/src/shortcut.rs
match (&a.0, &b.0) {
    (keyboard::Key::Character(a), keyboard::Key::Character(b)) => 
        a.to_lowercase() == b.to_lowercase(),
    (a, b) => a == b,
}
```

---

### Mistake 3: Event Being Propagated Twice

**❌ Problem**: Shortcut triggers twice

**✅ Solution**: Always capture the event:

```rust
// In shortcut widget
shell.publish((on_press)(command));
shell.capture_event();  // ← This prevents double-triggering
return;  // ← Exit early
```

---

### Mistake 4: Platform-Specific Issues

**❌ Problem**: Ctrl works on Windows but not macOS

**✅ Solution**: Use COMMAND modifier which maps correctly:

```rust
// Wrong - hardcoded Ctrl
modifiers: CTRL,

// Right - use COMMAND for cross-platform
modifiers: COMMAND,  // Ctrl on Windows/Linux, Cmd on macOS
```

---

### Mistake 5: Forgetting to Load Config

**❌ Problem**: Key bindings don't update when config changes

**✅ Solution**: Reload shortcuts on config change:

```rust
impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ConfigReloaded(config) => {
                self.shortcuts = load_keybindings(&config);
                Task::none()
            }
            // ...
        }
    }
}
```

---

## 🎯 11. Advanced Patterns

### Pattern 1: Dynamic Key Bindings

```rust
pub struct DynamicKeyHandler {
    bindings: RwLock<HashMap<KeyBind, Message>>,
}

impl DynamicKeyHandler {
    pub fn update_bindings(&self, new_bindings: HashMap<KeyBind, Message>) {
        *self.bindings.write().unwrap() = new_bindings;
    }
    
    pub fn handle_event(&self, event: &iced::Event) -> Option<Message> {
        // Thread-safe key handling
    }
}
```

### Pattern 2: Chorded Shortcuts (Multi-key)

```rust
pub enum Chord {
    Single(KeyBind),
    Chord(Vec<KeyBind>),
}

impl Chord {
    pub fn matches(&self, events: &[iced::Event]) -> bool {
        // Check if sequence of events matches the chord
    }
}

// Usage: Ctrl+K followed by Ctrl+P
```

### Pattern 3: Context-Sensitive Shortcuts

```rust
pub struct ContextKeyHandler {
    global_bindings: HashMap<KeyBind, Message>,
    context_bindings: HashMap<AppContext, HashMap<KeyBind, Message>>,
}

impl ContextKeyHandler {
    pub fn handle_event(
        &self, 
        context: AppContext, 
        event: &iced::Event
    ) -> Option<Message> {
        let bindings = self.context_bindings.get(&context)
            .unwrap_or(&self.global_bindings);
        
        // Handle event with context-specific bindings
    }
}

// Usage: Different bindings in command bar vs chat input
```

### Pattern 4: Key Binding Conflicts Resolution

```rust
pub struct ConflictResolver {
    priority: HashMap<Command, u32>,  // Higher priority wins
}

impl ConflictResolver {
    pub fn resolve(&self, commands: Vec<Command>) -> Option<Command> {
        commands.into_iter()
            .max_by_key(|cmd| self.priority.get(cmd).unwrap_or(&0))
    }
}

// Usage: Handle when multiple shortcuts match the same key
```

---

## 📚 12. Code Examples for Your App

### Example 1: Basic Command Bar (Ctrl+P)

```rust
// Step 1: Define your commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    OpenCommandBar,
    CloseWindow,
    ToggleFullscreen,
    // Add your commands here
}

// Step 2: Define key bindings
pub fn open_command_bar() -> KeyBind {
    KeyBind::Bind {
        key_code: KeyCode(iced_core::keyboard::Key::Character("p".into())),
        modifiers: COMMAND,  // Ctrl/Cmd
    }
}

// Step 3: Create shortcuts
let shortcuts = vec![
    data::shortcut(open_command_bar(), Command::OpenCommandBar),
    data::shortcut(
        KeyBind::Bind {
            key_code: KeyCode(iced_core::keyboard::Key::Named(
                iced_core::keyboard::key::Named::Escape
            )),
            modifiers: Modifiers::default(),
        },
        Command::CloseWindow,
    ),
    data::shortcut(
        KeyBind::Bind {
            key_code: KeyCode(iced_core::keyboard::Key::Named(
                iced_core::keyboard::key::Named::F11
            )),
            modifiers: Modifiers::default(),
        },
        Command::ToggleFullscreen,
    ),
];

// Step 4: Wrap your app
let app_content = shortcut(
    your_main_content(),
    shortcuts,
    |cmd| Message::Shortcut(cmd),
);

// Step 5: Handle in update
impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Shortcut(Command::OpenCommandBar) => {
                self.command_bar_visible = true;
                Task::none()
            }
            Message::Shortcut(Command::CloseWindow) => {
                window::close(window_id)
            }
            Message::Shortcut(Command::ToggleFullscreen) => {
                window::toggle_fullscreen(window_id)
            }
            _ => Task::none()
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let content = if self.command_bar_visible {
            let command_bar = self.render_command_bar();
            overlay(
                your_main_content(),
                command_bar,
                || Message::CommandBarClosed,
                0.7,
            )
        } else {
            your_main_content()
        };
        
        shortcut(content, self.shortcuts.clone(), |cmd| Message::Shortcut(cmd))
    }
}
```

### Example 2: Command Bar with ComboBox

```rust
// Command bar implementation
pub struct CommandBar {
    state: combo_box::State<AppCommand>,
    is_open: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppCommand {
    ConnectServer(String),
    JoinChannel(String),
    QueryUser(String),
    LeaveChannel,
    ToggleTheme,
    OpenSettings,
    // ...
}

impl CommandBar {
    pub fn new() -> Self {
        let commands = vec![
            AppCommand::ConnectServer("server.example.com".to_string()),
            AppCommand::JoinChannel("#general".to_string()),
            AppCommand::QueryUser("alice".to_string()),
            AppCommand::ToggleTheme,
            AppCommand::OpenSettings,
        ];
        
        Self {
            state: combo_box::State::new(commands),
            is_open: false,
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        if self.is_open {
            let combo_box = combo_box::combo_box(
                &self.state,
                "> ",
                None,
                Message::CommandSelected,
            )
            .on_input(Message::CommandInputChanged)
            .on_close(Message::CommandBarClosed)
            .menu_class(your_theme::combo_box_menu());
            
            combo_box.into()
        } else {
            space().into()
        }
    }
    
    pub fn update(&mut self, message: CommandBarMessage) {
        match message {
            CommandBarMessage::Open => self.is_open = true,
            CommandBarMessage::Close => self.is_open = false,
            CommandBarMessage::InputChanged(input) => {
                // Filter commands based on input
                self.state.filter(&input);
            }
            CommandBarMessage::Select(cmd) => {
                self.is_open = false;
                match cmd {
                    AppCommand::ConnectServer(server) => 
                        Message::ConnectToServer(server),
                    AppCommand::JoinChannel(channel) =>
                        Message::JoinChannel(channel),
                    AppCommand::ToggleTheme => Message::ToggleTheme,
                    // ...
                }
            }
        }
    }
}

// Usage in main app
impl App {
    pub fn handle_keyboard(&mut self, key: keyboard::Key, modifiers: keyboard::Modifiers) {
        let key_bind = KeyBind::from((key, modifiers));
        
        if key_bind == open_command_bar() {
            self.command_bar.update(CommandBarMessage::Open);
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let content = column![
            // Your main app content
            button("Open Command Bar")
                .on_press(Message::CommandBarOpen),
            // ...
        ];
        
        if self.command_bar_visible {
            let command_bar_view = self.command_bar.view()
                .map(|msg| Message::CommandBar(msg));
            
            overlay(content, command_bar_view, || Message::CommandBarClosed, 0.7)
        } else {
            content
        }
    }
}
```

### Example 3: Platform-Aware Key Bindings

```rust
pub struct PlatformKeyHandler {
    bindings: HashMap<&'static str, KeyBind>,
}

impl PlatformKeyHandler {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();
        
        #[cfg(target_os = "macos")]
        {
            bindings.insert("command_bar", KeyBind::Bind {
                key_code: KeyCode(iced_core::keyboard::Key::Character("k".into())),
                modifiers: COMMAND,
            });
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            bindings.insert("command_bar", KeyBind::Bind {
                key_code: KeyCode(iced_core::keyboard::Key::Character("p".into())),
                modifiers: COMMAND,
            });
        }
        
        Self { bindings }
    }
    
    pub fn get(&self, key: &str) -> Option<&KeyBind> {
        self.bindings.get(key)
    }
}

// Usage
let handler = PlatformKeyHandler::new();

#[cfg(target_os = "macos")]
let command_bar_binding = handler.get("command_bar")
    .expect("Command bar binding not found");

#[cfg(not(target_os = "macos"))]
let command_bar_binding = handler.get("command_bar")
    .expect("Command bar binding not found");
```

---

## 🎓 Key Takeaways

### 1. **Use the Shortcut Widget**
Always wrap your content with `src/widget/shortcut.rs` to handle key events properly.

### 2. **Commands Over Key Bindings**
Separate **what** you want to do (Command) from **how** you do it (KeyBind).

### 3. **Platform Awareness**
Use `COMMAND` modifier for cross-platform bindings (Ctrl on Windows/Linux, Cmd on macOS).

### 4. **Event Capturing**
Always call `shell.capture_event()` after handling a shortcut to prevent double-triggering.

### 5. **Configurable Bindings**
Load key bindings from config files for user customization.

### 6. **Text Editor Bindings**
For text input, use Iced's built-in `text_editor::TextEditor` which handles its own key bindings.

### 7. **Debugging**
Add logging to see which keys are being pressed and how they're being handled.

---

## 📊 Comparison with Other Approaches

| Approach | Pros | Cons | Halloy's Choice |
|----------|------|------|----------------|
| **Direct Key Handling** | Simple | Hard to maintain, no config | ❌ No |
| **Command Pattern** | Clean separation | More boilerplate | ✅ Yes |
| **Config-Based** | User customizable | More complex setup | ✅ Yes |
| **Platform-Specific** | Precise control | Harder to maintain | ✅ Yes |

---

## 🚀 Next Steps for Your App

1. **Start with basic shortcuts** (Ctrl+P for command bar)
2. **Add platform-aware bindings**
3. **Implement config loading**
4. **Test on all platforms**
5. **Add user customization**

---

## 📚 Resources

### Halloy References:
- `data/src/shortcut.rs` - Core command system
- `src/widget/shortcut.rs` - Shortcut widget implementation
- `src/event.rs` - Event filtering
- `src/main.rs` - Subscription setup

### Iced Documentation:
- [Iced Keyboard Events](https://docs.rs/iced/latest/iced/keyboard/)
- [Iced Subscriptions](https://docs.rs/iced/latest/iced/advanced/subscription/)
- [Iced Widgets](https://docs.rs/iced/latest/iced/widget/)

### Best Practices:
- [Keyboard Shortcuts UX](https://www.nngroup.com/articles/keyboard-shortcuts/)
- [Accessible Keyboard Navigation](https://developer.mozilla.org/en-US/docs/Web/Accessibility/Guides/Keyboard_navigation)
- [Command Palette Patterns](https://www.nngroup.com/articles/command-palette/)

---

## 🎉 Summary

Halloy's keyboard shortcut system is a **production-ready pattern** that you can reuse in your own Rust/Iced applications. The key components are:

1. **Commands** - Define what actions are available
2. **KeyBinds** - Define how to trigger those actions
3. **Shortcut Widget** - Handles the event-to-command mapping
4. **Config System** - Allows user customization
5. **Platform Awareness** - Works across macOS, Windows, Linux

**For Ctrl+P specifically:**
- Define `Command::CommandBar`
- Bind it to `KeyBind::command_bar()` (Ctrl+K by default, or Ctrl+P if you change it)
- Handle the command in your update function
- Show your command picker UI

This pattern scales from simple apps to complex ones with hundreds of commands!
