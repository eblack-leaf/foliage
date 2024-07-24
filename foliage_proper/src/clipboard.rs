use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct ClipboardHandle {
    write_message: Option<String>,
}
impl ClipboardHandle {
    pub(crate) fn new() -> Self {
        Self {
            write_message: None,
        }
    }
    pub fn write<S: AsRef<str>>(&mut self, s: S) {
        self.write_message.replace(s.as_ref().to_string());
    }
    pub fn write_message(&mut self) -> Option<String> {
        self.write_message.take()
    }
}
pub fn clipboard_write(message: String) {
    #[cfg(not(target_family = "wasm"))]
    {
        // copypasta
    }
    #[cfg(target_family = "wasm")]
    {}
}
