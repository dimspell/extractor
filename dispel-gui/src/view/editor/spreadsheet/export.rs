//! Async CSV export task. Editors call [`export_csv_task`] from their own
//! message handler so the returned `Task` can be flattened into the editor's
//! update function.

use super::message::SpreadsheetMessage;
use super::state::SpreadsheetState;
use crate::components::editable::EditableRecord;
use crate::message::{Message, SystemMessage};

pub fn export_csv_task<R: EditableRecord>(
    spreadsheet: &SpreadsheetState,
    catalog: &[R],
    default_file_name: &str,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> iced::Task<Message> {
    use iced::Task;

    let bytes = match spreadsheet.to_csv_bytes(catalog) {
        Ok(b) => b,
        Err(e) => {
            return Task::done(Message::System(SystemMessage::ShowError(format!(
                "CSV export failed: {e}"
            ))));
        }
    };
    let default_file_name = default_file_name.to_string();

    Task::perform(
        async move {
            let Some(handle) = rfd::AsyncFileDialog::new()
                .set_file_name(&default_file_name)
                .add_filter("CSV", &["csv"])
                .save_file()
                .await
            else {
                return Err("cancelled".to_string());
            };
            let path = handle.path().to_path_buf();
            tokio::fs::write(&path, &bytes)
                .await
                .map(|_| path)
                .map_err(|e| e.to_string())
        },
        move |result| spreadsheet_msg(SpreadsheetMessage::CsvExported(result)),
    )
}
