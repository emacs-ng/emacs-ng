//! libvterm facilities

use remacs_macros::lisp_fn;

use libc::{c_char, c_uchar, c_void, size_t, strlen};

use std::{cmp, mem};

use crate::remacs_sys::color_to_rgb_string;

use crate::{
    lisp::ExternalPtr,
    lisp::LispObject,
    obarray::intern,
    remacs_sys::{
        call0, call1, call2, call3, code_convert_string_norecord, del_range, looking_at_1,
        make_string, pvec_type, send_process, vterminal, EmacsInt, Fforward_char,
        Fget_buffer_window, Finsert, Flength, Fline_end_position, Fpoint, Fput_text_property,
        Fselected_window, Fset, Fset_window_point, Lisp_Type, Qbold, Qcursor_type, Qface, Qitalic,
        Qnil, Qnormal, Qt, Qterminal_live_p, Qutf_8, STRING_BYTES,
    },

    remacs_sys::{
        fetch_cell, is_eol, parser_callbacks, row_to_linenr, search_command, set_point,
        term_redraw_cursor, utf8_to_codepoint, vterm_output_read, vterm_screen_callbacks,
        vterm_screen_set_callbacks, VtermScrollbackLine,
    },

    // libvterm
    remacs_sys::{
        vterm_color_is_equal, vterm_input_write, vterm_keyboard_end_paste, vterm_keyboard_key,
        vterm_keyboard_start_paste, vterm_keyboard_unichar, vterm_new, vterm_obtain_screen,
        vterm_obtain_state, vterm_output_get_buffer_current, vterm_screen_enable_altscreen,
        vterm_screen_flush_damage, vterm_screen_get_cell, vterm_screen_is_eol, vterm_screen_reset,
        vterm_screen_set_damage_merge, vterm_set_size, vterm_set_utf8, vterm_state_get_cursorpos,
        vterm_state_get_default_colors, vterm_state_set_unrecognised_fallbacks, VTermColor,
        VTermDamageSize, VTermKey, VTermModifier, VTermPos, VTermProp, VTermRect, VTermScreenCell,
        VTermState, VTermValue,
    },

    threads::ThreadState,
};

pub type LispVterminalRef = ExternalPtr<vterminal>;
pub type VTermScreenCellRef = ExternalPtr<VTermScreenCell>;

impl LispObject {
    pub fn is_vterminal(self) -> bool {
        self.as_vectorlike()
            .map_or(false, |v| v.is_pseudovector(pvec_type::PVEC_VTERMINAL))
    }

    pub fn as_vterminal(self) -> Option<LispVterminalRef> {
        self.as_vectorlike().and_then(|v| v.as_vterminal())
    }

    pub fn as_vterminal_or_error(self) -> LispVterminalRef {
        self.as_vterminal()
            .unwrap_or_else(|| wrong_type!(Qterminal_live_p, self))
    }
}

impl From<LispObject> for LispVterminalRef {
    fn from(o: LispObject) -> Self {
        o.as_vterminal_or_error()
    }
}

impl From<LispVterminalRef> for LispObject {
    fn from(v: LispVterminalRef) -> Self {
        Self::tag_ptr(v, Lisp_Type::Lisp_Vectorlike)
    }
}

impl From<LispObject> for Option<LispVterminalRef> {
    fn from(o: LispObject) -> Self {
        o.as_vterminal()
    }
}

impl LispVterminalRef {
    pub unsafe fn set_callbacks(mut self) {
        vterm_screen_set_callbacks(
            (*self).vts,
            &vterm_screen_callbacks,
            self.as_mut() as *mut libc::c_void,
        );

        let state: *mut VTermState = vterm_obtain_state((*self).vt);
        vterm_state_set_unrecognised_fallbacks(
            state,
            &parser_callbacks,
            self.as_mut() as *mut libc::c_void,
        );
    }

    pub fn set_size(self, rows: i32, cols: i32) {
        unsafe {
            vterm_set_size((*self).vt, rows, cols);
        }
    }

    /// Return cursor position as VTermPos
    pub unsafe fn get_cursorpos(self) -> VTermPos {
        let state: *mut VTermState = vterm_obtain_state((*self).vt);
        let mut pos: VTermPos = std::mem::zeroed();
        vterm_state_get_cursorpos(state, &mut pos);
        pos
    }

    /// Get cell at given offset
    pub unsafe fn fetch_cell(mut self, row: i32, col: i32) -> VTermScreenCell {
        let mut cell: VTermScreenCell = std::mem::zeroed();
        fetch_cell(self.as_mut(), row, col, &mut cell);
        cell
    }

    // TODO: this function shouldn't exist on vterminal type
    //
    pub unsafe fn is_eol(mut self, end_col: i32, row: i32, col: i32) -> bool {
        is_eol(self.as_mut(), end_col, row, col)
        // myis_eol(self, end_col, row, col)
    }

    // TODO: cell and vterm don't need mut
    /// Make lisp string from c string and add properties
    pub unsafe fn make_propertized_string(
        mut self,
        buffer: *mut c_char,
        len: i32,
        cell: *mut VTermScreenCell,
    ) -> LispObject {
        let mut text = make_string(buffer, len as isize);

        let start = LispObject::from(0);
        let end = Flength(text);
        let properties = list!(
            LispObject::from(intern(":foreground")),
            color_to_rgb_string(self.as_mut(), &mut (*cell).fg),
            LispObject::from(intern(":background")),
            color_to_rgb_string(self.as_mut(), &mut (*cell).bg),
            LispObject::from(intern(":weight")),
            if (*cell).attrs.bold() > 0 {
                Qbold
            } else {
                Qnormal
            },
            LispObject::from(intern(":underline")),
            if (*cell).attrs.underline() > 0 {
                Qt
            } else {
                Qnil
            },
            LispObject::from(intern(":slant")),
            if (*cell).attrs.italic() > 0 {
                Qitalic
            } else {
                Qnormal
            },
            LispObject::from(intern(":inverse-video")),
            if (*cell).attrs.reverse() > 0 {
                Qt
            } else {
                Qnil
            },
            LispObject::from(intern(":strike-through")),
            if (*cell).attrs.strike() > 0 { Qt } else { Qnil }
        );

        Fput_text_property(start, end, Qface, properties, text);
        text
    }

    // TODO: remove end_col and get value inside of this method
    /// Refresh lines from START_ROW to END_ROW.
    pub unsafe fn refresh_lines(mut self, start_row: i32, end_row: i32, end_col: i32) {
        let mut size = ((end_row - start_row + 1) * end_col) * 4;
        let mut v: Vec<c_char> = Vec::with_capacity(size as usize);

        let mut lastcell: VTermScreenCell = self.fetch_cell(start_row, 0);

        let mut length = 0;

        let mut i = start_row;
        while i < end_row {
            let mut j = 0;

            while j < end_col {
                let cell = self.fetch_cell(i, j);

                // if cell attributes are not equal to last cell, insert contents of vector v
                if !cell.compare(lastcell) {
                    if length > 0 {
                        let mut t =
                            self.make_propertized_string(v.as_mut_ptr(), length, &mut lastcell);
                        Finsert(1, &mut t);
                    }

                    size -= length;
                    // TODO: don't reset vector size
                    v = Vec::with_capacity(size as usize);

                    length = 0;
                }

                lastcell = cell;
                if cell.chars[0] == 0 {
                    if self.is_eol(end_col, i, j) {
                        /* This cell is EOL if this and every cell to the right is black */
                        break;
                    }

                    v.push(' ' as c_char);
                    length += 1;
                } else {
                    let size = vterminal_push_cell(&mut v, cell);
                    length += size;
                }

                // TODO: this will only be changed for else from last conditional -> put it there
                if cell.width > 1 {
                    let w = cell.width - 1;
                    j = j + w as i32;
                }
                j += 1;
            }

            v.push('\n' as c_char);
            length += 1;
            i += 1;
        }

        if length > 0 {
            let mut t = self.make_propertized_string(v.as_mut_ptr(), length, &mut lastcell);
            Finsert(1, &mut t);
        }
    }

    /// Refresh the screen (visible part of the buffer when the terminal is focused)
    /// of a invalidated terminal
    pub unsafe fn refresh_screen(mut self) {
        // Term height may have decreased before `invalid_end` reflects it.
        (*self).invalid_end = cmp::min((*self).invalid_end, (*self).height);

        if (*self).invalid_end >= (*self).invalid_start {
            let startrow = -((*self).height - (*self).invalid_start - (*self).linenum_added as i32);
            // startrow is negative,so we backward  -startrow lines from end of buffer
            // then delete lines there.
            // vterminal_goto_line(startrow as EmacsInt);
            call1(
                LispObject::from(intern("vterm--goto-line")),
                LispObject::from(startrow),
            );
            // vterminal_delete_lines(
            //     startrow as EmacsInt,
            //     LispObject::from((*self).invalid_end - (*self).invalid_start),
            // );
            call3(
                LispObject::from(intern("vterm--delete-lines")),
                LispObject::from(startrow as EmacsInt),
                LispObject::from(
                    (*self).invalid_end as EmacsInt - (*self).invalid_start as EmacsInt,
                ),
                LispObject::from(Qt),
            );

            self.refresh_lines((*self).invalid_start, (*self).invalid_end, (*self).width);

            // term->linenum_added is lines added by window height increased
            (*self).linenum += (*self).linenum_added;
            (*self).linenum_added = 0;
        }
        (*self).invalid_start = std::i32::MAX;
        (*self).invalid_end = -1;
    }

    /// Refresh cursor, scrollback and screen.
    /// Also adjust the top line.
    pub unsafe fn redraw(mut self) {
        term_redraw_cursor(self.as_mut());

        if self.is_invalidated {
            let oldlinenum = (*self).linenum;

            vterminal_refresh_scrollback(self);
            self.refresh_screen();
            (*self).linenum_added = (*self).linenum - oldlinenum;
            vterminal_adjust_topline(self);
            (*self).linenum_added = 0;
        }

        self.is_invalidated = false;
    }

    /// Send current contents of VTERM to the running shell process
    pub unsafe fn flush_output(self) {
        let len = vterm_output_get_buffer_current((*self).vt);
        if len > 0 {
            let mut buffer: Vec<c_char> = Vec::with_capacity(len);
            let len = vterm_output_read((*self).vt, buffer.as_mut_ptr() as *mut c_char, len);

            let lisp_string = make_string(buffer.as_mut_ptr() as *mut c_char, len as isize);

            send_process(
                (*self).process,
                buffer.as_mut_ptr() as *mut c_char,
                len as isize,
                lisp_string,
            );
        }
    }
}

pub unsafe fn vterminal_push_cell(v: &mut Vec<c_char>, cell: VTermScreenCell) -> i32 {
    let mut bytes: [c_uchar; 4] = std::mem::zeroed();
    let size = cell.to_utf8(&mut bytes);
    for n in 0..size {
        v.push(bytes[n] as c_char);
    }
    size as i32
}

impl VTermScreenCell {
    /// Convert contents of cell to utf8 and write result in TO
    pub unsafe fn to_utf8(self, to: &mut [u8]) -> usize {
        let cp = self.chars[0];

        if cp <= 0x7F {
            to[0] = cp as u8;
            1
        } else if cp >= 0x80 && cp <= 0x07FF {
            // note: setting later bytes first to avoid multiple bound checks
            to[1] = 0x80 | (cp & 0x3F) as u8;
            to[0] = 0xC0 | (cp >> 6) as u8;
            2
        } else if cp >= 0x0800 && cp <= 0xFFFF {
            to[2] = 0x80 | (cp & 0x3F) as u8;
            to[1] = 0x80 | ((cp >> 6) & 0x3F) as u8;
            to[0] = 0xE0 | (cp >> 12) as u8;
            3
        } else if cp >= 0x10000 && cp <= 0x10FFFF {
            to[3] = 0x80 | (cp & 0x3F) as u8;
            to[2] = 0x80 | ((cp >> 6) & 0x3F) as u8;
            to[1] = 0x80 | ((cp >> 12) & 0x3F) as u8;
            to[0] = 0xF0 | (cp >> 18) as u8;
            4
        } else {
            0
        }
    }

    /// Compare cell attributes
    pub unsafe fn compare(self, other: VTermScreenCell) -> bool {
        let self_attr = self.attrs;
        let other_attr = other.attrs;

        vterm_color_is_equal(&self.fg, &other.fg) > 0
            && vterm_color_is_equal(&self.bg, &other.bg) > 0
            && (self_attr.bold() == other_attr.bold())
            && (self_attr.underline() == other_attr.underline())
            && (self_attr.italic() == other_attr.italic())
            && (self_attr.reverse() == other_attr.reverse())
            && (self_attr.strike() == other_attr.strike())
    }
}

impl VTermPos {
    pub unsafe fn is_eol(self, vterm: LispVterminalRef) -> bool {
        vterm_screen_is_eol((*vterm).vts, self) > 0
    }

    /// Return offset in vterm
    pub unsafe fn offset(self, vterm: LispVterminalRef) -> i32 {
        let mut offset: size_t = 0;
        let pos_col = self.col;

        let mut col: i32 = 0;
        while col < pos_col {
            let cell = vterm.fetch_cell(self.row, col);

            if cell.chars[0] > 0 {
                if cell.width > 0 {
                    offset += cell.width as size_t - 1;
                }
            } else if vterm.is_eol((*vterm).width, self.row, col) {
                offset += cell.width as size_t;
            }
            col += cell.width as i32;
        }
        offset as i32
    }
}

macro_rules! allocate_zeroed_pseudovector {
    ($ty: ty, $field: ident, $vectype: expr) => {
        unsafe {
            crate::remacs_sys::allocate_pseudovector(
                vecsize!($ty) as ::libc::c_int,
                pseudovecsize!($ty, $field) as ::libc::c_int,
                vecsize!($ty) as ::libc::c_int,
                $vectype,
            ) as *mut $ty
        }
    };
}

fn allocate_vterm() -> LispVterminalRef {
    let v: *mut vterminal = allocate_zeroed_pseudovector!(vterminal, vt, pvec_type::PVEC_VTERMINAL);
    LispVterminalRef::from_ptr(v as *mut c_void).unwrap()
}

unsafe fn vterminal_adjust_topline(mut term: LispVterminalRef) {
    let state: *mut VTermState = vterm_obtain_state((*term).vt);

    let pos: VTermPos = term.get_cursorpos();

    /* pos.row-term->height is negative,so we backward term->height-pos.row
     * lines from end of buffer
     */
    // vterminal_goto_line(EmacsInt::from(pos.row - (*term).height));
    call1(
        LispObject::from(intern("vterm--goto-line")),
        LispObject::from(pos.row - (*term).height),
    );

    let offset = pos.offset(term);

    Fforward_char(LispObject::from(pos.col - offset as i32));

    let following = (*term).height == 1 + pos.row + (*term).linenum_added as i32; // cursor at end?

    let window = Fget_buffer_window(term.buffer, Qt);
    let swindow = Fselected_window();

    if window.eq(swindow) {
        if following {
            // "Follow" the terminal output
            call1(LispObject::from(intern("recenter")), LispObject::from(-1));
        } else {
            call1(
                LispObject::from(intern("recenter")),
                LispObject::from(pos.row),
            );
        }
    } else {
        if !window.is_nil() {
            Fset_window_point(window, Fpoint());
        }
    }
}

/// Refresh the scrollback of an invalidated terminal.
unsafe fn vterminal_refresh_scrollback(mut term: LispVterminalRef) {
    let max_line_count = (*term).sb_current as i32 + (*term).height;
    let mut del_cnt = 0;

    if (*term).sb_pending > 0 {
        // This means that either the window height has decreased or the screen
        // became full and libvterm had to push all rows up. Convert the first
        // pending scrollback row into a string and append it just above the visible
        // section of the buffer

        del_cnt = (*term).linenum - (*term).height - (*term).sb_size as i32 + (*term).sb_pending
            - (*term).sb_pending_by_height_decr;

        if del_cnt > 0 {
            // vterminal_delete_lines(1, LispObject::from(del_cnt as EmacsInt));
            call3(
                LispObject::from(intern("vterm--delete-lines")),
                LispObject::from(1),
                LispObject::from(del_cnt),
                LispObject::from(Qt),
            );
            (*term).linenum -= del_cnt;
        }

        (*term).linenum += (*term).sb_pending;
        del_cnt = (*term).linenum - max_line_count; /* extra lines at the bottom */
        /* buf_index is negative,so we move to end of buffer,then backward
        -buf_index lines. goto lines backward is effectively when
        vterm-max-scrollback is a large number.
         */

        let buf_index = -((*term).height + del_cnt);
        // vterminal_goto_line(buf_index as EmacsInt);
        call1(
            LispObject::from(intern("vterm--goto-line")),
            LispObject::from(buf_index),
        );

        term.refresh_lines(-(*term).sb_pending, 0, (*term).width);
        (*term).sb_pending = 0;
    }

    // Remove extra lines at the bottom

    del_cnt = (*term).linenum - max_line_count;
    if del_cnt > 0 {
        (*term).linenum -= del_cnt;
        /* -del_cnt is negative,so we delete_lines from end of buffer.
          this line means: delete del_cnt count of lines at end of buffer.
        */
        // vterminal_delete_lines(-del_cnt as EmacsInt, LispObject::from(del_cnt as EmacsInt));
        call3(
            LispObject::from(intern("vterm--delete-lines")),
            LispObject::from(-del_cnt),
            LispObject::from(del_cnt),
            LispObject::from(Qt),
        );
    }

    (*term).sb_pending_by_height_decr = 0;
    (*term).height_resize = 0;
}

#[lisp_fn(name = "vterminal-redraw")]
pub fn vterminal_redraw_lisp(mut vterm: LispVterminalRef) {
    unsafe {
        vterm.redraw();
    }
}

/// Start a libvterm terminal-emulator in a new buffer.
/// The maximum scrollback is set by the argument SCROLLBACK.
/// You can customize the value with `vterm-max-scrollback`.
///
/// libvterm requires several callback functions that are stored
/// in VTermScreenCallbacks.
#[lisp_fn(name = "vterm-new")]
pub fn vterminal_new_lisp(
    rows: i32,
    cols: i32,
    process: LispObject,
    scrollback: EmacsInt,
) -> LispVterminalRef {
    unsafe {
        let mut term = allocate_vterm();

        (*term).vt = vterm_new(rows, cols);
        vterm_set_utf8((*term).vt, 1);
        (*term).vts = vterm_obtain_screen((*term).vt);

        term.set_callbacks();
        vterm_screen_reset((*term).vts, 1);
        vterm_screen_set_damage_merge((*term).vts, VTermDamageSize::VTERM_DAMAGE_SCROLL);

        vterm_screen_enable_altscreen((*term).vts, 1);

        (*term).sb_size = scrollback as usize;
        let s = mem::size_of::<VtermScrollbackLine>() * scrollback as usize;
        (*term).sb_buffer = libc::malloc(s as libc::size_t) as *mut *mut VtermScrollbackLine;

        (*term).sb_current = 0;
        (*term).sb_pending = 0;
        (*term).sb_pending_by_height_decr = 0;
        (*term).invalid_start = 0;
        (*term).invalid_end = rows;
        (*term).width = cols;
        (*term).height = rows;
        (*term).height_resize = 0;

        let mut newline = make_string("\n".as_ptr() as *mut c_char, 1);
        let mut i = 0;
        while i < (*term).height {
            Finsert(1, &mut newline);
            i += 1;
        }

        (*term).linenum = (*term).height;
        (*term).linenum_added = 0;

        // (*term).directory = std::mem::zeroed();
        // (*term).directory_changed = false;

        (*term).buffer = LispObject::from(ThreadState::current_buffer_unchecked());
        (*term).process = process;
        term
    }
}

/// Flush output and redraw terminal VTERM.
/// If the function is called with STRING, convert it to utf8 and send it to
/// the terminal before updating.
#[lisp_fn(min = "1", name = "vterm-update")]
pub fn vterminal_update(
    vterm: LispVterminalRef,
    string: LispObject,
    shift: bool,
    meta: bool,
    ctrl: bool,
) {
    unsafe {
        if string.is_not_nil() {
            let mut utf8 = code_convert_string_norecord(string, Qutf_8, true).force_string();
            let len = STRING_BYTES(utf8.as_mut()) as usize;

            let mut v: Vec<c_uchar> = Vec::with_capacity(len as usize);

            let key = libc::memcpy(
                v.as_mut_ptr() as *mut c_void,
                utf8.data_ptr() as *mut c_void,
                len as libc::size_t,
            ) as *const c_char;

            let mut modifier = VTermModifier::VTERM_MOD_NONE;
            if shift {
                modifier |= VTermModifier::VTERM_MOD_SHIFT;
            }

            if meta {
                modifier |= VTermModifier::VTERM_MOD_ALT;
            }

            if ctrl {
                modifier |= VTermModifier::VTERM_MOD_CTRL;
            }

            let is_key = |key: *const c_char, val: *const c_char, len: usize| {
                libc::memcmp(key as *mut c_void, val as *mut c_void, len as size_t) == 0
            };

            if len > 1 && *(key.offset(0)) == 60 {
                if is_key(key, "<return>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_ENTER, modifier);
                } else if is_key(key, "<start_paste>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_start_paste((*vterm).vt);
                } else if is_key(key, "<end_paste>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_end_paste((*vterm).vt);
                } else if is_key(key, "<up>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_UP, modifier);
                } else if is_key(key, "<down>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_DOWN, modifier);
                } else if is_key(key, "<left>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_LEFT, modifier);
                } else if is_key(key, "<right>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_RIGHT, modifier);
                } else if is_key(key, "<tab>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_TAB, modifier);
                } else if is_key(key, "<backspace>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_BACKSPACE, modifier);
                } else if is_key(key, "<escape>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_ESCAPE, modifier);
                } else if is_key(key, "<insert>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_INS, modifier);
                } else if is_key(key, "<delete>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_DEL, modifier);
                } else if is_key(key, "<home>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_HOME, modifier);
                } else if is_key(key, "<end>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_END, modifier);
                } else if is_key(key, "<prior>".as_ptr() as *const c_char, len) {
                    vterm_keyboard_key((*vterm).vt, VTermKey::VTERM_KEY_PAGEUP, modifier);
                } else if *(key.offset(1)) == 102 {
                    let n = if len == 4 {
                        *(key.offset(2))
                    } else {
                        10 + *(key.offset(3))
                    } as u32;

                    vterm_keyboard_key(
                        (*vterm).vt,
                        VTermKey::VTERM_KEY_FUNCTION_0 + n - 48,
                        modifier,
                    );
                }
            } else if is_key(key, "SPC".as_ptr() as *const c_char, len) {
                vterm_keyboard_unichar((*vterm).vt, ' ' as u32, modifier);
            } else if len <= 4 {
                let mut codepoint: libc::uint32_t = std::mem::zeroed();
                if utf8_to_codepoint(key as *const c_uchar, len, &mut codepoint) {
                    vterm_keyboard_unichar((*vterm).vt, codepoint, modifier);
                }
            }
        }

        vterm.flush_output();
        if (*vterm).is_invalidated {
            call0(LispObject::from(intern("vterm--invalidate")));
        }
    }
}

/// Send INPUT to terminal VTERM.
#[lisp_fn(name = "vterm-write-input")]
pub fn vterminal_write_input(vterm: LispVterminalRef, string: LispObject) {
    unsafe {
        let mut utf8 = code_convert_string_norecord(string, Qutf_8, true).force_string();

        vterm_input_write(
            (*vterm).vt,
            utf8.sdata_ptr(),
            STRING_BYTES(utf8.as_mut()) as usize + 1,
        );
        vterm_screen_flush_damage((*vterm).vts);
    }
}

/// Change size of VTERM according to ROWS and COLS.
#[lisp_fn(name = "vterm-set-size")]
pub fn vterminal_set_size_lisp(mut vterm: LispVterminalRef, rows: i32, cols: i32) {
    unsafe {
        if cols != (*vterm).width || rows != (*vterm).height {
            (*vterm).height_resize = rows - (*vterm).height;
            if rows > (*vterm).height {
                if rows - (*vterm).height > (*vterm).sb_current as i32 {
                    (*vterm).linenum_added = rows - (*vterm).height - (*vterm).sb_current as i32;
                }
            }
            vterm.set_size(rows, cols);
            vterm_screen_flush_damage((*vterm).vts);
            vterm.redraw();
        }
    }
}

// // TODO: try to avoid goto-line and just del_range
// /// Delete COUNT lines starting from LINENUM.
// #[lisp_fn]
// pub fn vterminal_delete_lines(linenum: EmacsInt, count: LispObject) {
//     unsafe {
//         // let cur_buf = ThreadState::current_buffer_unchecked();
//         // let orig_pt = cur_buf.pt;
//         // // vterminal_goto_line(linenum);
//         // call1(
//         //     LispObject::from(intern("vterm--goto-line")),
//         //     LispObject::from(linenum),
//         // );

//         // let start = cur_buf.pt;
//         // let end = EmacsInt::from(Fline_end_position(count)) as isize;
//         // del_range(start, end);
//         // let pos = cur_buf.pt;
//         // if !looking_at_1(make_string("\n".as_ptr() as *mut c_char, 1), false).is_nil() {
//         //     del_range(pos, pos + 1);
//         // }
//         // // set_point(cmp::min(orig_pt, cur_buf.zv))
//         // set_point(orig_pt)
//         del_range(start, end);
//     };
// }

/// Return process of terminal VTERM
#[lisp_fn(name = "vterm-process")]
pub fn vterminal_process(vterm: LispVterminalRef) -> LispObject {
    (*vterm).process
}

#[lisp_fn]
pub fn vterminal_goto_line(line: EmacsInt) {
    unsafe {
        set_point(1);
        let regexp = make_string("\n".as_ptr() as *mut c_char, 1);
        search_command(regexp, Qnil, Qt, LispObject::from(line - 1), 1, 0, false)
    };
}

// vterm_screen_callbacks

#[no_mangle]
pub unsafe extern "C" fn vterminal_invalidate_terminal(
    term: *mut vterminal,
    start_row: i32,
    end_row: i32,
) {
    if start_row != -1 && end_row != -1 {
        (*term).invalid_start = cmp::min((*term).invalid_start, start_row);
        (*term).invalid_end = cmp::max((*term).invalid_end, end_row);
    }
    (*term).is_invalidated = true;
}

#[no_mangle]
pub unsafe extern "C" fn vterminal_damage(rect: VTermRect, data: *mut c_void) -> i32 {
    vterminal_invalidate_terminal(data as *mut vterminal, rect.start_row, rect.end_row);
    1
}

#[no_mangle]
pub unsafe extern "C" fn vterminal_moverect(
    dest: VTermRect,
    src: VTermRect,
    data: *mut c_void,
) -> i32 {
    vterminal_invalidate_terminal(
        data as *mut vterminal,
        cmp::min(dest.start_row, src.start_row),
        cmp::max(dest.end_row, src.end_row),
    );

    1
}

#[no_mangle]
pub unsafe extern "C" fn vterminal_movecursor(
    new: VTermPos,
    old: VTermPos,
    _visible: i32,
    data: *mut libc::c_void,
) -> i32 {
    let term: *mut vterminal = data as *mut vterminal;
    (*term).cursor.row = new.row;
    (*term).cursor.col = new.col;
    vterminal_invalidate_terminal(term, old.row, old.row + 1);
    vterminal_invalidate_terminal(term, new.row, new.row + 1);

    1
}

#[no_mangle]
pub unsafe extern "C" fn vterminal_resize(
    rows: i32,
    cols: i32,
    user_data: *mut libc::c_void,
) -> i32 {
    let term: *mut vterminal = user_data as *mut vterminal;
    (*term).invalid_start = 0;
    (*term).invalid_end = rows;
    (*term).width = cols;
    (*term).height = rows;
    vterminal_invalidate_terminal(term, -1, -1);
    1
}

// TODO: Make this work again
// #[no_mangle]
// pub extern "C" fn rust_syms_of_vterm() {
//     def_lisp_sym!(Qvtermp, "vtermp");
// }

#[lisp_fn]
pub fn vterminal_linenum(vterm: LispVterminalRef) -> LispObject {
    LispObject::from((*vterm).linenum)
}

#[lisp_fn]
pub fn vterminal_linenum_added(vterm: LispVterminalRef) -> LispObject {
    LispObject::from((*vterm).linenum_added)
}

#[lisp_fn]
pub fn vterminal_height(vterm: LispVterminalRef) -> LispObject {
    LispObject::from((*vterm).height)
}

#[lisp_fn]
pub fn vterminal_width(vterm: LispVterminalRef) -> LispObject {
    LispObject::from((*vterm).width)
}

#[lisp_fn]
pub fn vterminal_sb_current(vterm: LispVterminalRef) -> LispObject {
    LispObject::from((*vterm).sb_current as i32)
}

#[lisp_fn]
pub fn vterminal_sb_size(vterm: LispVterminalRef) -> LispObject {
    LispObject::from((*vterm).sb_size as i32)
}

#[lisp_fn]
pub fn vterminal_sb_pending(vterm: LispVterminalRef) -> LispObject {
    LispObject::from((*vterm).sb_pending)
}

#[lisp_fn]
pub fn vterminal_sb_pending_by_height_decr(vterm: LispVterminalRef) -> LispObject {
    LispObject::from((*vterm).sb_pending_by_height_decr)
}

#[lisp_fn]
pub fn vterminal_cursor_row(vterm: LispVterminalRef) -> LispObject {
    LispObject::from((*vterm).cursor.row)
}

#[lisp_fn]
pub fn vterminal_cursor_col(vterm: LispVterminalRef) -> LispObject {
    LispObject::from((*vterm).cursor.col)
}

#[lisp_fn]
pub fn vterminal_get_cursor_pos(vterm: LispVterminalRef) -> LispObject {
    unsafe {
        let pos = vterm.get_cursorpos();
        LispObject::cons(pos.row, pos.col)
    }
}

#[lisp_fn]
pub fn vterminal_is_eol(vterm: LispVterminalRef) -> LispObject {
    unsafe {
        let pos = vterm.get_cursorpos();
        LispObject::from(vterm.is_eol((*vterm).width, pos.row, pos.col))
    }
}

#[lisp_fn]
pub fn vterminal_line_contents(
    vterm: LispVterminalRef,
    start_row: i32,
    end_row: i32,
    end_col: i32,
) -> LispObject {
    unsafe {
        let mut size = ((end_row - start_row + 1) * end_col) * 4;
        let mut v: Vec<c_char> = Vec::with_capacity(size as usize);
        let mut lastcell: VTermScreenCell = vterm.fetch_cell(start_row, 0);

        let mut length = 0;

        let mut i = start_row;
        while i < end_row {
            let mut j = 0;

            while j < end_col {
                let cell = vterm.fetch_cell(i, j);
                lastcell = cell;
                if cell.chars[0] == 0 {
                    if vterm.is_eol(end_col, i, j) {
                        /* This cell is EOL if this and every cell to the right is black */
                        break;
                    }

                    // v.insert(length as usize, ' ' as c_char);
                    v.push(' ' as c_char);
                    length += 1;
                } else {
                    // make this a function
                    let mut bytes: [c_uchar; 4] = std::mem::zeroed();
                    let size = cell.to_utf8(&mut bytes);
                    for n in 0..size {
                        // v.insert(length as usize + n, bytes[n] as c_char);
                        v.push(' ' as c_char);
                    }
                    length += size as i32;
                }

                if cell.width > 1 {
                    let w = cell.width - 1;
                    j = j + w as i32;
                }
                j += 1;
            }
            v.push('\n' as c_char);
            length += 1;
            i += 1;
        }
        make_string(v.as_mut_ptr(), length as isize)
    }
}

// insert 10 lines
// grab lines with fetch_cell line by line

include!(concat!(env!("OUT_DIR"), "/vterm_exports.rs"));
