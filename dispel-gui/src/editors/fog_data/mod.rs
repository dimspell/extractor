// fog_data editor module
// Editor for `ExtraInGame/fogdata.dat` — observed map-lighting fade tables
// (123 levels × 512 brightness factors, each byte a flicker-curve sample).

mod curve_canvas;
mod message;
mod state;
mod update;
mod view;

pub use curve_canvas::FogCurveCanvas;
pub use message::FogDataMessage;
pub use state::FogDataEditorState;
pub use update::{RecordingParams, handle, save_fog_data, save_to_disk};
pub use view::view;
