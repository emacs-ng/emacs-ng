#[cfg(feature = "window-system-android")]
pub use crate::bindings::android_display_info as DisplayInfo;
#[cfg(feature = "window-system-ns")]
pub use crate::bindings::ns_display_info as DisplayInfo;
#[cfg(feature = "window-system-w32")]
pub use crate::bindings::w32_display_info as DisplayInfo;
#[cfg(feature = "window-system-x")]
pub use crate::bindings::x_display_info as DisplayInfo;
#[cfg(any(feature = "window-system-pgtk", feature = "window-system-winit"))]
pub use crate::bindings::Display_Info as DisplayInfo;

use crate::lisp::ExternalPtr;
use crate::terminal::TerminalRef;

pub type DisplayInfoRef = ExternalPtr<DisplayInfo>;

impl DisplayInfoRef {
    pub fn terminal(&self) -> TerminalRef {
        return TerminalRef::new(self.terminal);
    }
}
