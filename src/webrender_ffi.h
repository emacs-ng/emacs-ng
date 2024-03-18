#if defined (HAVE_PGTK)
typedef struct pgtk_output output;
#elif defined (HAVE_WINIT)
typedef struct winit_output output;
#endif

extern int wr_get_fontset(output* output);
extern struct font *wr_get_font(output* output);
extern int wr_get_baseline_offset(output* output);
extern int wr_get_pixel(WRImage *ximg, int x, int y);
extern int wr_put_pixel(WRImage *ximg, int x, int y, unsigned long pixel);
extern bool wr_load_image (struct frame *f, struct image *img,
			   Lisp_Object spec_file, Lisp_Object spec_data);
extern bool wr_can_use_native_image_api (Lisp_Object type);
extern void wr_transform_image(struct frame *f, struct image *img, int width, int height, double rotation);

extern void wr_scroll_run (struct window *w, struct run *run);

extern void wr_update_window_begin (struct window *);
extern void wr_update_window_end (struct window *, bool, bool);
extern void wr_after_update_window_line (struct window *w,
					 struct glyph_row *desired_row);
extern void wr_flush_display (struct frame *f);
extern void
wr_draw_fringe_bitmap (struct window *w, struct glyph_row *row,
		       struct draw_fringe_bitmap_params *p);
extern void
wr_draw_glyph_string (struct glyph_string *s);
extern void wr_clear_frame_area (struct frame *, int, int, int, int);
extern void
wr_draw_window_cursor (struct window *w, struct glyph_row *glyph_row, int x,
			 int y, enum text_cursor_kinds cursor_type,
		       int cursor_width, bool on_p, bool active_p);
extern void
wr_draw_vertical_window_border (struct window *w, int x, int y0, int y1);
extern void
wr_draw_window_divider (struct window *w, int x0, int x1, int y0, int y1);
extern void
wr_free_pixmap (struct frame *f, Emacs_Pixmap pixmap);
extern void
wr_update_end (struct frame *f);
extern Lisp_Object wr_new_font (struct frame *f, Lisp_Object font_object, int fontset);
extern bool wr_defined_color (struct frame *, const char *, Emacs_Color *,
                               bool, bool);
extern void wr_clear_frame (struct frame *);

#if defined USE_WEBRENDER && defined HAVE_PGTK
extern int
gl_renderer_parse_color (struct frame *f, const char *color_name,
		Emacs_Color * color);
#endif

extern void
gl_clear_under_internal_border (struct frame *f);
extern void
gl_renderer_free_frame_resources (struct frame *f);
extern void
gl_renderer_free_terminal_resources (struct terminal *f);
extern void
gl_renderer_fit_context (struct frame *f);

extern void syms_of_webrender(void);

#define BLACK_PIX_DEFAULT(f) 0
#define WHITE_PIX_DEFAULT(f) 65535
