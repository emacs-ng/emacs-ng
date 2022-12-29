#ifndef __WRTERM_H_
#define __WRTERM_H_

#include "dispextern.h"

struct wr_bitmap_record
{
  char *file;
  int refcount;
  int height, width, depth;
};

typedef int Screen;

struct wr_display_info
{
  /* Chain of all wr_display_info structures.  */
  struct wr_display_info *next;

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

extern struct wr_display_info *wr_display_list;
#define x_display_list wr_display_list

struct wr_output
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
  struct wr_display_info *display_info;
};

typedef struct wr_output wr_output;
typedef struct wr_display_info wr_display_info;

extern Window wr_get_window_desc(wr_output* output);
extern int wr_get_fontset(wr_output* output);
extern struct font *wr_get_font(wr_output* output);
extern wr_display_info *wr_get_display_info(wr_output* output);
extern Display *wr_get_display(wr_display_info* output);
extern Screen wr_get_screen(wr_display_info* output);
extern int wr_get_baseline_offset(wr_output* output);
extern int wr_get_pixel(WRImage *ximg, int x, int y);
extern int wr_put_pixel(WRImage *ximg, int x, int y, unsigned long pixel);
extern bool wr_load_image (struct frame *f, struct image *img,
			   Lisp_Object spec_file, Lisp_Object spec_data);
extern bool wr_can_use_native_image_api (Lisp_Object type);

extern void wr_transform_image(struct frame *f, struct image *img, int width, int height, double rotation);

extern int wr_select (int nfds, fd_set *readfds, fd_set *writefds,
		       fd_set *exceptfds, struct timespec *timeout,
		       sigset_t *sigmask);

/* This is the `Display *' which frame F is on.  */
#define FRAME_X_DISPLAY(f) (wr_get_display(FRAME_DISPLAY_INFO (f)))

/* This gives the x_display_info structure for the display F is on.  */
#define FRAME_DISPLAY_INFO(f) (wr_get_display_info(FRAME_X_OUTPUT (f)))

/* Return the X output data for frame F.  */
#define FRAME_X_OUTPUT(f) ((f)->output_data.wr)

#define FRAME_OUTPUT_DATA(f) FRAME_X_OUTPUT (f)

#define FRAME_BASELINE_OFFSET(f) (wr_get_baseline_offset(FRAME_X_OUTPUT (f)))

/* This is the `Screen *' which frame F is on.  */
#define FRAME_X_SCREEN(f) (wr_get_display_info(FRAME_X_OUTPUT (f)))

/* Return the X window used for displaying data in frame F.  */
#define FRAME_X_WINDOW(f)  (wr_get_window_desc(FRAME_X_OUTPUT (f)))
#define FRAME_NATIVE_WINDOW(f) FRAME_X_WINDOW (f)

#define FRAME_FONTSET(f) (wr_get_fontset(FRAME_X_OUTPUT (f)))
#define FRAME_FONT(f) (wr_get_font(FRAME_X_OUTPUT (f)))


#define BLACK_PIX_DEFAULT(f) 0
#define WHITE_PIX_DEFAULT(f) 65535

extern const char *app_bundle_relocate (const char *);

/* Symbol initializations implemented in each pgtk sources. */
extern void syms_of_wrterm(void);
extern void syms_of_wrterm_rust(void);
extern void syms_of_wrfns (void);

EXFUN (Fxw_display_color_p, 1);
EXFUN (Fx_display_grayscale_p, 1);
EXFUN (Fx_hide_tip, 0);

#endif // __WRTERM_H_
