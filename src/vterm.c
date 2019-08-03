#include <config.h>
#ifdef HAVE_LIBVTERM

#include <inttypes.h>
#include <stdbool.h>
#include <assert.h>
#include <limits.h>
#include <string.h>
#include <unistd.h>

#include "vterm.h"

#include "lisp.h"
#include "coding.h"


static int vterminal_sb_push(int cols, const VTermScreenCell *cells, void *data) {
  vterminal *term = (vterminal *)data;

  if (!term->sb_size) {
    return 0;
  }

  // copy vterm cells into sb_buffer
  size_t c = (size_t)cols;
  VtermScrollbackLine *sbrow = NULL;
  if (term->sb_current == term->sb_size) {
    if (term->sb_buffer[term->sb_current - 1]->cols == c) {
      // Recycle old row if it's the right size
      sbrow = term->sb_buffer[term->sb_current - 1];
    } else {
      free(term->sb_buffer[term->sb_current - 1]);
    }

    // Make room at the start by shifting to the right.
    memmove(term->sb_buffer + 1, term->sb_buffer,
            sizeof(term->sb_buffer[0]) * (term->sb_current - 1));

  } else if (term->sb_current > 0) {
    // Make room at the start by shifting to the right.
    memmove(term->sb_buffer + 1, term->sb_buffer,
            sizeof(term->sb_buffer[0]) * term->sb_current);
  }

  if (!sbrow) {
    sbrow = malloc(sizeof(VtermScrollbackLine) + c * sizeof(sbrow->cells[0]));
    sbrow->cols = c;
  }

  // New row is added at the start of the storage buffer.
  term->sb_buffer[0] = sbrow;
  if (term->sb_current < term->sb_size) {
    term->sb_current++;
  }

  if (term->sb_pending < (int)term->sb_size) {
    term->sb_pending++;
  }

  memcpy(sbrow->cells, cells, sizeof(cells[0]) * c);

  return 1;
}

/// Scrollback pop handler (from pangoterm).
///
/// @param cols
/// @param cells  VTerm state to update.
/// @param data   Term
static int vterminal_sb_pop(int cols, VTermScreenCell *cells, void *data) {
  vterminal *term = (vterminal *)data;

  if (!term->sb_current) {
    return 0;
  }

  if (term->sb_pending) {
    term->sb_pending--;
  }

  VtermScrollbackLine *sbrow = term->sb_buffer[0];
  term->sb_current--;
  // Forget the "popped" row by shifting the rest onto it.
  memmove(term->sb_buffer, term->sb_buffer + 1,
          sizeof(term->sb_buffer[0]) * (term->sb_current));

  size_t cols_to_copy = (size_t)cols;
  if (cols_to_copy > sbrow->cols) {
    cols_to_copy = sbrow->cols;
  }

  // copy to vterm state
  memcpy(cells, sbrow->cells, sizeof(cells[0]) * cols_to_copy);
  size_t col;
  for (col = cols_to_copy; col < (size_t)cols; col++) {
    cells[col].chars[0] = 0;
    cells[col].width = 1;
  }

  free(sbrow);

  return 1;
}

size_t
codepoint_to_utf8(const uint32_t codepoint, unsigned char buffer[4]) {
  if (codepoint <= 0x7F) {
    buffer[0] = codepoint;
    return 1;
  }
  if (codepoint >= 0x80 && codepoint <= 0x07FF) {
    buffer[0] = 0xC0 | (codepoint >> 6);
    buffer[1] = 0x80 | (codepoint & 0x3F);
    return 2;
  }
  if (codepoint >= 0x0800 && codepoint <= 0xFFFF) {
    buffer[0] = 0xE0 | (codepoint >> 12);
    buffer[1] = 0x80 | ((codepoint >> 6) & 0x3F);
    buffer[2] = 0x80 | (codepoint & 0x3F);
    return 3;
  }

  if (codepoint >= 0x10000 && codepoint <= 0x10FFFF) {
    buffer[0] = 0xF0 | (codepoint >> 18);
    buffer[1] = 0x80 | ((codepoint >> 12) & 0x3F);
    buffer[2] = 0x80 | ((codepoint >> 6) & 0x3F);
    buffer[3] = 0x80 | (codepoint & 0x3F);
    return 4;
  }
  return 0;
}

bool
utf8_to_codepoint(const unsigned char buffer[4], const size_t len,
                       uint32_t *codepoint) {
  *codepoint = 0;
  if (len == 1 && buffer[0] <= 0x7F) {
    *codepoint = buffer[0];
    return true;
  }
  if (len == 2 && (buffer[0] >= 0xC0 && buffer[0] <= 0xDF) &&
      (buffer[1] >= 0x80 && buffer[1] <= 0xBF)) {
    *codepoint = buffer[0] & 0x1F;
    *codepoint = *codepoint << 6;
    *codepoint = *codepoint | (buffer[1] & 0x3F);
    return true;
  }
  if (len == 3 && (buffer[0] >= 0xE0 && buffer[0] <= 0xEF) &&
      (buffer[1] >= 0x80 && buffer[1] <= 0xBF) &&
      (buffer[2] >= 0x80 && buffer[2] <= 0xBF)) {
    *codepoint = buffer[0] & 0xF;
    *codepoint = *codepoint << 6;
    *codepoint = *codepoint | (buffer[1] & 0x3F);
    *codepoint = *codepoint << 6;
    *codepoint = *codepoint | (buffer[2] & 0x3F);
    return true;
  }
  if (len == 4 && (buffer[0] >= 0xF0 && buffer[0] <= 0xF7) &&
      (buffer[1] >= 0x80 && buffer[1] <= 0xBF) &&
      (buffer[2] >= 0x80 && buffer[2] <= 0xBF) &&
      (buffer[3] >= 0x80 && buffer[3] <= 0xBF)) {
    *codepoint = buffer[0] & 7;
    *codepoint = *codepoint << 6;
    *codepoint = *codepoint | (buffer[1] & 0x3F);
    *codepoint = *codepoint << 6;
    *codepoint = *codepoint | (buffer[2] & 0x3F);
    *codepoint = *codepoint << 6;
    *codepoint = *codepoint | (buffer[3] & 0x3F);
    return true;
  }

  return false;
}

VTermScreenCallbacks vterm_screen_callbacks = {
    .damage = vterminal_damage,
    .moverect = vterminal_moverect,
    .movecursor = vterminal_movecursor,
    .settermprop = vterminal_settermprop,
    .resize = vterminal_resize,
    .sb_pushline = vterminal_sb_push,
    .sb_popline = vterminal_sb_pop,
};

int row_to_linenr(vterminal *term, int row) {
  return row != INT_MAX ? row + (int)term->sb_current + 1 : INT_MAX;
}

void
fetch_cell(vterminal *term, int row, int col, VTermScreenCell *cell) {
  if (row < 0) {
    VtermScrollbackLine *sbrow = term->sb_buffer[-row - 1];
    if ((size_t)col < sbrow->cols) {
      *cell = sbrow->cells[col];
    } else {
      // fill the pointer with an empty cell
      VTermColor fg, bg;
      VTermState *state = vterm_obtain_state(term->vt);
      vterm_state_get_default_colors(state, &fg, &bg);

      *cell = (VTermScreenCell){.chars = {0}, .width = 1, .bg = bg};
    }
  } else {
    vterm_screen_get_cell(term->vts, (VTermPos){.row = row, .col = col}, cell);
  }
}

bool
is_eol(vterminal *term, int end_col, int row, int col) {
  /* This cell is EOL if this and every cell to the right is black */
  if (row >= 0) {
    VTermPos pos = {.row = row, .col = col};
    return vterm_screen_is_eol(term->vts, pos);
  }

  VtermScrollbackLine *sbrow = term->sb_buffer[-row - 1];
  int c;
  for (c = col; c < end_col && c < sbrow->cols;) {
    if (sbrow->cells[c].chars[0]) {
      return 0;
    }
    c += sbrow->cells[c].width;
  }
  return 1;
}

Lisp_Object
color_to_rgb_string(vterminal *term, VTermColor *color) {
  if (VTERM_COLOR_IS_DEFAULT_FG(color)) {
    return call1 (intern ("vterm--get-color") ,make_number (-1));
  }
  if (VTERM_COLOR_IS_DEFAULT_BG(color)) {
    return call1 (intern ("vterm--get-color"), make_number (-2));
  }
  if (VTERM_COLOR_IS_INDEXED(color)) {
    if (color->indexed.idx < 16) {
      return call1 (intern ("vterm--get-color"), make_number (color->indexed.idx));
    } else {
      VTermState *state = vterm_obtain_state(term->vt);
      vterm_state_get_palette_color(state, color->indexed.idx, color);
    }
  } else if (VTERM_COLOR_IS_RGB(color)) {
    /* do nothing just use the argument color directly */
  }

  char buffer[8];
  snprintf(buffer, 8, "#%02X%02X%02X", color->rgb.red, color->rgb.green,
           color->rgb.blue);
  return make_string(buffer, 7);

}

#endif
