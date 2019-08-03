//! libvterm facilities

use remacs_macros::lisp_fn;

use libc::{c_char, c_uchar, c_void, size_t};

use std::{cmp, mem};

use crate::remacs_sys::color_to_rgb_string;

use crate::{
    lisp::ExternalPtr,
    lisp::LispObject,
    obarray::intern,
    remacs_sys::{
        code_convert_string_norecord, del_range, looking_at_1, make_string, pvec_type,
        send_process, vterminal, EmacsInt, Fforward_char, Fget_buffer_window, Flength,
        Fline_end_position, Fput_text_property, Frecenter, Fselected_window, Fset, Lisp_Type,
        Qbold, Qcursor_type, Qface, Qitalic, Qnil, Qnormal, Qt, Qutf_8, STRING_BYTES, Finsert
            // ,Qvtermp
    },

    remacs_sys::{
        codepoint_to_utf8, fetch_cell, is_eol, row_to_linenr, search_command, set_point,
        utf8_to_codepoint, vterm_output_read, vterm_screen_callbacks, vterm_screen_set_callbacks,
        VtermScrollbackLine,
    },

    // libvterm
    remacs_sys::{
        vterm_color_is_equal,  vterm_input_write, vterm_keyboard_key,
        vterm_keyboard_unichar, vterm_new, vterm_obtain_screen, vterm_obtain_state,
        vterm_output_get_buffer_current, vterm_screen_enable_altscreen, vterm_screen_flush_damage,
        vterm_screen_reset, vterm_screen_set_damage_merge, vterm_set_size, vterm_set_utf8,
        vterm_state_get_cursorpos, VTermDamageSize, VTermKey, VTermModifier, VTermPos, VTermProp,
        VTermRect, VTermScreenCell, VTermState, VTermValue,
    },

    threads::ThreadState,
};

pub type LispVterminalRef = ExternalPtr<vterminal>;

impl LispVterminalRef {
    pub fn set_size(self, rows: i32, cols: i32) {
        unsafe {
            vterm_set_size((*self).vt, rows, cols);
        }
    }
}

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
            .unwrap_or_else(|| wrong_type!(Qt, self))
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

/// Start a libvterm terminal-emulator in a new buffer.
/// The maximum scrollback is set by the argument SCROLLBACK.
/// You can customize the value with `vterm-max-scrollback`.
///
/// libvterm requires several callback functions that are stored
/// in VTermScreenCallbacks.
#[lisp_fn(name = "vterm-new")]
pub fn vterminal_new_lisp(
    rows: EmacsInt,
    cols: EmacsInt,
    process: LispObject,
    scrollback: EmacsInt,
) -> LispVterminalRef {
    unsafe {
        let mut term = allocate_vterm();

        (*term).vt = vterm_new(rows as i32, cols as i32);
        vterm_set_utf8((*term).vt, 1);
        (*term).vts = vterm_obtain_screen((*term).vt);
        vterm_screen_reset((*term).vts, 1);

        vterm_screen_set_callbacks(
            (*term).vts,
            &vterm_screen_callbacks,
            term.as_mut() as *mut libc::c_void,
        );

        vterm_screen_set_damage_merge((*term).vts, VTermDamageSize::VTERM_DAMAGE_SCROLL);

        vterm_screen_enable_altscreen((*term).vts, 1);

        (*term).sb_size = scrollback as usize;
        (*term).sb_current = 0;
        (*term).sb_pending = 0;

        let s = mem::size_of::<VtermScrollbackLine>() * scrollback as usize;
        (*term).sb_buffer = libc::malloc(s as libc::size_t) as *mut *mut VtermScrollbackLine;

        (*term).invalid_start = 0;
        (*term).invalid_end = rows as i32;

        (*term).cursor.visible = true;

        (*term).width = cols as i32;
        (*term).height = rows as i32;

        (*term).buffer = LispObject::from(ThreadState::current_buffer_unchecked());
        (*term).process = process;

        term
    }
}

unsafe fn compare_cells(a: *mut VTermScreenCell, b: *mut VTermScreenCell) -> bool {
    let a_attr = (*a).attrs;
    let b_attr = (*b).attrs;

    vterm_color_is_equal(&mut (*a).fg, &mut (*b).fg) > 0
        && vterm_color_is_equal(&mut (*a).bg, &mut (*b).bg) > 0
        && (a_attr.bold() == b_attr.bold())
        && (a_attr.underline() == b_attr.underline())
        && (a_attr.italic() == b_attr.italic())
        && (a_attr.reverse() == b_attr.reverse())
        && (a_attr.strike() == b_attr.strike())
}

/// Return process of terminal VTERM
#[lisp_fn(name = "vterm-process")]
pub fn vterminal_process(vterm: LispVterminalRef) -> LispObject {
    (*vterm).process
}

unsafe fn refresh_lines(mut vterm: LispVterminalRef, start_row: i32, end_row: i32, end_col: i32) {
    let mut size = ((end_row - start_row + 1) * end_col) * 4;
    let mut v: Vec<c_char> = Vec::with_capacity(size as usize);

    let mut cell: VTermScreenCell = std::mem::zeroed();
    let mut lastcell: VTermScreenCell = std::mem::zeroed();
    fetch_cell(vterm.as_mut(), start_row, 0, &mut lastcell);

    let mut length = 0;
    let mut offset = 0;

    let mut i = start_row;
    while i < end_row {
        let mut j = 0;
        while j < end_col {
            fetch_cell(vterm.as_mut(), i, j, &mut cell);

            if !compare_cells(&mut cell, &mut lastcell) {
                let mut text = vterminal_render_text(vterm, v.as_mut_ptr(), length, &mut lastcell);
                Finsert (1, &mut text);

                size -= length;
                v = Vec::with_capacity(size as usize);
                length = 0;
            }

            lastcell = cell;
            if cell.chars[0] == 0 {
                if is_eol(vterm.as_mut(), end_col, i, j) {
                    /* This cell is EOL if this and every cell to the right is black */
                    break;
                }

                v.push(' ' as c_char);
                length += 1;
            } else {
                let mut bytes: [c_uchar; 4] = std::mem::zeroed();
                let count = codepoint_to_utf8(cell.chars[0], bytes.as_mut_ptr());

                let mut k = 0;
                while k < count {
                    v.push(bytes[k] as c_char);
                    length += 1;
                    k += 1;
                }
            }

            if cell.width > 1 {
                let w = cell.width - 1;
                offset += w;
                j = j + w as i32;
            }
            j += 1;
        }

        v.push('\n' as c_char);
        length += 1;
        i += 1;
    }

    let mut text = vterminal_render_text(vterm, v.as_mut_ptr(), length, &mut lastcell);
    Finsert (1, &mut text);
}

/// Refresh the screen (visible part of the buffer when the terminal is focused)
/// of a invalidated terminal
unsafe fn vterminal_refresh_screen(mut term: LispVterminalRef) {
    // Term height may have decreased before `invalid_end` reflects it.
    (*term).invalid_end = cmp::min((*term).invalid_end, (*term).height);

    if (*term).invalid_end >= (*term).invalid_start {
        let line_start = row_to_linenr(term.as_mut() as *mut vterminal, (*term).invalid_start);

        vterminal_goto_line(line_start as EmacsInt);

        let line_count = (*term).invalid_end - (*term).invalid_start;
        vterminal_delete_lines(
            line_start as EmacsInt,
            LispObject::from(line_count as EmacsInt),
        );

        refresh_lines(
            term,
            (*term).invalid_start,
            (*term).invalid_end,
            (*term).width,
        );
    }
    (*term).invalid_start = std::i32::MAX;
    (*term).invalid_end = -1;
}

unsafe fn get_col_offset(mut vterm: LispVterminalRef, row: i32, end_col: i32) -> i32 {
    let mut offset: size_t = 0;

    let mut col: i32 = 0;
    while col < end_col {
        let mut cell: VTermScreenCell = std::mem::zeroed();
        fetch_cell(vterm.as_mut(), row, col, &mut cell);

        if cell.chars[0] > 0 {
            if cell.width > 0 {
                offset += cell.width as size_t - 1;
            }
        } else if is_eol(vterm.as_mut(), (*vterm).width, row, col) {
            offset += cell.width as size_t;
        }
        col += cell.width as i32;
    }
    offset as i32
}

unsafe fn vterminal_adjust_topline(mut term: LispVterminalRef, added: i32) {
    let buffer_lnum = vterminal_count_lines();

    let state: *mut VTermState = vterm_obtain_state((*term).vt);
    let mut pos: VTermPos = std::mem::zeroed();
    vterm_state_get_cursorpos(state, &mut pos);

    let cursor_lnum = row_to_linenr(term.as_mut() as *mut vterminal, pos.row);

    vterminal_goto_line(cmp::min(cursor_lnum, buffer_lnum) as EmacsInt);

    let offset = get_col_offset(term, pos.row, pos.col);

    Fforward_char(LispObject::from((pos.col - offset as i32) as EmacsInt));

    let following = buffer_lnum == cursor_lnum + added; // cursor at end?

    let window = Fget_buffer_window(term.buffer, Qt);
    let swindow = Fselected_window();

    if window.eq(swindow) {
        if following {
            // "Follow" the terminal output
            Frecenter(LispObject::from(-1));
        } else {
            Frecenter(LispObject::from(pos.row));
        }
    }
}

/// Refresh the scrollback of an invalidated terminal.
unsafe fn vterminal_refresh_scrollback(mut term: LispVterminalRef) {
    let mut buffer_lnum: i32;

    if (*term).sb_pending > 0 {
        buffer_lnum = vterminal_count_lines();

        let del_cnt =
            buffer_lnum as i32 - (*term).height - (*term).sb_size as i32 + (*term).sb_pending;

        if del_cnt > 0 {
            vterminal_delete_lines(1, LispObject::from(del_cnt as EmacsInt));
            buffer_lnum = vterminal_count_lines();
        }

        let buf_index = buffer_lnum as i32 - (*term).height + 1;
        vterminal_goto_line(buf_index as EmacsInt);

        refresh_lines(term, -(*term).sb_pending, 0, (*term).width);
        (*term).sb_pending = 0;
    }

    let max_line_count = (*term).sb_current as i32 + (*term).height;
    buffer_lnum = vterminal_count_lines();

    // Remove extra lines at the bottom
    if buffer_lnum as i32 > max_line_count {
        let line_count = buffer_lnum as i32 - max_line_count + 1;
        vterminal_delete_lines(
            max_line_count as EmacsInt + 1,
            LispObject::from(line_count as EmacsInt),
        );
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

        vterminal_flush_output(vterm);
        vterminal_redraw(vterm);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vterminal_render_text(
    mut vterm: LispVterminalRef,
    buffer: *mut c_char,
    len: i32,
    cell: *mut VTermScreenCell,
) -> LispObject {
    if len == 0 {
        make_string("".as_ptr() as *mut c_char, 0)
    } else {
        let text = make_string(buffer, len as isize);

        let start = LispObject::from(0);
        let end = LispObject::from(EmacsInt::from(Flength(text)));
        let properties = list!(
            LispObject::from(intern(":foreground")),
            color_to_rgb_string(vterm.as_mut(), &mut (*cell).fg),
            LispObject::from(intern(":background")),
            color_to_rgb_string(vterm.as_mut(), &mut (*cell).bg),
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
}

/// Send current contents of VTERM to the running shell process
unsafe fn vterminal_flush_output(vterm: LispVterminalRef) {
    let len = vterm_output_get_buffer_current((*vterm).vt);
    if len > 0 {
        let mut buffer: Vec<c_char> = Vec::with_capacity(len);
        let len = vterm_output_read((*vterm).vt, buffer.as_mut_ptr() as *mut c_char, len);

        let lisp_string = make_string(buffer.as_mut_ptr() as *mut c_char, len as isize);

        send_process(
            (*vterm).process,
            buffer.as_mut_ptr() as *mut c_char,
            len as isize,
            lisp_string,
        );
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
pub fn vterminal_set_size_lisp(vterm: LispVterminalRef, rows: EmacsInt, cols: EmacsInt) {
    unsafe {
        if cols as i32 != (*vterm).width || rows as i32 != (*vterm).height {
            vterm.set_size(rows as i32, cols as i32);
            vterm_screen_flush_damage((*vterm).vts);
            vterminal_redraw(vterm);
        }
    }
}

/// Refresh cursor, scrollback and screen.
/// Also adjust the top line.
unsafe fn vterminal_redraw(mut vterm: LispVterminalRef) {
    if vterm.is_invalidated {
        if (*vterm).cursor.visible {
            Fset(Qcursor_type, Qt);
        } else {
            Fset(Qcursor_type, Qnil);
        }

        let bufline_before = vterminal_count_lines();

        vterminal_refresh_scrollback(vterm);
        vterminal_refresh_screen(vterm);

        let line_added = vterminal_count_lines() - bufline_before;

        vterminal_adjust_topline(vterm, line_added);
    }
    vterm.is_invalidated = false;
}

/// Delete COUNT lines starting from LINENUM.
#[lisp_fn]
pub fn vterminal_delete_lines(linenum: EmacsInt, count: LispObject) {
    let mut cur_buf = ThreadState::current_buffer_unchecked();
    let orig_pt = cur_buf.pt;

    vterminal_goto_line(linenum);

    let start = cur_buf.pt;
    unsafe {let end = EmacsInt::from(Fline_end_position(count)) as isize;
    
    del_range(start, end) };

    let pos = cur_buf.pt;
    unsafe {
        if !looking_at_1(make_string("\n".as_ptr() as *mut c_char, 1), false).is_nil() {
            del_range(pos, pos + 1);
        }
    }

    unsafe { set_point(cmp::min(orig_pt, cur_buf.zv)) };
}

/// Count lines in current buffer.
#[lisp_fn]
pub fn vterminal_count_lines() -> i32 {
    let cur_buf = ThreadState::current_buffer_unchecked();
    let orig_pt = cur_buf.pt;

    unsafe { set_point(cur_buf.beg()) };

    let mut count: i32 = 0; // same as count-lines
    let regexp = unsafe { make_string("\n".as_ptr() as *mut c_char, 1) };
    while unsafe { !search_command(regexp, Qnil, Qt, LispObject::from(1), 1, 0, false).is_nil() } {
        count += 1;
    }

    if !(cur_buf.pt == cur_buf.begv || cur_buf.fetch_byte(cur_buf.pt_byte - 1) == b'\n') {
        count += 1;
    }

    unsafe { set_point(orig_pt) };

    count
}

#[lisp_fn]
pub fn vterminal_goto_line(line: EmacsInt) {
    unsafe { set_point(1) };

    let regexp = unsafe { make_string("\n".as_ptr() as *mut c_char, 1) };

    unsafe { search_command(regexp, Qnil, Qt, LispObject::from(line - 1), 1, 0, false) };
}

// vterm_screen_callbacks

#[no_mangle]
pub unsafe extern "C" fn vterminal_settermprop(
    prop: VTermProp,
    val: *mut VTermValue,
    user_data: *mut c_void,
) -> i32 {
    let term = user_data as *mut vterminal;

    match prop {
        VTermProp::VTERM_PROP_ALTSCREEN => vterminal_invalidate_terminal(term, 0, (*term).height),
        VTermProp::VTERM_PROP_CURSORVISIBLE => {
            vterminal_invalidate_terminal(term, (*term).cursor.row, (*term).cursor.row + 1);
            (*term).cursor.visible = (*val).boolean != 0;
        }
        _ => return 0,
    }

    1
}

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

#[no_mangle]
pub extern "C" fn rust_syms_of_vterm() {
    def_lisp_sym!(Qvtermp, "vtermp");
}

include!(concat!(env!("OUT_DIR"), "/vterm_exports.rs"));
