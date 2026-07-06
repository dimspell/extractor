//! Modal dialog standalone example — demonstrates the gui-widgets `modal()` overlay.
//!
//! Shows three modal variants: an info dialog, a confirm dialog, and a settings form.
//! All modals close on Escape or click-outside (via the `on_blur` callback).
//!
//! Run with:
//! ```sh
//! cargo run --example modal_demo -p gui-widgets
//! ```

use gui_widgets::components::modal::modal;
use iced::keyboard::{self, key};
use iced::widget::{button, checkbox, column, container, row, rule, text, text_input};
use iced::{color, Background, Element, Fill, Subscription, Theme};

// ── Entry point ────────────────────────────────────────────────────────────────

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .theme(|_: &App| {
            Theme::custom(
                "Medieval",
                iced::theme::palette::Seed {
                    background: color!(0x2a2a2a),
                    text: color!(0xeae0c8),
                    primary: color!(0x8b5a2b),
                    success: color!(0x2d5a27),
                    danger: color!(0x800000),
                    warning: color!(0x8b8b00),
                },
            )
        })
        .subscription(App::subscription)
        .title("Modal Demo — gui-widgets")
        .window_size((800.0, 600.0))
        .run()
}

// ── Message ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Message {
    /// Open a specific modal type.
    OpenModal(ModalType),
    /// Dismiss whichever modal is currently shown.
    CloseModal,
    /// Confirm action inside a confirm dialog.
    ConfirmAction,
    /// Cancel a confirm dialog without acting.
    CancelAction,
    /// Text input inside the settings modal.
    NameChanged(String),
    /// Checkbox toggle inside the settings modal.
    NotificationsToggled(bool),
}

// ── Modal variants ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum ModalType {
    /// Simple info/help dialog.
    Info,
    /// Confirm/cancel dialog (e.g. "are you sure?").
    Confirm,
    /// Form with text input and checkbox.
    Settings,
}

// ── App state ──────────────────────────────────────────────────────────────────

struct App {
    /// Which modal is currently shown, if any.
    active_modal: Option<ModalType>,
    /// Simulated confirmation flag — set to true when ConfirmAction fires.
    confirmed: bool,
    /// Settings fields.
    name: String,
    notifications_enabled: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            active_modal: None,
            confirmed: false,
            name: String::new(),
            notifications_enabled: true,
        }
    }
}

impl App {
    fn new() -> Self {
        Self::default()
    }

    /// Listen for Escape globally to close any open modal.
    fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| {
            if let keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            } = event
            {
                Some(Message::CloseModal)
            } else {
                None
            }
        })
    }

    // ── Update ─────────────────────────────────────────────────────────────

    fn update(&mut self, message: Message) {
        match message {
            Message::OpenModal(modal_type) => {
                self.active_modal = Some(modal_type);
            }
            Message::CloseModal => {
                self.active_modal = None;
            }
            Message::ConfirmAction => {
                self.confirmed = true;
                self.active_modal = None;
            }
            Message::CancelAction => {
                self.active_modal = None;
            }
            Message::NameChanged(v) => self.name = v,
            Message::NotificationsToggled(v) => self.notifications_enabled = v,
        }
    }

    // ── View ───────────────────────────────────────────────────────────────

    fn view(&self) -> Element<'_, Message> {
        let muted = |_t: &Theme| text::Style {
            color: Some(color!(0x888888)),
        };
        let green = |_t: &Theme| text::Style {
            color: Some(color!(0x66bb6a)),
        };
        // ── Base content ───────────────────────────────────────────────
        let base = container(
            column![
                // Header
                text("Modal Dialog Demo").size(28),
                text("gui-widgets modal() overlay widget")
                    .size(14)
                    .style(muted),
                rule::horizontal(1),
                // Description
                column![
                    text("This example demonstrates three modal variants built with the").size(13),
                    text("gui-widgets modal widget. Click opens an overlay; Escape or").size(13),
                    text("click-outside closes it.").size(13),
                ]
                .spacing(2),
                rule::horizontal(1),
                // Open buttons
                row![
                    button(
                        column![
                            text("Info").size(16),
                            text("Simple dialog with a message").size(11).style(muted),
                        ]
                        .spacing(4)
                        .padding(8),
                    )
                    .on_press(Message::OpenModal(ModalType::Info))
                    .width(180),
                    button(
                        column![
                            text("Confirm").size(16),
                            text("Yes/no with simulated action").size(11).style(muted),
                        ]
                        .spacing(4)
                        .padding(8),
                    )
                    .on_press(Message::OpenModal(ModalType::Confirm))
                    .width(180),
                    button(
                        column![
                            text("Settings").size(16),
                            text("Form with inputs and toggles").size(11).style(muted),
                        ]
                        .spacing(4)
                        .padding(8),
                    )
                    .on_press(Message::OpenModal(ModalType::Settings))
                    .width(180),
                ]
                .spacing(12),
                // Confirmation result
                if self.confirmed {
                    text("✓ Action confirmed!").size(13).style(green)
                } else {
                    text("No action taken yet.").size(13).style(muted)
                },
            ]
            .spacing(16)
            .padding(32),
        )
        .width(Fill)
        .height(Fill);

        // ── Wrap with modal when one is active ─────────────────────────
        match &self.active_modal {
            Some(modal_type) => {
                // The on_blur callback fires when the user presses Escape or
                // clicks outside the modal overlay.
                let modal_content: Element<'_, Message> = match modal_type {
                    ModalType::Info => Self::view_info_modal(),
                    ModalType::Confirm => Self::view_confirm_modal(),
                    ModalType::Settings => self.view_settings_modal(),
                };

                modal(base, modal_content, || Message::CloseModal, 0.5)
            }
            None => base.into(),
        }
    }

    // ── Modal content builders ────────────────────────────────────────────

    fn modal_surface() -> impl Fn(&Theme) -> container::Style {
        |_theme: &Theme| container::Style {
            background: Some(Background::Color(color!(0x2d2a24))),
            ..container::Style::default()
        }
    }

    fn view_info_modal() -> Element<'static, Message> {
        let gold = |_t: &Theme| text::Style {
            color: Some(color!(0xe0c060)),
        };

        container(
            column![
                text("ℹ Info — About the Modal Widget").size(20),
                rule::horizontal(1),
                column![
                    text("The modal() function from gui-widgets lets you overlay any").size(13),
                    text("Element on top of the base UI with a dimmed backdrop.").size(13),
                    text("").size(8),
                    text("Features:").size(14).style(gold),
                    text("  • Escape key or click-outside to dismiss").size(13),
                    text("  • Configurable backdrop opacity").size(13),
                    text("  • All input blocked from the base layer").size(13),
                    text("  • Works with any Element, Theme, and Renderer").size(13),
                    text("  • Built as an Iced overlay widget").size(13),
                ]
                .spacing(2),
                rule::horizontal(1),
                button(text("Close").size(14))
                    .on_press(Message::CloseModal)
                    .width(120),
            ]
            .spacing(12)
            .padding(24),
        )
        .width(400)
        .style(Self::modal_surface())
        .into()
    }

    fn view_confirm_modal() -> Element<'static, Message> {
        let red = |_t: &Theme| text::Style {
            color: Some(color!(0xcc5555)),
        };

        container(
            column![
                text("⚠ Confirm Action").size(20),
                rule::horizontal(1),
                text("Delete this item?").size(14),
                text("This action cannot be undone.").size(12).style(red),
                rule::horizontal(1),
                row![
                    button(text("Cancel").size(14))
                        .on_press(Message::CancelAction)
                        .width(100),
                    button(text("Delete").size(14))
                        .on_press(Message::ConfirmAction)
                        .width(100),
                ]
                .spacing(12),
            ]
            .spacing(12)
            .padding(24),
        )
        .width(360)
        .style(Self::modal_surface())
        .into()
    }

    fn view_settings_modal(&self) -> Element<'_, Message> {
        container(
            column![
                text("⚙ Settings").size(20),
                rule::horizontal(1),
                // Name field
                column![
                    text("Display name").size(13),
                    text_input("Enter your name...", &self.name)
                        .on_input(Message::NameChanged)
                        .padding(8)
                        .size(14),
                ]
                .spacing(4),
                // Notifications toggle
                row![
                    checkbox(self.notifications_enabled).on_toggle(Message::NotificationsToggled),
                    text("Enable notifications").size(14),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                rule::horizontal(1),
                row![button(text("Close").size(14))
                    .on_press(Message::CloseModal)
                    .width(100),]
                .spacing(12),
            ]
            .spacing(12)
            .padding(24),
        )
        .width(400)
        .style(Self::modal_surface())
        .into()
    }
}
