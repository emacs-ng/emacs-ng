#[cfg(feature = "window-system-android")]
pub use crate::bindings::android_display_info as DisplayInfo;
use crate::bindings::gui_display_get_arg;
#[cfg(have_ns)]
pub use crate::bindings::ns_display_info as DisplayInfo;
#[cfg(feature = "window-system-w32")]
pub use crate::bindings::w32_display_info as DisplayInfo;
#[cfg(have_x11)]
pub use crate::bindings::x_display_info as DisplayInfo;
#[cfg(any(feature = "window-system-pgtk", feature = "window-system-winit"))]
pub use crate::bindings::Display_Info as DisplayInfo;
use crate::display_traits::FrameParam;
use crate::frame::FrameRef;
use crate::globals::Qnil;
use crate::globals::Qunbound;
use crate::lisp::LispObject;

use crate::lisp::ExternalPtr;
use crate::terminal::TerminalRef;

pub type DisplayInfoRef = ExternalPtr<DisplayInfo>;

impl DisplayInfoRef {
    pub fn terminal(&self) -> TerminalRef {
        return TerminalRef::new(self.terminal);
    }

    #[cfg(have_window_system)]
    pub fn gui_arg(&mut self, alist: LispObject, param: impl Into<FrameParam>) -> LispObject {
        let param: FrameParam = param.into();
        let res_type = param.resource_type();
        let (attr, class) = param.x_resource();

        let value = unsafe {
            gui_display_get_arg(
                self.as_mut(),
                alist,
                param.into(),
                attr.as_ptr(),
                class.as_ptr(),
                res_type.into(),
            )
        };

        // Do some validation here
        match param {
            FrameParam::IconName => {
                if value.is_string() {
                    value
                } else {
                    Qnil
                }
            }
            FrameParam::ParentId => match value {
                Qunbound | Qnil => Qnil,
                _ => {
                    unsafe { crate::bindings::CHECK_NUMBER(value) };
                    value
                }
            },
            FrameParam::Terminal | FrameParam::Display => match value {
                Qunbound => Qnil,
                _ => value,
            },
            FrameParam::ParentFrame => {
                if value.base_eq(Qunbound)
                    || value.is_nil()
                    || !value.is_frame()
                    || !FrameRef::from(value).is_live()
                    || !FrameRef::from(value).is_current_window_system()
                {
                    Qnil
                } else {
                    value
                }
            }
            _ => value,
        }
    }

    pub fn terminal_or_display_arg(&mut self, params: LispObject) -> LispObject {
        let terminal = self.gui_arg(params, FrameParam::Terminal);
        if terminal.is_not_nil() {
            return terminal;
        }
        self.gui_arg(params, FrameParam::Display)
    }
}
