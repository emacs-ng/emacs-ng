#ifndef __WRTERM_H_
#define __WRTERM_H_

#include "dispextern.h"

typedef int Screen;

struct winit_display_info
{
  /* Chain of all winit_display_info structures.  */
  struct winit_display_info *next;

  /* The generic display parameters corresponding to this NS display.  */
  struct terminal *terminal;

  /* This is a cons cell of the form (NAME . FONT-LIST-CACHE).  */
  Lisp_Object name_list_element;

  /* Dots per inch of the screen.  */
  double resx, resy;

  /* Number of planes on this screen.  */
  int n_planes;

  /* Mask of things that cause the mouse to be grabbed.  */
  int grabbed;

  /* The root window of this screen.  */
  Window root_window;

  /* The cursor to use for vertical scroll bars.  */
  Emacs_Cursor vertical_scroll_bar_cursor;

  /* Resource data base */
  XrmDatabase rdb;

  /* Minimum width over all characters in all fonts in font_table.  */
  int smallest_char_width;

  /* Minimum font height over all fonts in font_table.  */
  int smallest_font_height;

  /* Information about the range of text currently shown in
     mouse-face.  */
  Mouse_HLInfo mouse_highlight;

  /* The number of fonts actually stored in wr_font_table.
     font_table[n] is used and valid if 0 <= n < n_fonts. 0 <=
     n_fonts <= font_table_size. and font_table[i].name != 0. */
  int n_fonts;

  /* Pointer to bitmap records.  */
  struct wr_bitmap_record *bitmaps;

  /* Allocated size of bitmaps field.  */
  ptrdiff_t bitmaps_size;

  /* Last used bitmap index.  */
  ptrdiff_t bitmaps_last;


  /* The frame which currently has the visual highlight, and should get
     keyboard input (other sorts of input have the frame encoded in the
     event).  It points to the focus frame's selected window's
     frame. */
  struct frame *highlight_frame;

  /* The frame where the mouse was last time we reported a mouse event.  */
  struct frame *last_mouse_frame;

  /* The frame where the mouse was last time we reported a mouse motion.  */
  struct frame *last_mouse_motion_frame;

  /* Position where the mouse was last time we reported a motion.
     This is a position on last_mouse_motion_frame.  */
  int last_mouse_motion_x;
  int last_mouse_motion_y;

  /* Inner perporty in Rust */
  void *inner;
};

extern struct winit_display_info *winit_display_list;
#define x_display_list winit_display_list

struct winit_output
{

  /* The X window that is the parent of this X window.
     Usually this is a window that was made by the window manager,
     but it can be the root window, and it can be explicitly specified
     (see the explicit_parent field, below).  */
  Window parent_desc;

  /* Descriptor for the cursor in use for this window.  */
  Emacs_Cursor text_cursor;
  Emacs_Cursor nontext_cursor;
  Emacs_Cursor modeline_cursor;
  Emacs_Cursor hand_cursor;
  Emacs_Cursor hourglass_cursor;
  Emacs_Cursor horizontal_drag_cursor;
  Emacs_Cursor vertical_drag_cursor;
  Emacs_Cursor left_edge_cursor;
  Emacs_Cursor top_left_corner_cursor;
  Emacs_Cursor top_edge_cursor;
  Emacs_Cursor top_right_corner_cursor;
  Emacs_Cursor right_edge_cursor;
  Emacs_Cursor bottom_right_corner_cursor;
  Emacs_Cursor bottom_edge_cursor;
  Emacs_Cursor bottom_left_corner_cursor;

  /* This is the Emacs structure for the X display this frame is on.  */
  struct winit_display_info *display_info;

  struct font *font;
  int baseline_offset;

  /* If a fontset is specified for this frame instead of font, this
     value contains an ID of the fontset, else -1.  */
  int fontset; /* only used with font_backend */

  /* Inner perporty in Rust */
  void *inner;
};

typedef struct winit_output winit_output;
typedef struct winit_display_info winit_display_info;

extern Window winit_get_window_desc(winit_output* output);
extern winit_display_info *winit_get_display_info(winit_output* output);


extern Display *winit_get_display(winit_display_info* output);
extern Screen winit_get_screen(winit_display_info* output);
extern int winit_select (int nfds, fd_set *readfds, fd_set *writefds,
		       fd_set *exceptfds, struct timespec *timeout,
		       sigset_t *sigmask);

/* This is the `Display *' which frame F is on.  */
#define FRAME_X_DISPLAY(f) (winit_get_display(FRAME_DISPLAY_INFO (f)))

/* This gives the x_display_info structure for the display F is on.  */
#define FRAME_DISPLAY_INFO(f) (winit_get_display_info(FRAME_X_OUTPUT (f)))

/* Return the X output data for frame F.  */
#define FRAME_X_OUTPUT(f) ((f)->output_data.winit)

#define FRAME_OUTPUT_DATA(f) FRAME_X_OUTPUT (f)

/* This is the `Screen *' which frame F is on.  */
#define FRAME_X_SCREEN(f) (winit_get_display_info(FRAME_X_OUTPUT (f)))

/* Return the X window used for displaying data in frame F.  */
#define FRAME_X_WINDOW(f)  (winit_get_window_desc(FRAME_X_OUTPUT (f)))
#define FRAME_NATIVE_WINDOW(f) FRAME_X_WINDOW (f)
#define FRAME_FONT(f)             (FRAME_X_OUTPUT (f)->font)
#define FRAME_FONTSET(f) (FRAME_X_OUTPUT (f)->fontset)
#define FRAME_BASELINE_OFFSET(f) (FRAME_X_OUTPUT (f)->baseline_offset)

extern const char *app_bundle_relocate (const char *);

/* Symbol initializations implemented in each pgtk sources. */
extern void syms_of_winit_term(void);

#include "webrender_ffi.h"

#endif // __WRTERM_H_
