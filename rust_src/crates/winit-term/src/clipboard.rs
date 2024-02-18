use crate::api::event_loop::EventLoop;
pub use arboard::Clipboard;

pub trait ClipboardExt {
    fn build(event_loop: &EventLoop<i32>) -> Self;
    fn write(&mut self, content: String);
    fn read(&mut self) -> String;
}

impl ClipboardExt for Clipboard {
    fn build(_: &EventLoop<i32>) -> Self {
        Clipboard::new().unwrap()
    }
    fn write(&mut self, content: String) {
        self.set_text(content).unwrap();
    }

    fn read(&mut self) -> String {
        match &self.get_text() {
            Ok(s) => s.to_string(),
            Err(_) => String::from(""),
        }
    }
}
