use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";
const CLEAR_LINE: &str = "\r\x1b[K";

pub struct Spinner {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Spinner {
    pub fn new(message: impl Into<String> + Send + 'static) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = Arc::clone(&stop);
        let message = message.into();

        let handle = thread::spawn(move || {
            let mut i = 0usize;
            while !stop2.load(Ordering::Relaxed) {
                print!(
                    "\r{}{}{} {}",
                    CYAN,
                    FRAMES[i % FRAMES.len()],
                    RESET,
                    message
                );
                std::io::stdout().flush().ok();
                i += 1;
                thread::sleep(Duration::from_millis(100));
            }
        });

        Self {
            stop,
            handle: Some(handle),
        }
    }

    pub fn finish_and_clear(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            h.join().ok();
        }
        print!("{}", CLEAR_LINE);
        std::io::stdout().flush().ok();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if !self.stop.load(Ordering::Relaxed) {
            self.finish_and_clear();
        }
    }
}
