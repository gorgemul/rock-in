use notify_rust::{Notification, error::Result, set_application};

// NOTE: this name should be changed in lockstep with cargo.toml, should have a way to make
// cargo.toml and main.rs to read from the same source, which could be done by using build.rs
// but is kinda ugly, will do that in the future(maybe).
#[cfg(feature = "macos-un")]
const APP_IDENTIFIER: &str = "com.gorgemul.rock-in";
#[cfg(not(feature = "macos-un"))]
const APP_IDENTIFIER: &str = "com.apple.Terminal";

#[cfg(all(feature = "macos-un", target_os = "macos"))]
const EXECUTABLE_PATH: &str = "Contents/MacOS";
#[cfg(all(feature = "macos-un", target_os = "linux"))]
const EXECUTABLE_PATH: &str = todo!("Not support linux yet");
#[cfg(all(feature = "macos-un", target_os = "windows"))]
const EXECUTABLE_PATH: &str = todo!("Not support windows yet");

#[derive(Debug)]
pub struct Notifier {}

impl Notifier {
    pub fn new() -> Result<Self> {
        set_application(APP_IDENTIFIER)?;
        #[cfg(feature = "macos-un")]
        {
            notify_rust::request_auth_blocking()?;
        }
        Ok(Self {})
    }
    pub fn notify(&self, summary: &str, body: &str) -> Result<()> {
        Notification::new().summary(summary).body(body).show()?;
        Ok(())
    }
}
