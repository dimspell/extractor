# 🎓 Lessons Learned: Implementing Loaders & Progress Indicators

> **Based on real-world patterns from Halloy IRC Client**
> *A modern IRC client built with Rust and Iced-RS*

This document distills the **key lessons learned** from analyzing Halloy's implementation of loaders, progress bars, and loading states. It covers:

- ✅ **Core concepts** behind effective loading indicators
- ✅ **Halloy's actual implementations** with code examples
- ✅ **Best practices** for different types of loading states
- ✅ **Common pitfalls** and how to avoid them
- ✅ **Reusable patterns** you can apply immediately
- ✅ **AI-friendly explanations** for code generation

---

## 📋 Table of Contents

1. [Core Concepts: What Makes a Good Loader](#1-core-concepts-what-makes-a-good-loader)
2. [Types of Loading States in Halloy](#2-types-of-loading-states-in-halloy)
3. [File Transfer Progress Bars](#3-file-transfer-progress-bars)
4. [Typing Indicators (Pulse Animations)](#4-typing-indicators-pulse-animations)
5. [Preview Loading States](#5-preview-loading-states)
6. [Configuration Reload Indicators](#6-configuration-reload-indicators)
7. [Spinner vs Progress Bar vs Pulse](#8-spinner-vs-progress-bar-vs-pulse)
8. [State Management for Loaders](#9-state-management-for-loaders)
9. [Theme Integration](#10-theme-integration)
10. [Performance Considerations](#11-performance-considerations)
11. [Testing Loaders](#12-testing-loaders)
12. [Reusable Patterns Library](#13-reusable-patterns-library)
13. [Common Mistakes & Solutions](#14-common-mistakes--solutions)
14. [When NOT to Use Loaders](#15-when-not-to-use-loaders)
15. [Advanced Patterns](#16-advanced-patterns)

---

## 🎯 1. Core Concepts: What Makes a Good Loader

### The 5 Principles of Effective Loading Indicators

#### 1. **Provide Immediate Feedback**
> "The interface should respond within 100ms to user actions. If loading takes longer, show a loader."

**Halloy Lesson**: Even simple operations like typing indicators use animation to provide instant feedback that the system is responsive.

```rust
// From src/buffer/typing.rs
let indicator = animate(
    start.elapsed(),
    Duration::from_millis(1000),
    |progress| pulse(progress),
);
```

#### 2. **Indicate Duration & Progress**
> "Users should understand if a process will take seconds vs minutes."

**Halloy Lesson**: Progress bars show exact percentage for long operations (file transfers), while pulse animations indicate ongoing activity without suggesting completion time.

#### 3. **Maintain Visual Hierarchy**
> "Loaders should be noticeable but not overwhelming."

**Halloy Lesson**: Loaders use subtle animations and colors that don't compete with primary content.

#### 4. **Preserve Context**
> "Don't obscure the content being loaded."

**Halloy Lesson**: Overlays are semi-transparent, progress bars don't cover the entire screen.

#### 5. **Support Accessibility**
> "Screen readers should announce loading states."

**Halloy Lesson**: Text-based indicators (like "*") are screen-reader friendly.

---

## 📊 2. Types of Loading States in Halloy

Halloy implements **4 distinct types** of loading indicators:

| Type | Purpose | Example | File | Pattern |
|------|---------|---------|------|---------|
| **Progress Bar** | Shows exact completion percentage | File transfers | `src/buffer/file_transfers.rs` | Determinate |
| **Pulse Animation** | Shows ongoing activity | Typing indicators | `src/buffer/typing.rs` | Indeterminate |
| **Loading State** | Shows "loading..." placeholder | Preview images | `src/screen/dashboard.rs` | State-based |
| **Reload Indicator** | Shows configuration reload | Config reload | `src/screen/dashboard/sidebar.rs` | Status-based |

### Visual Comparison

```
Progress Bar: [=======>      ] 65%  (Determinate)

Pulse Animation: ••••• (Blinking dots)  (Indeterminate)

Loading State: "Loading preview..." text with spinner icon

Reload Indicator: 🔄 Reloading configuration...
```

---

## 📥 3. File Transfer Progress Bars

### Halloy's Implementation

**File**: `fixtures/halloy/src/buffer/file_transfers.rs`

```rust
use iced::widget::{column, container, progress_bar, row, text};

// In the view function for each transfer
let progress_bar = container(progress_bar(
    0.0,  // min
    1.0,  // max
    transfer.progress() as f32,  // current value (0.0-1.0)
))
.style(theme::progress_bar::progress(theme))
.height(8);

let content = row![
    text(&transfer.filename),
    horizontal_space(),
    text(format!("{:.1} MB", transfer.received / 1_000_000)),
    progress_bar,
    text(format!("{:.1}%".transfer.progress() * 100.0)),
];
```

### Key Design Decisions

#### ✅ **Determinate Progress**
- Shows exact percentage (0-100%)
- Users know exactly how much is left
- Updates in real-time as file transfers progress

#### ✅ **Compact Layout**
```rust
row![
    filename,       // Left-aligned
    space(),        // Flexible space
    size,           // Current size
    progress_bar,   // Centered progress
    percentage,     // Right-aligned
]
```

#### ✅ **Theme Integration**
```rust
// src/appearance/theme/progress_bar.rs
use iced::widget::progress_bar::{Catalog, Style, StyleFn};

impl Catalog for Theme {
    type Class<'a> = progress_bar::Class<'a>;
    
    fn appearance(&self) -> progress_bar::Appearance {
        progress_bar::Appearance {
            background: Background::Color(self.surface),
            bar: Background::Color(self.primary.with_alpha(0.7)),
            border_radius: 4.0.into(),
        }
    }
}
```

### When to Use This Pattern

**Use file transfer progress bars when:**
- ✅ You have operations with known duration
- ✅ You can calculate progress percentage
- ✅ The operation takes >2 seconds
- ✅ Multiple parallel operations need individual tracking

**Don't use for:**
- ❌ Operations with unknown duration
- ❌ Very short operations (<1 second)
- ❌ Batch operations where individual progress isn't meaningful

---

## ⌨️ 4. Typing Indicators (Pulse Animations)

### Halloy's Implementation

**File**: `fixtures/halloy/src/buffer/typing.rs`

```rust
use iced::widget::text;

// In the view function
let indicator = animate(
    start.elapsed(),
    Duration::from_millis(1000),  // Animation duration
    |progress| pulse(progress),   // Animation function
);

if let Some(indicator) = indicator {
    row![text, indicator].into()
} else {
    text.into()
}

// Animation function
fn pulse(progress: f32) -> f32 {
    let fade = if progress <= 0.5 {
        ease_in_out_sine(progress * 2.0)
    } else {
        ease_in_out_sine((1.0 - progress) * 2.0)
    };
    fade
}

fn ease_in_out_sine(progress: f32) -> f32 {
    0.5 * (1.0 - (PI * progress.clamp(0.0, 1.0)).cos())
}
```

### Visual Representation

```
Normal state: "Hello"

Typing state: "Hello•" (dot appears and pulses)

Typing state: "Hello••" (two dots appear)

Typing state: "Hello•••" (three dots appear and pulse together)
```

### Key Design Decisions

#### ✅ **Indeterminate Animation**
- No percentage shown (impossible to calculate)
- Suggests "activity is happening" without implying completion time
- Uses smooth easing functions for natural feel

#### ✅ **Subtle Integration**
- Added to existing text without covering content
- Uses simple dot characters (screen-reader friendly)
- Animation loops indefinitely until typing stops

#### ✅ **Performance Optimized**
- Uses `animate()` helper for efficient rendering
- Only renders when needed (when typing is detected)
- Lightweight CSS-style animation (no heavy computation)

### When to Use This Pattern

**Use typing indicators when:**
- ✅ Showing remote user is typing
- ✅ Waiting for user input in forms
- ✅ Background processes are running
- ✅ Any indeterminate duration operation

**Don't use for:**
- ❌ Operations with known completion time
- ❌ Success/failure states (use checkmarks/X instead)
- ❌ When you need exact progress

---

## 🖼️ 5. Preview Loading States

### Halloy's Implementation

**File**: `fixtures/halloy/src/screen/dashboard.rs`

```rust
// State tracking
pub struct Dashboard {
    previews: preview::Collection,
    // ...
}

// When loading a preview
self.previews.insert(url.clone(), preview::State::Loading);

// In the view function
match self.previews.get(&url) {
    Some(preview::State::Loading) => {
        container(
            column![
                spinner(),
                text("Loading preview..."),
            ]
            .align_items(Alignment::Center)
            .spacing(10),
        )
        .into()
    }
    Some(preview::State::Loaded(content)) => {
        // Show the actual preview content
        content_view(content)
    }
    Some(preview::State::Failed(error)) => {
        error_view(error)
    }
    None => {
        // No preview available
        placeholder_view()
    }
}
```

### Key Design Decisions

#### ✅ **State Machine Pattern**
```rust
pub enum State {
    Loading,      // Initial state
    Loaded(T),    // Success with content
    Failed(String), // Error with message
}
```

#### ✅ **Spinner Animation**
```rust
fn spinner() -> Element<Message> {
    // Simple rotating spinner using text
    text("⠋")
        .size(20)
        .style(theme::text::secondary(theme))
}
```

#### ✅ **Progressive Enhancement**
- Start with placeholder
- Transition to loading state
- Finally show content or error
- Never block the UI

### When to Use This Pattern

**Use preview loading states when:**
- ✅ Loading external content (images, videos, web previews)
- ✅ Waiting for network responses
- ✅ Processing data that takes noticeable time
- ✅ Providing fallback for slow operations

**Don't use for:**
- ❌ Immediate local operations
- ❌ Operations that complete in <100ms
- ❌ When content is always available

---

## 🔄 6. Configuration Reload Indicators

### Halloy's Implementation

**File**: `fixtures/halloy/src/screen/dashboard/sidebar.rs`

```rust
pub struct Sidebar {
    reloading_config: bool,
    // ... other fields
}

impl Sidebar {
    pub fn reload_configuration(&mut self) -> Task<Message> {
        self.reloading_config = true;
        
        Task::perform(
            async move { reload_config().await },
            |result| match result {
                Ok(_) => Message::ConfigReloaded,
                Err(e) => Message::ConfigReloadError(e),
            }
        )
    }
    
    pub fn view(&self) -> Element<Message> {
        if self.reloading_config {
            container(
                row![
                    spinner(),
                    text("Reloading configuration..."),
                ]
                .spacing(8),
            )
            .style(theme::container::secondary(theme))
            .into()
        } else {
            // Normal view
        }
    }
}
```

### Key Design Decisions

#### ✅ **Status Flag Pattern**
- Simple boolean flag tracks loading state
- Easy to implement and understand
- Works well with Task-based async operations

#### ✅ **Non-Blocking**
- UI remains interactive during reload
- Users can still navigate and use other features
- Loading state is subtle (not a modal)

#### ✅ **Error Handling**
```rust
match result {
    Ok(_) => Message::ConfigReloaded,
    Err(e) => Message::ConfigReloadError(e),
}
```

### When to Use This Pattern

**Use reload indicators when:**
- ✅ Configuration changes need to be applied
- ✅ Application state needs to be refreshed
- ✅ External dependencies need to be reloaded
- ✅ User-initiated operations that may take time

**Don't use for:**
- ❌ Automatic background operations
- ❌ Operations that don't affect user experience
- ❌ When the operation is guaranteed to succeed quickly

---

## 🎨 7. Spinner vs Progress Bar vs Pulse

### Comparison Table

| Type | Determinate | Visual | Use Case | Halloy Example |
|------|-------------|--------|----------|----------------|
| **Progress Bar** | ✅ Yes | [====>    ] 45% | Known duration, partial completion | File transfers |
| **Spinner** | ❌ No | ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ | Unknown duration, ongoing | Preview loading |
| **Pulse** | ❌ No | ••• (pulsing) | Ongoing activity, multiple items | Typing indicators |
| **Status Text** | ❌ No | "Loading..." | Simple feedback | Configuration reload |

### Visual Examples

#### Progress Bar (Determinate)
```
[█████████████████░░░░░░░░] 75%
```
**Properties**:
- Shows exact percentage
- Updates as operation progresses
- Uses color to indicate progress
- Best for: File transfers, downloads, uploads

#### Spinner (Indeterminate)
```
⠋ Loading...
⠙ Loading...
⠹ Loading...
⠸ Loading...
```
**Properties**:
- Rotating animation
- No percentage shown
- Suggests "working" without implying completion time
- Best for: Network requests, API calls, preview loading

#### Pulse Animation (Indeterminate)
```
Hello•   (dot appears and pulses)
Hello••  (two dots)
Hello••• (three dots, all pulse together)
```
**Properties**:
- Simple dot characters
- Smooth fade in/out
- Multiple dots can appear
- Best for: Typing indicators, background processes

#### Status Text (Simple)
```
🔄 Reloading configuration...
✓ Configuration reloaded successfully
✗ Failed to reload configuration
```
**Properties**:
- Text-based
- Can include icons
- Easy to understand
- Best for: Simple status updates

---

## 🗃️ 8. State Management for Loaders

### The 3-State Pattern (Halloy's Approach)

```rust
pub enum LoadingState<T> {
    Idle,           // No operation in progress
    Loading,        // Operation started, no result yet
    Loaded(T),      // Operation completed successfully
    Failed(String), // Operation failed with error
}
```

### Implementation Examples

#### File Transfer State
```rust
// src/buffer/file_transfers.rs
pub struct FileTransfer {
    state: LoadingState<TransferData>,
    progress: f32,
    // ... other fields
}

impl FileTransfer {
    pub fn update(&mut self, event: TransferEvent) {
        match event {
            TransferEvent::Progress(percent) => {
                self.progress = percent;
            }
            TransferEvent::Complete(data) => {
                self.state = LoadingState::Loaded(data);
            }
            TransferEvent::Failed(error) => {
                self.state = LoadingState::Failed(error);
            }
        }
    }
}
```

#### Preview State
```rust
// src/screen/dashboard.rs
pub enum PreviewState {
    Idle,
    Loading,
    Loaded(PreviewContent),
    Failed(String),
}

self.previews.insert(url, PreviewState::Loading);
```

### Key Design Principles

#### ✅ **Separation of Concerns**
- Loading state is separate from content
- State transitions are explicit
- Easy to add new states

#### ✅ **Immutable Transitions**
```rust
// Bad - mutating state
self.loading = true;
self.progress = 0.5;

// Good - state transitions
self.state = match event {
    Event::Progress(p) => LoadingState::Loading(p),
    Event::Complete(data) => LoadingState::Loaded(data),
};
```

#### ✅ **Task-Based Async Handling**
```rust
Task::perform(
    async { load_data().await },
    |result| match result {
        Ok(data) => Message::DataLoaded(data),
        Err(e) => Message::LoadFailed(e),
    }
)
```

---

## 🎨 9. Theme Integration

### Progress Bar Styling

**File**: `fixtures/halloy/src/appearance/theme/progress_bar.rs`

```rust
use iced::widget::progress_bar::{Catalog, Style, StyleFn};

impl Catalog for Theme {
    type Class<'a> = progress_bar::Class<'a>;
    
    fn appearance(&self) -> progress_bar::Appearance {
        progress_bar::Appearance {
            background: Background::Color(self.surface),
            bar: Background::Color(
                self.primary.with_alpha(0.7)
            ),
            border_radius: 4.0.into(),
        }
    }
    
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style::default()
    }
}
```

### Spinner Styling

```rust
// Can be implemented as a text spinner with theme colors
impl Theme {
    pub fn spinner_style(&self) -> text::Style {
        text::Style {
            color: Some(self.primary),
        }
    }
}
```

### Loading Text Styling

```rust
// Status messages use theme colors
impl Theme {
    pub fn loading_text_style(&self) -> text::Style {
        text::Style {
            color: Some(self.text_secondary),
        }
    }
}
```

### Key Design Decisions

#### ✅ **Consistent Color Scheme**
- Primary color for active progress
- Secondary/surface colors for background
- Subtle animations that respect theme

#### ✅ **Border Radius**
- Rounded corners for modern look
- Consistent across all progress indicators
- Matches app design language

#### ✅ **Alpha Transparency**
- Semi-transparent backgrounds
- Doesn't obscure content
- Works in both light and dark themes

---

## ⚡ 10. Performance Considerations

### Animation Performance

#### ✅ **Use Iced's Built-in Animation**
```rust
// Good - Iced handles animation efficiently
let indicator = animate(
    start.elapsed(),
    Duration::from_millis(1000),
    |progress| pulse(progress),
);
```

#### ❌ **Avoid Manual Animation Loops**
```rust
// Bad - manual animation loops can cause performance issues
loop {
    self.progress += 0.01;
    if self.progress > 1.0 { self.progress = 0.0; }
    iced::futures::pending!();
}
```

### State Management

#### ✅ **Lazy Loading**
```rust
// Only show loader when needed
if let Some(preview) = &self.preview {
    match &preview.state {
        PreviewState::Loading => show_loader(),
        _ => show_content(),
    }
}
```

#### ✅ **Debounce Rapid Updates**
```rust
// For progress bars that update frequently
if self.last_update.elapsed() > Duration::from_millis(50) {
    self.last_update = Instant::now();
    // Update progress
}
```

### Memory Usage

#### ✅ **Reuse Loading States**
```rust
// Don't create new loaders for each operation
// Reuse existing state
self.file_transfers.update(event);
```

#### ✅ **Clean Up Completed Loaders**
```rust
// Remove loaders when done
if let LoadingState::Loaded(_) = &self.state {
    self.state = LoadingState::Idle;
}
```

---

## 🧪 11. Testing Loaders

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_progress_bar_updates() {
        let mut transfer = FileTransfer::new("test.txt");
        
        // Initial state
        assert!(matches!(transfer.state, LoadingState::Loading));
        
        // Update progress
        transfer.update(TransferEvent::Progress(0.5));
        assert_eq!(transfer.progress, 0.5);
        
        // Complete transfer
        transfer.update(TransferEvent::Complete(TransferData::new()));
        assert!(matches!(transfer.state, LoadingState::Loaded(_)));
    }
    
    #[test]
    fn test_typing_indicator_animation() {
        let start = Instant::now();
        let indicator = animate(
            start.elapsed(),
            Duration::from_millis(1000),
            |progress| pulse(progress),
        );
        
        assert!(indicator.is_some());
    }
}
```

### Integration Tests

```rust
#[test]
fn test_file_transfer_view() {
    let transfer = FileTransfer::new("test.txt");
    let view = transfer.view();
    
    // Verify progress bar is rendered
    assert!(view.contains("progress_bar"));
    
    // Update to 50% and verify
    transfer.update(TransferEvent::Progress(0.5));
    let view = transfer.view();
    assert!(view.contains("50%"));
}
```

### Visual Regression Tests

```rust
#[test]
fn test_loader_appearance() {
    let theme = Theme::dark();
    
    // Capture screenshot of progress bar
    let progress_bar = progress_bar(0.0, 1.0, 0.75);
    let rendered = progress_bar.view(&theme);
    
    // Compare with golden image
    assert_snapshot!(rendered, theme = theme);
}
```

---

## 📚 12. Reusable Patterns Library

### Pattern 1: Determinate Progress Bar

**Use Case**: File transfers, downloads, uploads

```rust
pub struct ProgressBar {
    current: f32,      // 0.0 to 1.0
    total: f32,        // Total value (optional)
    theme: Theme,
}

impl ProgressBar {
    pub fn new(current: f32) -> Self {
        Self { current: current.clamp(0.0, 1.0), theme: Theme::Light }
    }
    
    pub fn view(&self) -> Element<Message> {
        iced::widget::progress_bar(0.0, 1.0, self.current)
            .style(self.theme.progress_bar_style())
            .into()
    }
}
```

### Pattern 2: Indeterminate Spinner

**Use Case**: Loading states, network requests

```rust
pub struct Spinner {
    size: f32,
    color: Color,
    duration: Duration,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: 20.0,
            color: Color::from_rgb(0.2, 0.5, 0.8),
            duration: Duration::from_millis(1000),
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let angle = (self.start.elapsed().as_millis() as f32 / 
                    self.duration.as_millis() as f32) * 2.0 * PI;
        
        // Draw spinner using rotation
        container(text("⠋"))
            .style(text::Style { color: Some(self.color) })
            .into()
    }
}
```

### Pattern 3: Pulse Animation for Typing

**Use Case**: Real-time feedback, background activity

```rust
pub struct PulseIndicator {
    dots: Vec<Dot>,
    animation: Animation,
}

impl PulseIndicator {
    pub fn new() -> Self {
        Self {
            dots: vec![Dot::new(0), Dot::new(1), Dot::new(2)],
            animation: Animation::new(Duration::from_millis(300)),
        }
    }
    
    pub fn update(&mut self, delta: Duration) {
        self.animation.update(delta);
        
        let progress = self.animation.progress();
        for (i, dot) in self.dots.iter_mut().enumerate() {
            dot.update(progress, i);
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        row(self.dots.iter().map(|dot| dot.view())).into()
    }
}
```

### Pattern 4: Loading State Machine

**Use Case**: Any async operation

```rust
pub enum LoadingState<T> {
    Idle,
    Loading,
    Loaded(T),
    Failed(String),
}

impl<T> LoadingState<T> {
    pub fn is_loading(&self) -> bool {
        matches!(self, LoadingState::Loading)
    }
    
    pub fn is_loaded(&self) -> bool {
        matches!(self, LoadingState::Loaded(_))
    }
    
    pub fn unwrap(&self) -> &T {
        match self {
            LoadingState::Loaded(data) => data,
            _ => panic!("Called unwrap on non-loaded state"),
        }
    }
}
```

---

## 🚫 13. Common Mistakes & Solutions

### Mistake 1: Overusing Modal Loaders

**❌ Bad**: Blocking the entire UI with a modal loader
```rust
// Bad - modal blocks everything
modal::modal(content, loading_spinner, || Message::None, 0.7)
```

**✅ Good**: Use inline loaders that preserve context
```rust
// Good - inline loader
container(
    column![
        content,
        if loading { spinner() } else { space() },
    ]
)
```

**Halloy Lesson**: Use `PreviewState::Loading` with a spinner in the content area, not a modal.

---

### Mistake 2: No Visual Feedback for Fast Operations

**❌ Bad**: Only showing loader for operations >2 seconds
```rust
if duration > 2000 {
    show_loader();
}
```

**✅ Good**: Show immediate feedback even for fast operations
```rust
// Always show typing indicator when user is typing
// Always show spinner for preview loading
```

**Halloy Lesson**: Halloy shows typing indicators immediately, even though they complete in <1 second.

---

### Mistake 3: Not Handling Errors

**❌ Bad**: Only showing loading state, no error handling
```rust
match state {
    Loading => show_loader(),
    Loaded(data) => show_content(data),
    // No Failed case!
}
```

**✅ Good**: Always handle all states
```rust
match state {
    Loading => show_loader(),
    Loaded(data) => show_content(data),
    Failed(error) => show_error(error),
}
```

**Halloy Lesson**: Every loader has a corresponding error state.

---

### Mistake 4: Blocking UI During Loading

**❌ Bad**: Freezing UI while loading
```rust
// Bad - synchronous loading
let data = load_sync();
update_ui(data);
```

**✅ Good**: Use async operations with Task
```rust
// Good - async loading
Task::perform(
    async { load_data().await },
    Message::DataLoaded
)
```

**Halloy Lesson**: All file transfers and previews use async Task-based loading.

---

### Mistake 5: Inconsistent Loader Styles

**❌ Bad**: Different colors, sizes, and animations across the app
```rust
// Different progress bars have different colors
progress_bar1.style(red_theme)
progress_bar2.style(blue_theme)
```

**✅ Good**: Use theme-based styling
```rust
// All progress bars use the same theme
progress_bar.style(theme.progress_bar_style())
```

**Halloy Lesson**: All loaders use the theme's appearance settings.

---

### Mistake 6: Not Cleaning Up Loaders

**❌ Bad**: Keeping loaders in memory after completion
```rust
// Bad - state never resets
self.loading = true; // Never set to false
```

**✅ Good**: Reset state when done
```rust
// Good - proper state management
self.state = match result {
    Ok(data) => LoadingState::Loaded(data),
    Err(e) => LoadingState::Failed(e),
};
```

**Halloy Lesson**: Every loader has an Idle state when not active.

---

### Mistake 7: Using Loaders for Everything

**❌ Bad**: Showing loader for every button click
```rust
button("Save").on_press(Message::SaveWithLoader)
```

**✅ Good**: Only show loaders for operations that take >200ms
```rust
button("Save").on_press(Message::Save)

// In update
if operation_takes_too_long {
    show_loader();
}
```

**Halloy Lesson**: Halloy only shows loaders for network operations and file transfers.

---

## 🚫 14. When NOT to Use Loaders

### ❌ Don't Use Loaders For:

1. **Immediate Operations**
   - Local state changes (<100ms)
   - Button clicks that complete instantly
   - UI transitions

2. **Success States**
   - Use checkmarks ✓ instead of loaders
   - Use "Saved" text instead of spinner

3. **Error States**
   - Use error messages ⚠️ instead of loaders
   - Show error text with retry button

4. **Background Tasks**
   - Use notifications instead of inline loaders
   - Let users continue using the app

5. **Very Short Operations**
   - <200ms operations don't need feedback
   - Users won't notice the loader anyway

### ✅ Halloy's Approach:

```rust
// Good - immediate feedback (no loader needed)
button("Save").on_press(Message::Save) // Completes instantly

// Good - loader for network operations
Task::perform(
    async { save_to_server().await },
    |result| match result {
        Ok(_) => Message::SaveSuccess,
        Err(e) => Message::SaveFailed(e),
    }
)
```

---

## 🎯 15. Advanced Patterns

### Pattern 1: Multi-Stage Loading

**Use Case**: Operations with multiple phases

```rust
pub enum MultiStageState {
    Stage1(LoadingState<Phase1Data>),
    Stage2(LoadingState<Phase2Data>),
    Stage3(LoadingState<Phase3Data>),
    Complete(FinalData),
}

impl MultiStageState {
    pub fn current_stage(&self) -> usize {
        match self {
            MultiStageState::Stage1(_) => 1,
            MultiStageState::Stage2(_) => 2,
            MultiStageState::Stage3(_) => 3,
            MultiStageState::Complete(_) => 4,
        }
    }
}
```

### Pattern 2: Progress Queue

**Use Case**: Multiple parallel operations

```rust
pub struct ProgressQueue {
    operations: VecDeque<LoadingState<Operation>>,
    max_concurrent: usize,
}

impl ProgressQueue {
    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push_back(LoadingState::Loading(operation));
    }
    
    pub fn complete_current(&mut self, result: Result<OperationResult, String>) {
        if let Some(state) = self.operations.pop_front() {
            match result {
                Ok(data) => self.operations.push_back(LoadingState::Loaded(data)),
                Err(e) => self.operations.push_back(LoadingState::Failed(e)),
            }
        }
    }
}
```

### Pattern 3: Estimated Time Remaining

**Use Case**: Operations where duration can be estimated

```rust
pub struct EstimatedProgress {
    current: f32,
    total: f32,
    start_time: Instant,
    estimated_duration: Option<Duration>,
}

impl EstimatedProgress {
    pub fn estimated_time_remaining(&self) -> Option<Duration> {
        if let Some(estimated) = self.estimated_duration {
            let elapsed = self.start_time.elapsed();
            let progress = self.current / self.total;
            let estimated_elapsed = estimated.mul_f32(progress);
            
            if elapsed < estimated_elapsed {
                Some(estimated_elapsed - elapsed)
            } else {
                Some(Duration::from_secs(0))
            }
        } else {
            None
        }
    }
}
```

### Pattern 4: Skeleton Screens

**Use Case**: Preview content before it loads

```rust
// Instead of showing a blank area, show placeholder shapes
pub fn skeleton_view() -> Element<Message> {
    column![
        container(
            row![
                circle(40),
                column![
                    rectangle(200, 10),
                    rectangle(150, 10),
                ]
                .spacing(8),
            ]
            .spacing(12),
        )
        .style(theme::container::surface(theme)),
        // ... more skeleton elements
    ]
    .into()
}
```

---

## 📊 Summary: Halloy's Loading Patterns

### What Halloy Does Well:

✅ **Progressive Disclosure** - Show loaders only when needed
✅ **Multiple Indicator Types** - Progress bars, spinners, pulses
✅ **State Machines** - Clear 3-4 state patterns
✅ **Theme Integration** - Loaders respect app theme
✅ **Non-Blocking** - UI remains interactive
✅ **Error Handling** - Every loader has an error state
✅ **Performance Optimized** - Efficient animations

### Key Takeaways:

1. **Use the right loader for the job**
   - Progress bar for known duration
   - Spinner for unknown duration
   - Pulse for ongoing activity
   - Status text for simple feedback

2. **Keep UI responsive**
   - Use async Task-based loading
   - Don't block with modals unless necessary
   - Show loaders immediately for >200ms operations

3. **Handle all states**
   - Loading, Loaded, Failed, Idle
   - Don't forget error states
   - Clean up when done

4. **Respect user attention**
   - Subtle animations that don't distract
   - Don't overuse loaders
   - Provide immediate feedback

5. **Theme your loaders**
   - Use consistent colors
   - Match your app's design language
   - Consider light/dark mode

---

## 🎓 Final Recommendations

### For Your IRC Client or Similar App:

1. **Start Simple**: Use progress bars for file transfers, spinners for previews
2. **Be Consistent**: Use the same loader style throughout your app
3. **Handle Errors**: Every loader should have an error state
4. **Test Performance**: Ensure loaders don't cause jank
5. **Document**: Add comments explaining when each loader appears

### Example Integration:

```rust
// File transfer with progress bar
let transfer = FileTransfer::new("document.pdf");

// Preview loading with spinner
let preview = Preview::new(url);

// Configuration reload with status text
let config_reload = ConfigReload::new();

// Typing indicator with pulse animation
let typing = TypingIndicator::new();
```

### Code Generation Prompt:

```
"Create a Rust Iced application with loading indicators that:
1. Show progress bars for file uploads/downloads
2. Show spinners for preview loading
3. Show pulse animations for typing indicators
4. Handle all error states gracefully
5. Respect dark/light theme
6. Are non-blocking and preserve UI interactivity

Use these patterns as inspiration:
[PASTE Progress Bar Pattern]
[PASTE Spinner Pattern]
[PASTE Pulse Animation Pattern]

Provide complete working code with proper imports and theme integration."
```

---

## 📚 Additional Resources

### Iced Documentation:
- [Iced Widgets - Progress Bar](https://docs.rs/iced/latest/iced/widget/progress_bar/)
- [Iced Widgets - Animation](https://docs.rs/iced/latest/iced/widget/struct.Animation.html)
- [Iced Examples - Loading](https://github.com/iced-rs/iced/tree/master/examples)

### Halloy References:
- `src/buffer/file_transfers.rs` - Progress bars
- `src/buffer/typing.rs` - Pulse animations
- `src/screen/dashboard.rs` - Preview loading
- `src/appearance/theme/progress_bar.rs` - Theme integration

### Best Practices:
- [NN/g - Progress Indicators](https://www.nngroup.com/articles/progress-indicators/)
- [Material Design - Loading](https://m3.material.io/styles/loading)
- [Apple HIG - Progress](https://developer.apple.com/design/human-interface-guidelines/progress-indicators)

---

## 🎉 Conclusion

Halloy demonstrates **production-ready patterns** for implementing loaders and progress indicators in Rust/Iced applications. The key lessons are:

1. **Use the right pattern for the job** (don't overgeneralize)
2. **Keep UI responsive** (async + non-blocking)
3. **Handle all states** (including errors)
4. **Respect performance** (efficient animations)
5. **Be consistent** (theme and style)

These patterns are **immediately reusable** in your project and will provide a polished, professional user experience.
