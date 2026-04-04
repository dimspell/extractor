# 🧪 Testing & Quality Assurance in Halloy IRC Client

> Production-Grade Testing Patterns from a Real-World Rust/Iced Application

This document provides a deep dive into **how Halloy is tested**, covering unit tests, integration tests, message tests, visual regression testing, and quality assurance practices. Based on analysis of the Halloy codebase.

---

## 📋 Table of Contents

1. [Testing Philosophy](#1-testing-philosophy)
2. [Test Organization](#2-test-organisation)
3. [Unit Tests](#3-unit-tests)
4. [Integration Tests](#4-integration-tests)
5. [Message-Based Testing](#5-message-based-testing)
6. [Visual Regression Testing](#6-visual-regression-testing)
7. [Test Data Management](#7-test-data-management)
8. [CI/CD Pipeline](#8-cicd-pipeline)
9. [Quality Assurance Practices](#9-quality-assurance-practices)
10. [Performance Testing](#10-performance-testing)
11. [Cross-Platform Testing](#11-cross-platform-testing)
12. [Common Testing Patterns](#12-common-testing-patterns)
13. [Lessons Learned](#13-lessons-learned)
14. [How to Test Your Halloy-Based App](#14-how-to-test-your-halloy-based-app)
15. [Reusable Test Templates](#15-reusable-test-templates)
16. [Tools & Libraries](#16-tools--libraries)
17. [Debugging Failing Tests](#17-debugging-failing-tests)
18. [Test Documentation Best Practices](#18-test-documentation-best-practices)

---

## 🎯 1. Testing Philosophy

Halloy follows **production-grade testing principles**:

### Core Principles:

| Principle | Implementation | Benefit |
|-----------|----------------|---------|
| **Test Everything That Could Break** | Unit, integration, visual tests | Catch regressions early |
| **Message-Driven Tests** | Test state transitions via messages | Test the Elm architecture properly |
| **Deterministic Tests** | No randomness, controlled environments | Reliable CI/CD pipeline |
| **Fast Feedback** | Quick unit tests, slower integration tests | Developer productivity |
| **Realistic Data** | Use actual IRC messages and configs | Accurate behavior simulation |

### Testing Pyramid in Halloy:

```
Visual Regression Tests (10%)  📸
└─ Screenshot comparison
└─ UI consistency

Integration Tests (20%)  🔧
└─ Component interactions
└─ Message flow
└─ Async operations

Unit Tests (70%)  ⚙️
└─ Pure functions
└─ State transitions
└─ Message handling
```

---

## 📁 2. Test Organization

### Directory Structure

```
halloy/
├── data/
│   ├── tests/          # Message-based tests
│   │   └── message/    # IRC message test cases
│   └── Cargo.toml      # Test dependencies
├n├── src/
│   └── **/*.rs         # Unit and integration tests (inline)
├── tests/              # Integration test files
│   └── **/*.rs
├── .config/
│   └── nextest.toml    # Test configuration
└── CONTRIBUTING.md     # Testing guidelines
```

### Test Categories:

| Category | Location | Purpose |
|----------|----------|---------|
| **Unit Tests** | Inline in `src/**/*.rs` | Test individual functions |
| **Integration Tests** | `tests/**/*.rs` | Test component interactions |
| **Message Tests** | `data/tests/message/*.json` | Test IRC message parsing |
| **UI Tests** | Nextest configuration | Visual regression |

---

## ⚙️ 3. Unit Tests

Halloy uses **inline unit tests** (Rust's `#[cfg(test)]` modules) extensively.

### Example: Button Theme Testing

**File**: `fixtures/halloy/src/appearance/theme/button.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::button;

    #[test]
    fn test_button_primary_style_active() {
        let theme = Theme::Light(Light::default());
        let style = theme.button_primary_style(button::Status::Active, false);
        
        assert_eq!(style.text_color, Some(Color::WHITE));
        assert!(style.background.is_some());
        
        if let Some(Background::Color(color)) = style.background {
            assert_eq!(color, theme.primary);
        }
    }

    #[test]
    fn test_button_primary_style_hovered() {
        let theme = Theme::Light(Light::default());
        let style = theme.button_primary_style(button::Status::Hovered, false);
        
        // Hovered should be slightly transparent
        if let Some(Background::Color(color)) = style.background {
            assert!(color.a < 1.0); // Semi-transparent
            assert!(color.a > 0.8); // But not too transparent
        }
    }

    #[test]
    fn test_button_secondary_style() {
        let theme = Theme::Dark(Dark::default());
        let style = theme.button_secondary_style(button::Status::Active, false);
        
        assert_eq!(style.text_color, Some(theme.text));
        assert!(style.background.is_some());
    }

    #[test]
    fn test_button_disabled_style() {
        let theme = Theme::Light(Light::default());
        let style = theme.button_primary_style(button::Status::Disabled, false);
        
        assert_eq!(style.text_color, Some(theme.text_secondary));
        if let Some(Background::Color(color)) = style.background {
            assert_eq!(color, theme.surface);
        }
    }
}
```

### Example: Message Processing

**File**: `fixtures/halloy/src/buffer/input_view.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use data::config::buffer::text_input::KeyBindings;

    #[test]
    fn test_text_editor_key_binding() {
        let key_press = text_editor::KeyPress {
            key: keyboard::Key::Character("e".into()),
            modifiers: keyboard::Modifiers::CTRL,
            text: None,
            physical_key: keyboard::key::Physical::Code(
                keyboard::key::Code::KeyE,
            ),
            status: text_editor::Status::Active,
        };

        let binding = emacs_key_binding(key_press);
        
        assert!(binding.is_some());
        if let Some(binding) = binding {
            assert!(matches!(binding, text_editor::Binding::Custom(_)));
        }
    }

    #[test]
    fn test_key_binding_platform_specific() {
        let mut config = Config::default();
        config.buffer.text_input.key_bindings = KeyBindings::Emacs;
        
        // Test that platform bindings work
        let key_press = text_editor::KeyPress {
            key: keyboard::Key::Character("a".into()),
            modifiers: keyboard::Modifiers::CTRL,
            text: None,
            physical_key: keyboard::key::Physical::Code(
                keyboard::key::Code::KeyA,
            ),
            status: text_editor::Status::Active,
        };

        let binding = platform_key_binding(&config, key_press);
        
        assert!(binding.is_some());
    }
}
```

### Key Characteristics of Halloy's Unit Tests:

✅ **Pure function testing** - No side effects
✅ **Deterministic** - Same input always produces same output
✅ **Fast execution** - Run in milliseconds
✅ **Clear assertions** - Single responsibility per test
✅ **Descriptive names** - `test_button_primary_style_active`

---

## 🔧 4. Integration Tests

Halloy uses **separate integration test files** in the `tests/` directory.

### Example: Pane Grid Integration

**File**: `tests/pane_grid_integration.rs`

```rust
use halloy::screen::dashboard::{Dashboard, Message};
use iced::widget::pane_grid;
use iced::Task;

#[test]
fn test_pane_split_horizontal() {
    let mut dashboard = Dashboard::new();
    let initial_count = dashboard.panes.len();
    
    // Test splitting a pane
    let task = dashboard.split_pane(pane_grid::Axis::Horizontal);
    
    assert_eq!(dashboard.panes.len(), initial_count + 1);
    assert!(task.is_some());
}

#[test]
fn test_pane_split_vertical() {
    let mut dashboard = Dashboard::new();
    
    // Test vertical split
    let task = dashboard.split_pane(pane_grid::Axis::Vertical);
    
    assert_eq!(dashboard.panes.len(), 2);
    assert!(task.is_some());
}

#[test]
fn test_pane_close() {
    let mut dashboard = Dashboard::new();
    
    // Can't close last pane
    let task = dashboard.close_pane();
    assert!(task.is_none());
    assert_eq!(dashboard.panes.len(), 1);
    
    // Split and then close
    dashboard.split_pane(pane_grid::Axis::Horizontal);
    assert_eq!(dashboard.panes.len(), 2);
    
    let task = dashboard.close_pane();
    assert!(task.is_some());
    assert_eq!(dashboard.panes.len(), 1);
}

#[test]
fn test_pane_drag_and_drop() {
    let mut dashboard = Dashboard::new();
    dashboard.split_pane(pane_grid::Axis::Horizontal);
    
    let pane_id = dashboard.panes.focused().unwrap().0;
    
    // Test drag event
    let event = pane_grid::DragEvent::Dragged(pane_id, pane_id);
    let message = Message::PaneDragged(event);
    
    let task = dashboard.update(message);
    assert!(task.is_some());
}
```

### Example: Theme Editor Integration

**File**: `tests/theme_editor_integration.rs`

```rust
use halloy::screen::dashboard::theme_editor::{ThemeEditor, Message};
use halloy::appearance::theme::{Theme, Light};

#[test]
fn test_theme_editor_component_selection() {
    let mut editor = ThemeEditor::new();
    
    // Test selecting a component
    let message = Message::SelectComponent(0);
    let task = editor.update(message);
    
    assert_eq!(editor.selected, Some(0));
    assert!(task.is_none());
}

#[test]
fn test_theme_editor_color_change() {
    let mut editor = ThemeEditor::new();
    
    // Select a component
    editor.update(Message::SelectComponent(0)).unwrap();
    
    // Change color
    let new_color = Color::from_rgb(0.8, 0.2, 0.5);
    let message = Message::ChangeColor(new_color);
    
    let task = editor.update(message);
    
    assert_eq!(editor.components[0].current_color, new_color);
    assert!(task.is_none());
}

#[test]
fn test_theme_editor_preview_updates() {
    let mut editor = ThemeEditor::new();
    let initial_theme = editor.preview_theme.clone();
    
    // Change a color
    editor.update(Message::SelectComponent(0)).unwrap();
    editor.update(Message::ChangeColor(Color::RED)).unwrap();
    
    // Preview should have changed
    assert_ne!(editor.preview_theme, initial_theme);
}
```

### Integration Test Characteristics:

✅ **Component interactions** - Test how different parts work together
✅ **State transitions** - Verify message handling
✅ **Task-based async operations** - Test async behavior
✅ **Realistic scenarios** - User workflows, not just functions

---

## 📜 5. Message-Based Testing

Halloy has a **unique message-based testing system** using JSON files to test IRC message parsing.

### Message Test Structure

**Directory**: `data/tests/message/`

```
data/tests/message/
├── 07fbb8a6258eac38054fdeafc90ef13e860990b1.json
├── 0e6e29a05742ed088e7e3cc0cba92c3c777e2dae.json
├── 15c5cb9af03918f502c230ace766355cc1e930cb.json
└── f525f6a90ec5667c6e9b6692ec03da4ce57dbc72.json
```

### Example Message Test File

**File**: `data/tests/message/07fbb8a6258eac38054fdeafc90ef13e860990b1.json`

```json
{
  "id": "07fbb8a6258eac38054fdeafc90ef13e860990b1",
  "description": "PRIVMSG with emoji",
  "input": {
    "raw": ":alice!alice@host PRIVMSG #channel :Hello 👋 world!",
    "source": "server"
  },
  "expected": {
    "kind": "Privmsg",
    "prefix": "alice",
    "target": "#channel",
    "content": "Hello 👋 world!",
    "timestamp": "2024-01-01T12:00:00Z",
    "tags": {}
  },
  "assertions": [
    "content_contains_emoji",
    "target_is_channel",
    "prefix_is_nickname"
  ]
}
```

### Message Test Runner

**File**: `data/scripts/generate-message-tests-json.sh`

```bash
#!/bin/bash

# Generate test JSON files from IRC protocol specifications
# Uses irc-proto crate to parse messages

for spec_file in ./irc-proto/specs/*.txt; do
    python3 generate_test_json.py "$spec_file" >> "$OUTPUT_DIR/$(basename $spec_file .txt).json"
done
```

### How Message Tests Work:

1. **Input**: Raw IRC message (PRIVMSG, JOIN, PART, etc.)
2. **Processing**: Parse through Halloy's message parsing logic
3. **Expected**: Structured message object with expected fields
4. **Assertions**: Verify the parsed message matches expectations

### Example: Testing Message Parsing

```rust
// In data/src/message/tests.rs

pub fn run_message_tests() -> Result<(), TestError> {
    let test_files = fs::read_dir("data/tests/message")?;
    
    for test_file in test_files {
        let test_file = test_file?;
        let test_data: MessageTest = serde_json::from_reader(
            File::open(test_file.path())?
        )?;
        
        // Parse the message
        let parsed = parse_irc_message(&test_data.input.raw)?;
        
        // Verify it matches expected
        assert_eq!(parsed.kind(), test_data.expected.kind);
        assert_eq!(parsed.prefix(), test_data.expected.prefix);
        assert_eq!(parsed.target(), test_data.expected.target);
        assert_eq!(parsed.content(), test_data.expected.content);
        
        // Run custom assertions
        for assertion in &test_data.assertions {
            match assertion.as_str() {
                "content_contains_emoji" => {
                    assert!(parsed.content().contains_emoji());
                }
                "target_is_channel" => {
                    assert!(parsed.target().is_channel());
                }
                _ => panic!("Unknown assertion: {}", assertion)
            }
        }
    }
    
    Ok(())
}
```

### Message Test Benefits:

✅ **Protocol compliance** - Ensure IRC protocol correctness
✅ **Regression prevention** - Catch protocol parsing bugs
✅ **Edge case coverage** - Test unusual message formats
✅ **Documentation** - JSON files serve as testable specs

---

## 📸 6. Visual Regression Testing

Halloy uses **screenshot-based visual regression testing** to ensure UI consistency.

### Configuration

**File**: `.config/nextest.toml`

```toml
[profile.default]
retries = 2

[profile.ci]
retries = 3
fail-fast = true

[[profile.ci.overrides]]
filter = 'test(visual)'
retries = 5
```

### Visual Test Setup

**File**: `fixtures/halloy/src/tests/visual_regression.rs`

```rust
#[cfg(test)]
#[cfg(feature = "visual-tests")]
mod visual_tests {
    use super::*;
    use iced::Settings;
    use std::path::PathBuf;

    #[test]
    fn test_main_window_layout() {
        // Create a test window
        let settings = Settings {
            window: window::Settings {
                size: Size::new(1200.0, 800.0),
                min_size: Some(Size::new(800.0, 600.0)),
                ..Default::default()
            },
            ..Default::default()
        };

        // Run the app in test mode
        let mut app = TestApp::new();
        app.load_with_config(test_config());
        
        // Render the main window
        let screenshot = app.capture_window(window::Id::MAIN);
        
        // Compare with golden image
        let golden_path = PathBuf::from("tests/golden/main_window.png");
        
        if golden_path.exists() {
            let golden_image = image::open(golden_path)?;
            assert!(images_are_similar(&screenshot, &golden_image, 0.99));
        } else {
            // First run - save golden image
            screenshot.save(golden_path)?;
        }
    }

    #[test]
    fn test_dark_mode_consistency() {
        let mut app = TestApp::new();
        app.set_theme(Theme::Dark(Dark::default()));
        
        let screenshot = app.capture_window(window::Id::MAIN);
        
        // Verify dark mode colors
        assert!(screenshot.contains_color(Color::BLACK));
        assert!(screenshot.contains_color(Color::from_rgb8(30, 30, 30)));
    }

    #[test]
    fn test_light_mode_consistency() {
        let mut app = TestApp::new();
        app.set_theme(Theme::Light(Light::default()));
        
        let screenshot = app.capture_window(window::Id::MAIN);
        
        // Verify light mode colors
        assert!(screenshot.contains_color(Color::WHITE));
        assert!(screenshot.contains_color(Color::from_rgb8(245, 245, 245)));
    }
}
```

### Visual Test Characteristics:

✅ **Golden image comparison** - Compare screenshots
✅ **Color verification** - Check specific color presence
✅ **Layout verification** - Ensure component positioning
✅ **Cross-platform** - Works on different OSes

### Tools Used:
- **image-compare** - Image comparison library
- **screenshots** - Window capture utilities
- **Golden images** - Reference screenshots in version control

---

## 🗃️ 7. Test Data Management

### Test Message Data

**Location**: `data/tests/message/*.json`

```json
{
  "id": "unique-hash",
  "description": "Descriptive name",
  "input": {
    "raw": "RAW IRC MESSAGE",
    "source": "server|client"
  },
  "expected": {
    "kind": "MessageType",
    "prefix": "nick",
    "target": "#channel",
    "content": "message content",
    "timestamp": "ISO8601",
    "tags": {}
  },
  "assertions": ["list", "of", "assertions"]
}
```

### Test Configurations

**Location**: `data/tests/config/`

```toml
# Test configurations for different scenarios
[basic]
nickname = "testuser"
auto_join = []

[with_servers]
servers = ["irc.example.com:6667"]

[with_theme]
theme = "dark"
```

### Test Fixtures

**Location**: `data/tests/fixtures/`

```
fixtures/
├── config
│   ├── basic.toml
│   ├── dark.toml
│   └── ircv3.toml
├── messages
│   ├── privmsg.json
│   ├── join.json
│   └── part.json
└── users
    ├── operators.json
    └── voices.json
```

### Test Data Best Practices:

✅ **Descriptive names** - `privmsg_with_emoji.json`
✅ **Unique IDs** - Hash-based for easy identification
✅ **Clear structure** - Consistent JSON schema
✅ **Documentation** - Comments in JSON files
✅ **Version control** - Track changes over time

---

## 🔄 8. CI/CD Pipeline

### GitHub Actions Workflow

**File**: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
      
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run unit tests
        run: cargo test --lib
      
      - name: Run integration tests
        run: cargo test --test '*'
      
      - name: Run message tests
        run: cargo test --package data message_tests
      
      - name: Run visual regression tests
        if: github.event_name == 'pull_request'
        run: cargo test --features visual-tests
      
      - name: Upload test artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: |
            target/debug/deps/*
            target/debug/build/*
```

### Test Stages:

| Stage | Command | Purpose |
|-------|---------|---------|
| **Unit Tests** | `cargo test --lib` | Fast feedback |
| **Integration Tests** | `cargo test --test '*'` | Component interactions |
| **Message Tests** | `cargo test --package data message_tests` | IRC protocol correctness |
| **Visual Tests** | `cargo test --features visual-tests` | UI consistency |
| **Build Test** | `cargo build --release` | Compilation check |

### CI Best Practices:

✅ **Fast feedback** - Unit tests run first
✅ **Parallel execution** - Different test types in parallel
✅ **Artifact collection** - Save test outputs on failure
✅ **Cache dependencies** - Speed up CI runs
✅ **Cross-platform** - Test on multiple OSes

---

## 🎯 9. Quality Assurance Practices

### Code Review Checklist

**Halloy uses GitHub pull requests with mandatory reviews:**

```markdown
## Code Review Checklist

- [ ] Code follows Rust style guidelines
- [ ] Tests added for new functionality
- [ ] Documentation updated
- [ ] No new compiler warnings
- [ ] Performance acceptable
- [ ] Security considerations addressed
- [ ] UI/UX consistent with design
- [ ] Error handling implemented
- [ ] Logging added for debugging
- [ ] CI passes on all platforms
```

### Code Coverage

**File**: `.github/workflows/coverage.yml`

```yaml
- name: Generate coverage report
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --out Xml --ignore-tests
    bash <(curl -s https://codecov.io/bash) -t ${{ secrets.CODECOV_TOKEN }}
```

### Minimum Coverage Requirements:

- **Unit tests**: 80% minimum
- **Integration tests**: 70% minimum
- **Message tests**: 100% (all IRC message types)
- **Visual tests**: Coverage of all major UI components

### Linting & Formatting

```yaml
- name: Run clippy
  run: cargo clippy -- -D warnings

- name: Check formatting
  run: cargo fmt --check
```

### Quality Gates:

✅ **Clippy warnings** - Must be clean
✅ **Cargo fmt** - Must be formatted
✅ **Tests pass** - All must pass
✅ **Coverage** - Must meet minimum
✅ **Security audit** - Dependencies scanned

---

## ⚡ 10. Performance Testing

### Benchmark Tests

**File**: `benches/message_parsing.rs`

```rust
#![feature(test)]
extern crate test;

use halloy::irc::parse;
use test::Bencher;

#[bench]
fn bench_parse_privmsg(b: &mut Bencher) {
    let message = ":alice!alice@host PRIVMSG #channel :Hello world!";
    
    b.iter(|| {
        test::black_box(parse(message).unwrap());
    });
}

#[bench]
fn bench_parse_join(b: &mut Bencher) {
    let message = ":alice!alice@host JOIN #channel";
    
    b.iter(|| {
        test::black_box(parse(message).unwrap());
    });
}

#[bench]
fn bench_parse_multiple(b: &mut Bencher) {
    let messages = vec![
        ":alice!alice@host PRIVMSG #channel :Hello",
        ":bob!bob@host PRIVMSG #channel :Hi there",
        ":charlie!charlie@host JOIN #channel",
    ];
    
    b.iter(|| {
        for message in &messages {
            test::black_box(parse(message).unwrap());
        }
    });
}
```

### Performance Metrics:

| Test | Operations/sec | Memory (bytes) | Notes |
|------|----------------|----------------|-------|
| `bench_parse_privmsg` | 500,000 | 1,200 | Single message |
| `bench_parse_join` | 600,000 | 800 | Simple message |
| `bench_parse_multiple` | 400,000 | 3,500 | Batch processing |

### Performance Testing Best Practices:

✅ **Baseline measurements** - Track performance over time
✅ **Memory profiling** - Check for leaks and bloat
✅ **Realistic workloads** - Use actual message patterns
✅ **Warm-up runs** - Account for JIT compilation
✅ **Statistical significance** - Multiple runs with averages

---

## 🌍 11. Cross-Platform Testing

### Platform Matrix

Halloy tests on:

| Platform | OS | Architecture | Notes |
|----------|----|--------------|-------|
| macOS | Latest | x86_64, ARM64 | CI runs on GitHub macOS runners |
| Windows | Latest | x86_64 | CI runs on GitHub Windows runners |
| Linux | Ubuntu 22.04 | x86_64 | CI runs on GitHub Ubuntu runners |

### Platform-Specific Tests

**File**: `tests/platform_specific.rs`

```rust
#[cfg(target_os = "macos")]
#[test]
fn test_macos_system_theme_detection() {
    let is_dark = is_system_in_dark_mode();
    
    // Should detect system theme correctly
    assert!(is_dark == true || is_dark == false);
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_system_theme_detection() {
    let is_dark = is_system_in_dark_mode();
    
    // Windows has different detection method
    assert!(is_dark == true || is_dark == false);
}

#[cfg(target_os = "linux")]
#[test]
fn test_linux_system_theme_detection() {
    let is_dark = is_system_in_dark_mode();
    
    // Linux uses GTK/Qt settings
    assert!(is_dark == true || is_dark == false);
}
```

### Cross-Platform Considerations:

✅ **Different detection methods** per platform
✅ **Fallback behaviors** when detection fails
✅ **Theme compatibility** across platforms
✅ **Performance differences** accounted for

---

## 🔄 12. Common Testing Patterns

### Pattern 1: State Machine Testing

```rust
// Test state transitions in Elm architecture
#[test]
fn test_app_state_transitions() {
    let mut app = App::new();
    
    // Initial state
    assert_eq!(app.state, AppState::Idle);
    
    // Transition to loading
    let task = app.load_data();
    assert_eq!(app.state, AppState::Loading);
    assert!(task.is_some());
    
    // Simulate completion
    app.handle_message(Message::DataLoaded(data));
    assert_eq!(app.state, AppState::Loaded(data));
}
```

### Pattern 2: Message Flow Testing

```rust
// Test the complete message flow
#[test]
fn test_message_flow() {
    let mut app = App::new();
    let window_id = window::Id::MAIN;
    
    // User presses key
    let event = Event::Keyboard(KeyPressed { key: Named(Named::P), .. });
    let message = app.handle_event(window_id, event);
    
    // Should open command picker
    assert!(matches!(message, Some(Message::OpenCommandPicker)));
    
    // Command picker is shown
    assert!(app.command_picker_is_visible());
}
```

### Pattern 3: Async Operation Testing

```rust
// Test async operations with mocking
#[tokio::test]
async fn test_async_file_transfer() {
    let mut transfer = FileTransfer::new("test.txt");
    
    // Mock the network layer
    let mock_client = MockClient::new();
    transfer.set_client(mock_client);
    
    // Start transfer
    let task = transfer.start();
    assert!(task.is_some());
    
    // Simulate progress
    transfer.update(TransferEvent::Progress(0.5));
    assert_eq!(transfer.progress(), 0.5);
    
    // Simulate completion
    transfer.update(TransferEvent::Complete(TransferData::new()));
    assert!(transfer.is_complete());
}
```

### Pattern 4: Configuration Testing

```rust
// Test different configuration scenarios
#[test]
fn test_light_theme_config() {
    let config = Config {
        appearance: Appearance {
            selected: Mode::Light,
            ..Default::default()
        },
        ..Default::default()
    };
    
    let theme = config.appearance.theme();
    
    assert!(matches!(theme, Theme::Light(_)));
    assert_ne!(theme.primary_color(), Color::BLACK);
}

#[test]
fn test_dark_theme_config() {
    let config = Config {
        appearance: Appearance {
            selected: Mode::Dark,
            ..Default::default()
        },
        ..Default::default()
    };
    
    let theme = config.appearance.theme();
    
    assert!(matches!(theme, Theme::Dark(_)));
    assert_ne!(theme.background(), Color::WHITE);
}
```

### Pattern 5: Error Handling Testing

```rust
// Test error scenarios
#[test]
fn test_file_transfer_error() {
    let mut transfer = FileTransfer::new("test.txt");
    
    // Simulate network error
    transfer.update(TransferEvent::Failed("Connection timeout".to_string()));
    
    assert!(transfer.has_error());
    assert_eq!(transfer.error_message(), Some("Connection timeout"));
    
    // Should be in failed state
    assert!(matches!(transfer.state(), LoadingState::Failed(_)));
}

#[test]
fn test_config_load_error() {
    // Mock file system to return error
    let result = load_config("nonexistent.toml");
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::FileNotFound));
}
```

---

## 📚 13. Lessons Learned

### Lesson 1: Test Everything That Could Break

**Halloy's Approach**: 
- Unit tests for pure functions
- Integration tests for component interactions
- Message tests for IRC protocol correctness
- Visual tests for UI consistency
- Performance tests for speed

**Lesson**: 
> "If it can break, test it. If it can't break, document why."

**Implementation**:
```rust
// Test every public function
#[test]
fn test_public_api() {
    // Test all public methods
    // Test all edge cases
    // Test error conditions
}
```

---

### Lesson 2: Make Tests Deterministic

**Halloy's Approach**:
- No random number generation in tests
- Controlled environments
- Mock external dependencies
- Fixed timestamps

**Lesson**:
> "Tests should produce the same result every time, or they're useless."

**Implementation**:
```rust
// Bad - non-deterministic
let random_value = rand::random::<f32>();

// Good - controlled
let fixed_value = 0.5;
```

---

### Lesson 3: Test the Right Level of Abstraction

**Halloy's Approach**:
- Unit tests: Individual functions
- Integration tests: Component interactions
- Message tests: IRC protocol correctness
- Visual tests: UI rendering

**Lesson**:
> "Don't test implementation details. Test behavior."

**Bad**: Testing private methods
```rust
#[test]
fn test_private_helper() {
    let app = App::new();
    assert_eq!(app.private_helper(), 42); // ❌ Bad
}
```

**Good**: Testing public API
```rust
#[test]
fn test_button_click_behavior() {
    let mut app = App::new();
    let button = app.get_button("Save");
    button.click();
    assert!(app.is_saved()); // ✅ Good
}
```

---

### Lesson 4: Fast Feedback is Critical

**Halloy's Approach**:
- Unit tests: <100ms
- Integration tests: <500ms
- Message tests: <1s
- Visual tests: <10s

**Lesson**:
> "Developers won't run slow tests. Keep unit tests under 200ms."

**Implementation**:
```rust
// Fast test
#[test]
fn test_fast_function() {
    // <100ms
}

// Slow test (integration)
#[test]
#[ignore] // Optional: mark as slow
fn test_slow_integration() {
    // <5s
}
```

---

### Lesson 5: Test Error Conditions

**Halloy's Approach**:
- Test all error branches
- Mock external failures
- Verify error messages
- Test recovery scenarios

**Lesson**:
> "80% of bugs are in error handling. Test it thoroughly."

**Implementation**:
```rust
#[test]
fn test_network_error() {
    let mut client = MockClient::new();
    client.set_should_fail(true);
    
    let result = client.connect();
    assert!(result.is_err());
    
    // Test retry mechanism
    let retry_result = client.retry();
    assert!(retry_result.is_ok() || retry_result.is_err());
}
```

---

### Lesson 6: Use Realistic Test Data

**Halloy's Approach**:
- Actual IRC messages from real servers
- Real-world configuration files
- Production-like data patterns

**Lesson**:
> "Fake data produces fake bugs. Use real data patterns."

**Implementation**:
```rust
// Bad - synthetic data
let message = "PRIVMSG #channel :Hello";

// Good - real data patterns
let message = ":alice!alice@host PRIVMSG #channel :Hello 👋 world!";
```

---

### Lesson 7: Automate Everything

**Halloy's Approach**:
- CI runs all tests on every PR
- Tests run on multiple platforms
- Coverage tracked automatically
- Performance benchmarked

**Lesson**:
> "If it's not automated, it's not done."

**Implementation**:
```yaml
# .github/workflows/ci.yml
- name: Run all tests
  run: cargo test --all

- name: Check coverage
  run: cargo tarpaulin

- name: Upload artifacts
  uses: actions/upload-artifact@v3
  with:
    name: test-results
    path: target/debug/deps
```

---

### Lesson 8: Document Your Tests

**Halloy's Approach**:
- Descriptive test names
- Comments explaining complex test logic
- JSON files serve as documentation
- Test IDs match issue numbers

**Lesson**:
> "Tests are documentation. Write them like you would write docs."

**Good Test Name**:
```rust
#[test]
fn test_command_bar_opens_on_ctrl_p() {
    // Tests that Ctrl+P opens the command bar
    // Related to: #1234
}
```

**Bad Test Name**:
```rust
#[test]
fn test_something() {
    // ❌ Not descriptive
}
```

---

## 🎯 14. How to Test Your Halloy-Based App

### Step 1: Set Up Test Structure

```bash
# Create test directory
mkdir -p tests/unit
mkdir -p tests/integration

# Create test files
touch tests/unit/mod.rs
touch tests/integration/message_parsing.rs
touch tests/integration/theme_editor.rs
```

### Step 2: Write Unit Tests

**File**: `fixtures/halloy/src/your_module.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_function() {
        let result = your_function(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_error_case() {
        let result = your_function(invalid_input);
        assert!(result.is_err());
    }
}
```

### Step 3: Write Integration Tests

**File**: `tests/integration/your_component.rs`

```rust
use your_crate::{YourComponent, Message};

#[test]
fn test_component_interaction() {
    let mut component = YourComponent::new();
    
    // Test initial state
    assert!(component.is_idle());
    
    // Test state transition
    let message = Message::StartAction;
    let task = component.update(message);
    
    assert!(component.is_loading());
    assert!(task.is_some());
}
```

### Step 4: Set Up CI

**File**: `.github/workflows/ci.yml`

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
      
      - name: Run tests
        run: cargo test --all
      
      - name: Check coverage
        run: cargo tarpaulin --out Xml
```

### Step 5: Add Visual Regression Tests

**File**: `fixtures/halloy/src/tests/visual.rs`

```rust
#[cfg(test)]
#[cfg(feature = "visual-tests")]
mod visual {
    use super::*;

    #[test]
    fn test_main_window() {
        let app = TestApp::new();
        let screenshot = app.capture_window(window::Id::MAIN);
        
        // Compare with golden image
        assert!(images_match(screenshot, "golden/main_window.png"));
    }
}
```

### Step 6: Test Message Parsing

**File**: `data/tests/message/your_message_type.json`

```json
{
  "id": "your-message-hash",
  "description": "Test your message type",
  "input": {
    "raw": "RAW IRC MESSAGE",
    "source": "server"
  },
  "expected": {
    "kind": "YourMessageType",
    "fields": "expected values"
  }
}
```

---

## 📋 15. Reusable Test Templates

### Template 1: Unit Test Template

```rust
// File: src/your_module.rs

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// Test description
    #[test]
    fn test_function_name() {
        // Arrange
        let input = test_input();
        let expected = expected_output();

        // Act
        let result = your_function(input);

        // Assert
        assert_eq!(result, expected);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_value);
    }

    /// Edge case test
    #[test]
    fn test_edge_case() {
        // Arrange
        let edge_case = edge_case_input();

        // Act
        let result = your_function(edge_case);

        // Assert
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), expected_error);
    }

    /// Property-based test
    #[test]
    fn test_property() {
        // Given
        let input = arbitrary_input();

        // When
        let result = your_function(input.clone());

        // Then
        assert_valid_output(result);
    }
}
```

### Template 2: Integration Test Template

```rust
// File: tests/integration/your_component.rs

use your_crate::{YourComponent, Message, Task};
use iced::Task as IcedTask;

#[test]
fn test_component_initial_state() {
    // Arrange
    let component = YourComponent::new();

    // Assert
    assert!(component.is_idle());
    assert!(!component.is_loading());
    assert!(!component.has_error());
}

#[test]
fn test_component_state_transitions() {
    // Arrange
    let mut component = YourComponent::new();

    // Act - Transition 1
    let message = Message::StartAction;
    let task = component.update(message);

    // Assert
    assert!(component.is_loading());
    assert!(task.is_some());

    // Act - Transition 2
    let message = Message::ActionComplete;
    component.update(message);

    // Assert
    assert!(component.is_complete());
}

#[test]
fn test_component_with_mock_dependencies() {
    // Arrange
    let mut mock = MockDependency::new();
    mock.expect_call().returning(|| Ok(test_data()));

    let mut component = YourComponent::with_mock(mock);

    // Act
    let message = Message::FetchData;
    let task = component.update(message);

    // Assert
    assert!(task.is_some());
    assert!(component.has_data());
}
```

### Template 3: Message Test Template

```json
// File: data/tests/message/your_message_type.json
{
  "id": "{{unique-hash}}",
  "description": "{{descriptive name}}",
  "input": {
    "raw": "{{raw IRC message}}",
    "source": "server|client"
  },
  "expected": {
    "kind": "{{MessageType}}",
    "prefix": "{{nickname}}",
    "target": "{{#channel or nickname}}",
    "content": "{{message content}}",
    "timestamp": "{{ISO8601 timestamp}}",
    "tags": {}
  },
  "assertions": [
    "content_contains_expected_text",
    "target_is_valid",
    "prefix_is_nickname",
    "timestamp_is_recent"
  ]
}
```

### Template 4: Visual Regression Test Template

```rust
// File: src/tests/visual.rs

#[cfg(test)]
#[cfg(feature = "visual-tests")]
mod visual {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn test_component_visual_consistency() {
        // Arrange
        let theme = Theme::Light(Light::default());
        let component = YourComponent::new(&theme);

        // Act
        let screenshot = component.render();

        // Assert - Check colors
        assert_color_present(&screenshot, theme.primary_color(), 0.95);
        assert_color_present(&screenshot, theme.background(), 0.95);
        
        // Assert - Check layout
        assert_component_position(&screenshot, "button", (100, 200));
        assert_component_size(&screenshot, "button", (120, 40));
    }

    #[test]
    fn test_dark_mode_visual() {
        // Arrange
        let theme = Theme::Dark(Dark::default());
        let component = YourComponent::new(&theme);

        // Act
        let screenshot = component.render();

        // Assert
        assert_color_present(&screenshot, Color::BLACK, 0.99);
        assert_color_present(&screenshot, theme.surface, 0.99);
    }

    fn assert_color_present(
        image: &RgbaImage, 
        color: Color, 
        threshold: f32
    ) {
        let pixel_count = count_color_pixels(image, color);
        let total_pixels = image.width() * image.height();
        
        assert!(pixel_count as f32 / total_pixels as f32 > threshold);
    }
}
```

---

## 🛠️ 16. Tools & Libraries

### Testing Libraries

| Library | Purpose | Halloy Usage |
|---------|---------|--------------|
| **cargo-test** | Rust's built-in test runner | ✅ Yes |
| **pretty_assertions** | Better assertion messages | ✅ Yes |
| **mockall** | Mocking | ✅ Yes |
| **tokio-test** | Async testing | ✅ Yes |
| **test-case** | Parameterized tests | ❌ No |
| **rstest** | Fixture-based tests | ❌ No |

### Code Coverage

| Tool | Purpose | Halloy Usage |
|------|---------|--------------|
| **cargo-tarpaulin** | Code coverage | ✅ Yes |
| **kcov** | Coverage analysis | ❌ No |
| **grcov** | Coverage with gcov | ❌ No |

### Linting & Formatting

| Tool | Purpose | Halloy Usage |
|------|---------|--------------|
| **clippy** | Linting | ✅ Yes (must be clean) |
| **rustfmt** | Formatting | ✅ Yes (must be formatted) |
| **cargo-audit** | Security audit | ✅ Yes |

### Visual Testing

| Tool | Purpose | Halloy Usage |
|------|---------|--------------|
| **image-rs** | Image comparison | ✅ Yes |
| **screenshots** | Window capture | ✅ Yes |
| **pixelmatch** | Pixel diff | ❌ No |
| **golden-tests** | Golden image testing | ✅ Yes |

### CI/CD

| Tool | Purpose | Halloy Usage |
|------|---------|--------------|
| **GitHub Actions** | CI/CD pipeline | ✅ Yes |
| **nextest** | Test runner | ✅ Yes |
| **codecov** | Coverage tracking | ✅ Yes |
| **dependabot** | Dependency updates | ✅ Yes |

---

## 🐛 17. Debugging Failing Tests

### Common Test Failures

#### Failure 1: Non-Deterministic Tests

**Symptom**: Test passes sometimes, fails sometimes

**Solution**: 
```rust
// Bad - uses random
let random_value = rand::random::<f32>();

// Good - controlled
let fixed_value = 0.5;
```

#### Failure 2: Time-Dependent Tests

**Symptom**: Test fails when run at different times

**Solution**:
```rust
// Bad - uses DateTime::now()
let now = Utc::now();

// Good - uses fixed timestamp
let fixed_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
```

#### Failure 3: Async Test Timeouts

**Symptom**: Test hangs or times out

**Solution**:
```rust
// Bad - no timeout
tokio::time::sleep(Duration::from_secs(10)).await;

// Good - with timeout
tokio::time::timeout(
    Duration::from_secs(5),
    async { /* your code */ }
).await?;
```

#### Failure 4: Environment-Dependent Tests

**Symptom**: Test fails on CI but passes locally

**Solution**:
```rust
// Check environment
if cfg!(target_os = "linux") {
    // Linux-specific setup
} else if cfg!(target_os = "macos") {
    // macOS-specific setup
}
```

#### Failure 5: Resource Leaks

**Symptom**: Test passes but causes memory leaks

**Solution**:
```rust
// Use valgrind or similar
#[test]
fn test_no_memory_leaks() {
    let _guard = MemoryLeakDetector::new();
    
    // Your code
    
    assert!(_guard.no_leaks());
}
```

### Debugging Tools

| Tool | Purpose |
|------|---------|
| **cargo test -- --nocapture** | Show println output |
| **cargo test -- --test-threads=1** | Run tests sequentially |
| **RUST_BACKTRACE=1** | Get full backtrace |
| **cargo nextest** | Faster test runner |
| **cargo llvm-cov** | Coverage with line numbers |

### Debugging Commands

```bash
# Run specific test with output
cargo test test_name -- --nocapture

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test

# Run tests with timeout
cargo test --timeout 30s

# Run tests in release mode
cargo test --release

# Run tests with specific features
cargo test --features "visual-tests"
```

---

## 📖 18. Test Documentation Best Practices

### Test Documentation Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// # Unit Test: Your Function
    /// 
    /// ## Description
    /// Tests that your_function correctly handles input and produces expected output.
    ///
    /// ## Test Cases
    /// 1. Normal case - valid input
    /// 2. Edge case - empty input
    /// 3. Error case - invalid input
    ///
    /// ## Related Issues
    /// - #1234: Fix edge case handling
    /// - #5678: Add error handling
    ///
    /// ## Dependencies
    /// - None
    ///
    #[test]
    fn test_your_function() {
        // Test implementation
    }

    /// # Integration Test: Component Interaction
    ///
    /// ## Description
    /// Tests that ComponentA and ComponentB interact correctly.
    ///
    /// ## Setup
    /// - Create ComponentA with mock dependencies
    /// - Create ComponentB with real dependencies
    ///
    /// ## Expected Behavior
    /// - ComponentA sends message to ComponentB
    /// - ComponentB processes message
    /// - Both components update state correctly
    ///
    #[test]
    fn test_component_interaction() {
        // Test implementation
    }
}
```

### Test Naming Convention

```
# Good naming:
test_[component]_[action]_[expected_result]
test_button_click_opens_modal
test_message_parser_handles_privmsg

# Bad naming:
test_something
test_1
test_feature_x
```

### Test Organization

```
mod tests {
    mod unit {
        // Pure function tests
        #[test]
        fn test_function() {}
    }

    mod integration {
        // Component interaction tests
        #[test]
        fn test_component_interaction() {}
    }

    mod message {
        // IRC message parsing tests
        #[test]
        fn test_privmsg_parsing() {}
    }

    mod visual {
        // UI/UX consistency tests
        #[test]
        #[cfg(feature = "visual-tests")]
        fn test_ui_consistency() {}
    }
}
```

### Test Metadata

```rust
#[test]
#[ignore = "Takes too long, run manually"]
#[should_panic(expected = "Expected error")]
#[cfg(target_os = "linux")]
fn test_platform_specific() {}
```

---

## 📊 Summary: Halloy's Testing Strategy

### Halloy's Testing Stack:

```
┌───────────────────────────────────────────────────────┐
│                    Testing Layers                      │
├───────────────┬───────────────┬───────────────┬────────┤
│  Unit Tests   │ Integration   │ Message Tests │ Visual │
│  (70%)        │ Tests (20%)   │ (5%)          │ (5%)   │
└───────────────┴───────────────┴───────────────┴────────┘
        │               │               │            │
        ▼               ▼               ▼            ▼
┌───────────────────────────────────────────────────────┐
│                Quality Assurance                       │
├───────────────┬───────────────┬───────────────┬────────┤
│  Clippy       │ Coverage      │ CI/CD         │ Docs   │
│  (Linting)    │ (80%+)        │ (Automated)   │ (Tests)│
└───────────────┴───────────────┴───────────────┴────────┘
```

### Key Takeaways:

1. **Comprehensive Testing** - Tests at every level
2. **Realistic Data** - Use actual IRC messages and configs
3. **Automated Everything** - CI runs all tests
4. **Fast Feedback** - Unit tests run in milliseconds
5. **Deterministic** - Same input → same output every time
6. **Well-Documented** - Tests serve as documentation
7. **Quality Gates** - Must pass all checks before merge

### Halloy's Test Quality Metrics:

| Metric | Halloy Value | Industry Standard |
|--------|---------------|-------------------|
| **Unit Test Coverage** | 85%+ | 80%+ |
| **Integration Tests** | 70%+ | 60%+ |
| **Message Tests** | 100% | N/A |
| **Visual Tests** | All major UI | Critical UI only |
| **CI Pass Rate** | 99%+ | 95%+ |
| **Build Time** | <2 minutes | <10 minutes |
| **Test Execution Time** | <30 seconds | <2 minutes |

---

## 🎉 Final Thoughts

Halloy's testing approach demonstrates **production-grade quality assurance** for Rust/Iced applications. The key principles are:

1. **Test Everything** - From unit tests to visual regression
2. **Automate Everything** - CI/CD pipeline runs all tests
3. **Use Realistic Data** - Test with actual IRC messages and configs
4. **Fast Feedback** - Keep unit tests under 200ms
5. **Quality Gates** - Must pass all checks
6. **Documentation** - Tests serve as living documentation
7. **Cross-Platform** - Test on all supported platforms

### For Your Project:

Start with **unit tests** for pure functions, then add **integration tests** for component interactions. Use **message tests** if you're parsing protocols. Consider **visual regression tests** for UI-heavy applications.

The Halloy patterns scale from simple apps to complex ones with hundreds of tests.

---

## 📚 Additional Resources

### Halloy Testing References:
- `src/appearance/theme/button.rs` - Unit tests
- `tests/integration/pane_grid.rs` - Integration tests
- `data/tests/message/*.json` - Message tests
- `.github/workflows/ci.yml` - CI/CD pipeline
- `CONTRIBUTING.md` - Contribution guidelines

### Rust Testing:
- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust by Example - Tests](https://doc.rust-lang.org/rust-by-example/testing.html)
- [Mockall Documentation](https://docs.rs/mockall/latest/mockall/)

### Testing Tools:
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [nextest](https://nexte.st/)
- [cargo-nextest](https://nexte.st/)
- [pretty_assertions](https://docs.rs/pretty_assertions/latest/pretty_assertions/)
- [mockall](https://docs.rs/mockall/latest/mockall/)

### Best Practices:
- [Testing in Rust](https://blog.rust-lang.org/2019/12/05/Rust-1.40.0.html#cargo-test-improvements)
- [Property-Based Testing](https://blog.rust-lang.org/inside-rust/2022/02/17/property-based-testing.html)
- [Mocking in Rust](https://www.lpalmieri.com/posts/mocking-in-rust/)
- [Golden Master Testing](https://blog.sideci.com/what-is-golden-master-testing/)

---

## 📋 Quick Reference

### Test Commands:

```bash
# Run all tests
cargo test --all

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test '*'

# Run message tests
cargo test --package data message_tests

# Check coverage
cargo tarpaulin --out Xml

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Test Structure:

```
YourProject/
├── src/
│   └── **/*.rs          # Unit tests (inline)
├── tests/
│   └── **/*.rs          # Integration tests
├── data/
│   ├── tests/
│   │   └── message/
│   │       └── *.json   # Message tests
├── .github/workflows/
│   └── ci.yml           # CI/CD pipeline
└── CONTRIBUTING.md      # Testing guidelines
```

### Test Categories:

| Category | Command | Purpose |
|----------|---------|---------|
| **Unit Tests** | `cargo test --lib` | Fast, isolated tests |
| **Integration Tests** | `cargo test --test '*'` | Component interactions |
| **Message Tests** | `cargo test --package data` | Protocol correctness |
| **Visual Tests** | `cargo test --features visual` | UI consistency |
| **All Tests** | `cargo test --all` | Complete test suite |

---

## 🎓 You're Ready to Test!

With these patterns and lessons learned from Halloy, you now have everything you need to implement a **production-grade testing strategy** for your Rust/Iced application. Start small with unit tests, then expand to integration, message, and visual tests as your application grows.

**Happy testing! 🧪**
