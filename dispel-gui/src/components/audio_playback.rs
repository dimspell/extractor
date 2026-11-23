use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

/// Owns the audio thread and the flags used to control it.
pub struct PlaybackHandle {
    pub player: Arc<rodio::Player>,
    stop_flag: Arc<AtomicBool>,
    pub loop_flag: Arc<AtomicBool>,
    _thread: std::thread::JoinHandle<()>,
}

impl PlaybackHandle {
    pub fn new(
        player: Arc<rodio::Player>,
        stop_flag: Arc<AtomicBool>,
        loop_flag: Arc<AtomicBool>,
        thread: std::thread::JoinHandle<()>,
    ) -> Self {
        PlaybackHandle {
            player,
            stop_flag,
            loop_flag,
            _thread: thread,
        }
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.player.stop();
    }

    pub fn set_looping(&self, v: bool) {
        self.loop_flag.store(v, Ordering::Relaxed);
    }
}

impl Drop for PlaybackHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Cloneable, `Debug`-safe carrier for a `PlaybackHandle` crossing a `Task`
/// boundary. The handle is moved out exactly once by [`PendingPlayback::take`].
#[derive(Clone)]
pub struct PendingPlayback(Arc<std::sync::Mutex<Option<PlaybackHandle>>>);

impl PendingPlayback {
    pub fn new(handle: PlaybackHandle) -> Self {
        PendingPlayback(Arc::new(std::sync::Mutex::new(Some(handle))))
    }

    pub fn take(&self) -> Option<PlaybackHandle> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
    }
}

impl std::fmt::Debug for PendingPlayback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingPlayback")
    }
}

/// Open the default output device and start the audio thread, returning a
/// handle once the thread reports its player.
///
/// This blocks briefly while the device opens and while the thread reports
/// back, so callers must run it off the UI thread (e.g. inside
/// `tokio::task::spawn_blocking`).
pub fn start_playback(
    audio_bytes: Vec<u8>,
    volume: f32,
    is_looping: bool,
) -> Result<PendingPlayback, String> {
    let (player_tx, player_rx) = mpsc::sync_channel::<Arc<rodio::Player>>(1);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let loop_flag = Arc::new(AtomicBool::new(is_looping));
    let stop_flag_thread = Arc::clone(&stop_flag);
    let loop_flag_thread = Arc::clone(&loop_flag);

    let thread = std::thread::spawn(move || {
        let mut device_sink = match rodio::stream::DeviceSinkBuilder::open_default_sink() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("audio: failed to open audio output: {e}");
                return;
            }
        };
        device_sink.log_on_drop(false);

        let player = Arc::new(rodio::Player::connect_new(device_sink.mixer()));
        player.set_volume(volume);

        if player_tx.send(Arc::clone(&player)).is_err() {
            // Receiver dropped (timed out) — abort before playing anything.
            return;
        }

        loop {
            if stop_flag_thread.load(Ordering::Relaxed) {
                break;
            }
            let cursor = Cursor::new(audio_bytes.clone());
            match rodio::Decoder::new(cursor) {
                Ok(source) => player.append(source),
                Err(e) => {
                    eprintln!("audio: decoder error: {e}");
                    break;
                }
            }
            player.sleep_until_end();
            if !loop_flag_thread.load(Ordering::Relaxed) || stop_flag_thread.load(Ordering::Relaxed)
            {
                break;
            }
        }
        // `device_sink` stays alive for the whole loop; dropping it stops audio.
    });

    let player = player_rx
        .recv_timeout(Duration::from_millis(500))
        .map_err(|_| "timed out waiting for audio thread to start".to_string())?;
    Ok(PendingPlayback::new(PlaybackHandle::new(
        player, stop_flag, loop_flag, thread,
    )))
}
