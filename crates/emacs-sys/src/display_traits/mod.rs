// Interface definitions for display code.
#[cfg(use_webrender)]
use webrender_api::LineStyle;

use crate::bindings::draw_glyphs_face;
use crate::bindings::face_id;
use crate::bindings::face_underline_type;
use crate::bindings::glyph_row_area;
use crate::bindings::glyph_type;
use crate::bindings::image_cache as ImageCache;
use crate::bindings::resource_types;
use crate::bindings::text_cursor_kinds;
use crate::bindings::Emacs_GC as EmacsGC;

use crate::bindings::glyph_row as GlyphRow;
use crate::globals::Qalpha;
use crate::globals::Qalpha_background;
use crate::globals::Qauto_lower;
use crate::globals::Qauto_raise;
use crate::globals::Qbackground_color;
use crate::globals::Qborder_color;
use crate::globals::Qborder_width;
use crate::globals::Qbottom_divider_width;
use crate::globals::Qbuffer_predicate;
use crate::globals::Qchild_frame_border_width;
use crate::globals::Qcursor_color;
use crate::globals::Qcursor_type;
use crate::globals::Qdisplay;
use crate::globals::Qfont;
use crate::globals::Qfont_backend;
use crate::globals::Qforeground_color;
use crate::globals::Qfullscreen;
use crate::globals::Qheight;
use crate::globals::Qhorizontal_scroll_bars;
use crate::globals::Qicon_name;
use crate::globals::Qicon_type;
use crate::globals::Qinhibit_double_buffering;
use crate::globals::Qinternal_border_width;
use crate::globals::Qleft_fringe;
use crate::globals::Qline_spacing;
use crate::globals::Qmenu_bar_lines;
use crate::globals::Qmin_height;
use crate::globals::Qmin_width;
use crate::globals::Qminibuffer;
use crate::globals::Qmouse_color;
use crate::globals::Qname;
use crate::globals::Qno_accept_focus;
use crate::globals::Qno_focus_on_map;
use crate::globals::Qno_special_glyphs;
use crate::globals::Qns_appearance;
use crate::globals::Qns_transparent_titlebar;
use crate::globals::Qoverride_redirect;
use crate::globals::Qparent_frame;
use crate::globals::Qparent_id;
use crate::globals::Qright_divider_width;
use crate::globals::Qright_fringe;
use crate::globals::Qscreen_gamma;
use crate::globals::Qscroll_bar_background;
use crate::globals::Qscroll_bar_foreground;
use crate::globals::Qscroll_bar_height;
use crate::globals::Qscroll_bar_width;
use crate::globals::Qshaded;
use crate::globals::Qskip_taskbar;
use crate::globals::Qsticky;
use crate::globals::Qtab_bar_lines;
use crate::globals::Qterminal;
use crate::globals::Qtitle;
use crate::globals::Qtool_bar_lines;
use crate::globals::Qtool_bar_position;
use crate::globals::Qundecorated;
use crate::globals::Qunsplittable;
use crate::globals::Quse_frame_synchronization;
use crate::globals::Qvertical_scroll_bars;
use crate::globals::Qvisibility;
use crate::globals::Qwait_for_wm;
use crate::globals::Qwidth;
use crate::globals::Qz_group;
use crate::lisp::ExternalPtr;
use crate::lisp::LispObject;
use std::ffi::CString;

mod glyph;
pub use glyph::*;
#[cfg(have_window_system)]
mod glyph_string;
#[cfg(have_window_system)]
pub use glyph_string::*;
mod face;
pub use face::*;

pub type ImageCacheRef = ExternalPtr<ImageCache>;
pub type EmacsGCRef = ExternalPtr<EmacsGC>;
pub type GlyphRowRef = ExternalPtr<GlyphRow>;

#[derive(Debug, Eq, PartialEq)]
pub enum GlyphType {
    Char,
    Composite,
    Glyphless,
    Image,
    Stretch,
    Xwidget,
}

impl From<glyph_type::Type> for GlyphType {
    fn from(t: glyph_type::Type) -> Self {
        match t {
            glyph_type::CHAR_GLYPH => GlyphType::Char,
            glyph_type::COMPOSITE_GLYPH => GlyphType::Composite,
            glyph_type::GLYPHLESS_GLYPH => GlyphType::Glyphless,
            glyph_type::IMAGE_GLYPH => GlyphType::Image,
            glyph_type::STRETCH_GLYPH => GlyphType::Stretch,
            glyph_type::XWIDGET_GLYPH => GlyphType::Xwidget,
            _ => panic!("unsupported glyph type"),
        }
    }
}

impl Into<glyph_type::Type> for GlyphType {
    fn into(self) -> glyph_type::Type {
        match self {
            GlyphType::Char => glyph_type::CHAR_GLYPH,
            GlyphType::Composite => glyph_type::COMPOSITE_GLYPH,
            GlyphType::Glyphless => glyph_type::GLYPHLESS_GLYPH,
            GlyphType::Image => glyph_type::IMAGE_GLYPH,
            GlyphType::Stretch => glyph_type::STRETCH_GLYPH,
            GlyphType::Xwidget => glyph_type::XWIDGET_GLYPH,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum DrawGlyphsFace {
    Cursor,
    ImageRaised,
    ImageSunken,
    InverseVideo,
    Mouse,
    NormalText,
}

impl From<draw_glyphs_face::Type> for DrawGlyphsFace {
    fn from(t: draw_glyphs_face::Type) -> Self {
        match t {
            draw_glyphs_face::DRAW_NORMAL_TEXT => DrawGlyphsFace::NormalText,
            draw_glyphs_face::DRAW_INVERSE_VIDEO => DrawGlyphsFace::InverseVideo,
            draw_glyphs_face::DRAW_CURSOR => DrawGlyphsFace::Cursor,
            draw_glyphs_face::DRAW_MOUSE_FACE => DrawGlyphsFace::Mouse,
            draw_glyphs_face::DRAW_IMAGE_RAISED => DrawGlyphsFace::ImageRaised,
            draw_glyphs_face::DRAW_IMAGE_SUNKEN => DrawGlyphsFace::ImageSunken,
            _ => panic!("unsupported draw glyphs face"),
        }
    }
}

impl Into<draw_glyphs_face::Type> for DrawGlyphsFace {
    fn into(self) -> draw_glyphs_face::Type {
        match self {
            DrawGlyphsFace::NormalText => draw_glyphs_face::DRAW_NORMAL_TEXT,
            DrawGlyphsFace::InverseVideo => draw_glyphs_face::DRAW_INVERSE_VIDEO,
            DrawGlyphsFace::Cursor => draw_glyphs_face::DRAW_CURSOR,
            DrawGlyphsFace::Mouse => draw_glyphs_face::DRAW_MOUSE_FACE,
            DrawGlyphsFace::ImageRaised => draw_glyphs_face::DRAW_IMAGE_RAISED,
            DrawGlyphsFace::ImageSunken => draw_glyphs_face::DRAW_IMAGE_SUNKEN,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TextCursorKind {
    Default,
    None,
    FilledBox,
    HollowBox,
    Bar,
    Hbar,
}

impl From<text_cursor_kinds::Type> for TextCursorKind {
    fn from(t: text_cursor_kinds::Type) -> Self {
        match t {
            text_cursor_kinds::DEFAULT_CURSOR => TextCursorKind::Default,
            text_cursor_kinds::NO_CURSOR => TextCursorKind::None,
            text_cursor_kinds::FILLED_BOX_CURSOR => TextCursorKind::FilledBox,
            text_cursor_kinds::HOLLOW_BOX_CURSOR => TextCursorKind::HollowBox,
            text_cursor_kinds::BAR_CURSOR => TextCursorKind::Bar,
            text_cursor_kinds::HBAR_CURSOR => TextCursorKind::Hbar,
            _ => panic!("unsupported text cursor kind"),
        }
    }
}

impl Into<text_cursor_kinds::Type> for TextCursorKind {
    fn into(self) -> text_cursor_kinds::Type {
        match self {
            TextCursorKind::Default => text_cursor_kinds::DEFAULT_CURSOR,
            TextCursorKind::None => text_cursor_kinds::NO_CURSOR,
            TextCursorKind::FilledBox => text_cursor_kinds::FILLED_BOX_CURSOR,
            TextCursorKind::HollowBox => text_cursor_kinds::HOLLOW_BOX_CURSOR,
            TextCursorKind::Bar => text_cursor_kinds::BAR_CURSOR,
            TextCursorKind::Hbar => text_cursor_kinds::HBAR_CURSOR,
        }
    }
}

// TODO window_part

pub enum FaceId {
    Default,
    ModeLineActive,
    ModeLineInactive,
    ToolBar,
    Fringe,
    HeaderLine,
    ScrollBar,
    Border,
    Cursor,
    Mouse,
    Menu,
    VerticalBorder,
    WindowDivider,
    WindowDividerFirstPixel,
    WindowDividerLastPixel,
    InternalBorder,
    ChildFrameBorder,
    TabBar,
    TabLine,
    BasicSentinel,
}
impl From<face_id> for FaceId {
    fn from(t: face_id) -> Self {
        match t {
            face_id::DEFAULT_FACE_ID => FaceId::Default,
            face_id::MODE_LINE_ACTIVE_FACE_ID => FaceId::ModeLineActive,
            face_id::MODE_LINE_INACTIVE_FACE_ID => FaceId::ModeLineInactive,
            face_id::TOOL_BAR_FACE_ID => FaceId::ToolBar,
            face_id::FRINGE_FACE_ID => FaceId::Fringe,
            face_id::HEADER_LINE_FACE_ID => FaceId::HeaderLine,
            face_id::SCROLL_BAR_FACE_ID => FaceId::ScrollBar,
            face_id::BORDER_FACE_ID => FaceId::Border,
            face_id::CURSOR_FACE_ID => FaceId::Cursor,
            face_id::MOUSE_FACE_ID => FaceId::Mouse,
            face_id::MENU_FACE_ID => FaceId::Menu,
            face_id::VERTICAL_BORDER_FACE_ID => FaceId::VerticalBorder,
            face_id::WINDOW_DIVIDER_FACE_ID => FaceId::WindowDivider,
            face_id::WINDOW_DIVIDER_FIRST_PIXEL_FACE_ID => FaceId::WindowDividerFirstPixel,
            face_id::WINDOW_DIVIDER_LAST_PIXEL_FACE_ID => FaceId::WindowDividerLastPixel,
            face_id::INTERNAL_BORDER_FACE_ID => FaceId::InternalBorder,
            face_id::CHILD_FRAME_BORDER_FACE_ID => FaceId::ChildFrameBorder,
            face_id::TAB_BAR_FACE_ID => FaceId::TabBar,
            face_id::TAB_LINE_FACE_ID => FaceId::TabLine,
            face_id::BASIC_FACE_ID_SENTINEL => FaceId::BasicSentinel,
        }
    }
}

impl Into<face_id> for FaceId {
    fn into(self) -> face_id {
        match self {
            FaceId::Default => face_id::DEFAULT_FACE_ID,
            FaceId::ModeLineActive => face_id::MODE_LINE_ACTIVE_FACE_ID,
            FaceId::ModeLineInactive => face_id::MODE_LINE_INACTIVE_FACE_ID,
            FaceId::ToolBar => face_id::TOOL_BAR_FACE_ID,
            FaceId::Fringe => face_id::FRINGE_FACE_ID,
            FaceId::HeaderLine => face_id::HEADER_LINE_FACE_ID,
            FaceId::ScrollBar => face_id::SCROLL_BAR_FACE_ID,
            FaceId::Border => face_id::BORDER_FACE_ID,
            FaceId::Cursor => face_id::CURSOR_FACE_ID,
            FaceId::Mouse => face_id::MOUSE_FACE_ID,
            FaceId::Menu => face_id::MENU_FACE_ID,
            FaceId::VerticalBorder => face_id::VERTICAL_BORDER_FACE_ID,
            FaceId::WindowDivider => face_id::WINDOW_DIVIDER_FACE_ID,
            FaceId::WindowDividerFirstPixel => face_id::WINDOW_DIVIDER_FIRST_PIXEL_FACE_ID,
            FaceId::WindowDividerLastPixel => face_id::WINDOW_DIVIDER_LAST_PIXEL_FACE_ID,
            FaceId::InternalBorder => face_id::INTERNAL_BORDER_FACE_ID,
            FaceId::ChildFrameBorder => face_id::CHILD_FRAME_BORDER_FACE_ID,
            FaceId::TabBar => face_id::TAB_BAR_FACE_ID,
            FaceId::TabLine => face_id::TAB_LINE_FACE_ID,
            FaceId::BasicSentinel => face_id::BASIC_FACE_ID_SENTINEL,
        }
    }
}

pub enum GlyphRowArea {
    Any,
    LeftMargin,
    Text,
    RightMargin,
    Last,
}

impl From<glyph_row_area::Type> for GlyphRowArea {
    fn from(t: glyph_row_area::Type) -> Self {
        match t {
            glyph_row_area::ANY_AREA => GlyphRowArea::Any,
            glyph_row_area::LEFT_MARGIN_AREA => GlyphRowArea::LeftMargin,
            glyph_row_area::TEXT_AREA => GlyphRowArea::Text,
            glyph_row_area::RIGHT_MARGIN_AREA => GlyphRowArea::RightMargin,
            glyph_row_area::LAST_AREA => GlyphRowArea::Last,
            _ => panic!("unsupported glyph_row_area"),
        }
    }
}

impl Into<glyph_row_area::Type> for GlyphRowArea {
    fn into(self) -> glyph_row_area::Type {
        match self {
            GlyphRowArea::Any => glyph_row_area::ANY_AREA,
            GlyphRowArea::LeftMargin => glyph_row_area::LEFT_MARGIN_AREA,
            GlyphRowArea::Text => glyph_row_area::TEXT_AREA,
            GlyphRowArea::RightMargin => glyph_row_area::RIGHT_MARGIN_AREA,
            GlyphRowArea::Last => glyph_row_area::LAST_AREA,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum FaceUnderlineType {
    None,
    Line,
    Double,
    Wave,
    Dotted,
    Dashed,
}

impl From<face_underline_type::Type> for FaceUnderlineType {
    fn from(t: face_underline_type::Type) -> Self {
        match t {
            face_underline_type::FACE_NO_UNDERLINE => FaceUnderlineType::None,
            face_underline_type::FACE_UNDERLINE_SINGLE => FaceUnderlineType::Line,
            face_underline_type::FACE_UNDERLINE_DOUBLE_LINE => FaceUnderlineType::Double,
            face_underline_type::FACE_UNDERLINE_DOTS => FaceUnderlineType::Dotted,
            face_underline_type::FACE_UNDERLINE_DASHES => FaceUnderlineType::Dashed,
            face_underline_type::FACE_UNDERLINE_WAVE => FaceUnderlineType::Wave,
            _ => FaceUnderlineType::None,
        }
    }
}

impl Into<face_underline_type::Type> for FaceUnderlineType {
    fn into(self) -> face_underline_type::Type {
        match self {
            FaceUnderlineType::Line => face_underline_type::FACE_UNDERLINE_SINGLE,
            FaceUnderlineType::Double => face_underline_type::FACE_UNDERLINE_DOUBLE_LINE,
            FaceUnderlineType::Dotted => face_underline_type::FACE_UNDERLINE_DOTS,
            FaceUnderlineType::Dashed => face_underline_type::FACE_UNDERLINE_DASHES,
            FaceUnderlineType::Wave => face_underline_type::FACE_UNDERLINE_WAVE,
            FaceUnderlineType::None => face_underline_type::FACE_NO_UNDERLINE,
        }
    }
}

#[cfg(use_webrender)]
impl Into<Option<LineStyle>> for FaceUnderlineType {
    fn into(self) -> Option<LineStyle> {
        match self {
            FaceUnderlineType::Line => Some(LineStyle::Solid),
            FaceUnderlineType::Dotted => Some(LineStyle::Dotted),
            FaceUnderlineType::Dashed => Some(LineStyle::Dashed),
            FaceUnderlineType::Wave => Some(LineStyle::Wavy),
            _ => None,
        }
    }
}

pub enum ResourceType {
    Number,
    Float,
    Boolean,
    String,
    Symbol,
    BoolNum,
}

impl From<resource_types::Type> for ResourceType {
    fn from(t: resource_types::Type) -> Self {
        match t {
            resource_types::RES_TYPE_NUMBER => ResourceType::Number,
            resource_types::RES_TYPE_FLOAT => ResourceType::Float,
            resource_types::RES_TYPE_BOOLEAN => ResourceType::Boolean,
            resource_types::RES_TYPE_STRING => ResourceType::String,
            resource_types::RES_TYPE_SYMBOL => ResourceType::Symbol,
            resource_types::RES_TYPE_BOOLEAN_NUMBER => ResourceType::BoolNum,
            _ => panic!("unsupported resource type"),
        }
    }
}

impl Into<resource_types::Type> for ResourceType {
    fn into(self) -> resource_types::Type {
        match self {
            ResourceType::Number => resource_types::RES_TYPE_NUMBER,
            ResourceType::Float => resource_types::RES_TYPE_FLOAT,
            ResourceType::Boolean => resource_types::RES_TYPE_BOOLEAN,
            ResourceType::String => resource_types::RES_TYPE_STRING,
            ResourceType::Symbol => resource_types::RES_TYPE_SYMBOL,
            ResourceType::BoolNum => resource_types::RES_TYPE_BOOLEAN_NUMBER,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum FrameParam {
    Alpha,
    AlphaBackground,
    AutoLower,
    AutoRaise,
    BackgroundColor,
    BorderColor,
    BorderWidth,
    BottomDividerWidth,
    BufferPredicate,
    ChildFrameBorderWidth,
    CursorColor,
    CursorType,
    Font,
    FontBackend,
    ForegroundColor,
    Fullscreen,
    HorizontalScrollBars,
    IconName,
    IconType,
    InhibitDoubleBuffering,
    InternalBorderWidth,
    LeftFringe,
    LineSpacing,
    MenuBarLines,
    Minibuffer,
    MouseColor,
    MinWidth,
    MinHeight,
    Width,
    Height,
    Name,
    NoAcceptFocus,
    NoFocusOnMap,
    NoSpecialGlyphs,
    NsAppearance,
    NsTransparentTitlebar,
    OverrideRedirect,
    ParentFrame,
    ParentId,
    RightDividerWidth,
    RightFringe,
    ScreenGamma,
    ScrollBarBackground,
    ScrollBarForeground,
    ScrollBarHeight,
    Terminal,
    Display,
    ScrollBarWidth,
    Shaded,
    SkipTaskbar,
    Sticky,
    TabBarLines,
    Title,
    ToolBarLines,
    ToolBarPosition,
    Undecorated,
    Unsplittable,
    UseFrameSynchronization,
    VerticalScrollBars,
    Visibility,
    WaitForWm,
    ZGroup,
}

impl FrameParam {
    //  Resources are grouped into named classes.  For instance, the
    // ‘Foreground’ class contains the ‘cursorColor’, ‘foreground’ and
    // ‘pointerColor’ resources (*note Table of Resources::)
    pub fn x_resource(&self) -> (CString, CString) {
        let (name, class) = match self {
            FrameParam::Alpha => ("alpha", "Alpha"),
            FrameParam::AlphaBackground => ("alphaBackground", "AlphaBackground"),
            FrameParam::AutoLower => ("autoRaise", "AutoRaiseLower"),
            FrameParam::AutoRaise => ("autoLower", "AutoRaiseLower"),
            FrameParam::BackgroundColor => ("background", "Background"),
            FrameParam::BorderColor => ("borderColor", "BorderColor"),
            FrameParam::BorderWidth => ("borderWidth", "BorderWidth"),
            FrameParam::BufferPredicate => ("bufferPredicate", "BufferPredicate"),
            FrameParam::ChildFrameBorderWidth => ("childFrameBorderWidth", "childFrameBorderWidth"),
            FrameParam::CursorColor => ("cursorColor", "Foreground"),
            FrameParam::CursorType => ("cursorType", "CursorType"),
            FrameParam::Font => ("font", "Font"),
            FrameParam::FontBackend => ("fontBackend", "FontBackend"),
            FrameParam::ForegroundColor => ("foreground", "Foreground"),
            FrameParam::Fullscreen => ("fullscreen", "Fullscreen"),
            FrameParam::HorizontalScrollBars => ("horizontalScrollBars", "ScrollBars"),
            FrameParam::IconName => ("iconName", "Title"),
            FrameParam::IconType => ("bitmapIcon", "BitmapIcon"),
            FrameParam::InhibitDoubleBuffering => {
                ("inhibitDoubleBuffering", "InhibitDoubleBuffering")
            }
            FrameParam::InternalBorderWidth => ("internalBorderWidth", "internalBorderWidth"),
            FrameParam::LeftFringe => ("leftFringe", "LeftFringe"),
            FrameParam::LineSpacing => ("lineSpacing", "LineSpacing"),
            FrameParam::MouseColor => ("pointerColor", "Foreground"),
            FrameParam::Minibuffer => ("minibuffer", "Minibuffer"),
            FrameParam::Name => ("name", "Name"),
            FrameParam::RightFringe => ("rightFringe", "RightFringe"),
            FrameParam::ScreenGamma => ("screenGamma", "ScreenGamma"),
            FrameParam::ScrollBarBackground => ("scrollBarBackground", "ScrollBarBackground"),
            FrameParam::ScrollBarForeground => ("scrollBarForeground", "ScrollBarForeground"),
            FrameParam::ScrollBarHeight => ("scrollBarHeight", "ScrollBarHeight"),
            FrameParam::ScrollBarWidth => ("scrollBarWidth", "ScrollBarWidth"),
            FrameParam::Title => ("title", "Title"),
            FrameParam::VerticalScrollBars => ("verticalScrollBars", "ScrollBars"),
            FrameParam::WaitForWm => ("waitForWM", "WaitForWM"),
            _ => ("", ""),
        };
        (CString::new(name).unwrap(), CString::new(class).unwrap())
    }

    pub fn resource_type(&self) -> ResourceType {
        match self {
            FrameParam::Alpha
            | FrameParam::AlphaBackground
            | FrameParam::BorderWidth
            | FrameParam::BottomDividerWidth
            | FrameParam::ChildFrameBorderWidth
            | FrameParam::InternalBorderWidth
            | FrameParam::LeftFringe
            | FrameParam::LineSpacing
            | FrameParam::MenuBarLines
            | FrameParam::MinWidth
            | FrameParam::MinHeight
            | FrameParam::Width
            | FrameParam::Height
            | FrameParam::RightDividerWidth
            | FrameParam::RightFringe
            | FrameParam::ScrollBarHeight
            | FrameParam::ScrollBarWidth
            | FrameParam::TabBarLines
            | FrameParam::ParentId
            | FrameParam::Terminal
            | FrameParam::ToolBarLines => ResourceType::Number,

            FrameParam::AutoLower
            | FrameParam::AutoRaise
            | FrameParam::IconType
            | FrameParam::InhibitDoubleBuffering
            | FrameParam::NoAcceptFocus
            | FrameParam::NoFocusOnMap
            | FrameParam::NoSpecialGlyphs
            | FrameParam::NsTransparentTitlebar
            | FrameParam::OverrideRedirect
            | FrameParam::Shaded
            | FrameParam::SkipTaskbar
            | FrameParam::Sticky
            | FrameParam::Undecorated
            | FrameParam::Unsplittable
            | FrameParam::UseFrameSynchronization
            | FrameParam::WaitForWm => ResourceType::Boolean,

            FrameParam::BackgroundColor
            | FrameParam::BorderColor
            | FrameParam::CursorColor
            | FrameParam::Font
            | FrameParam::FontBackend
            | FrameParam::ForegroundColor
            | FrameParam::IconName
            | FrameParam::MouseColor
            | FrameParam::Name
            | FrameParam::ScrollBarBackground
            | FrameParam::ScrollBarForeground
            | FrameParam::Display
            | FrameParam::Title => ResourceType::String,

            FrameParam::BufferPredicate
            | FrameParam::CursorType
            | FrameParam::Fullscreen
            | FrameParam::HorizontalScrollBars
            | FrameParam::NsAppearance
            | FrameParam::ParentFrame
            | FrameParam::ToolBarPosition
            | FrameParam::VerticalScrollBars
            | FrameParam::Visibility
            | FrameParam::Minibuffer
            | FrameParam::ZGroup => ResourceType::Symbol,

            FrameParam::ScreenGamma => ResourceType::Float,
        }
    }
}

impl From<LispObject> for FrameParam {
    fn from(param: LispObject) -> FrameParam {
        match param {
            Qalpha => FrameParam::Alpha,
            Qalpha_background => FrameParam::AlphaBackground,
            Qauto_lower => FrameParam::AutoLower,
            Qauto_raise => FrameParam::AutoRaise,
            Qbackground_color => FrameParam::BackgroundColor,
            Qborder_color => FrameParam::BorderColor,
            Qborder_width => FrameParam::BorderWidth,
            Qbottom_divider_width => FrameParam::BottomDividerWidth,
            Qbuffer_predicate => FrameParam::BufferPredicate,
            Qchild_frame_border_width => FrameParam::ChildFrameBorderWidth,
            Qcursor_color => FrameParam::CursorColor,
            Qcursor_type => FrameParam::CursorType,
            Qfont => FrameParam::Font,
            Qfont_backend => FrameParam::FontBackend,
            Qforeground_color => FrameParam::ForegroundColor,
            Qfullscreen => FrameParam::Fullscreen,
            Qhorizontal_scroll_bars => FrameParam::HorizontalScrollBars,
            Qicon_name => FrameParam::IconName,
            Qicon_type => FrameParam::IconType,
            Qinhibit_double_buffering => FrameParam::InhibitDoubleBuffering,
            Qinternal_border_width => FrameParam::InternalBorderWidth,
            Qleft_fringe => FrameParam::LeftFringe,
            Qline_spacing => FrameParam::LineSpacing,
            Qmenu_bar_lines => FrameParam::MenuBarLines,
            Qmouse_color => FrameParam::MouseColor,
            Qmin_width => FrameParam::MinWidth,
            Qmin_height => FrameParam::MinHeight,
            Qwidth => FrameParam::Width,
            Qheight => FrameParam::Height,
            Qname => FrameParam::Name,
            Qno_accept_focus => FrameParam::NoAcceptFocus,
            Qno_focus_on_map => FrameParam::NoFocusOnMap,
            Qno_special_glyphs => FrameParam::NoSpecialGlyphs,
            Qns_appearance => FrameParam::NsAppearance,
            Qns_transparent_titlebar => FrameParam::NsTransparentTitlebar,
            Qoverride_redirect => FrameParam::OverrideRedirect,
            Qparent_frame => FrameParam::ParentFrame,
            Qparent_id => FrameParam::ParentId,
            Qright_divider_width => FrameParam::RightDividerWidth,
            Qright_fringe => FrameParam::RightFringe,
            Qscreen_gamma => FrameParam::ScreenGamma,
            Qminibuffer => FrameParam::Minibuffer,
            Qscroll_bar_background => FrameParam::ScrollBarBackground,
            Qscroll_bar_foreground => FrameParam::ScrollBarForeground,
            Qscroll_bar_height => FrameParam::ScrollBarHeight,
            Qscroll_bar_width => FrameParam::ScrollBarWidth,
            Qshaded => FrameParam::Shaded,
            Qskip_taskbar => FrameParam::SkipTaskbar,
            Qsticky => FrameParam::Sticky,
            Qtab_bar_lines => FrameParam::TabBarLines,
            Qtitle => FrameParam::Title,
            Qtool_bar_lines => FrameParam::ToolBarLines,
            Qterminal => FrameParam::Terminal,
            Qdisplay => FrameParam::Display,
            Qtool_bar_position => FrameParam::ToolBarPosition,
            Qundecorated => FrameParam::Undecorated,
            Qunsplittable => FrameParam::Unsplittable,
            Quse_frame_synchronization => FrameParam::UseFrameSynchronization,
            Qvertical_scroll_bars => FrameParam::VerticalScrollBars,
            Qvisibility => FrameParam::Visibility,
            Qwait_for_wm => FrameParam::WaitForWm,
            Qz_group => FrameParam::ZGroup,
            _ => panic!("unknow frame param {param:?}"),
        }
    }
}

impl Into<LispObject> for FrameParam {
    fn into(self) -> LispObject {
        match self {
            FrameParam::Alpha => Qalpha,
            FrameParam::AlphaBackground => Qalpha_background,
            FrameParam::AutoLower => Qauto_lower,
            FrameParam::AutoRaise => Qauto_raise,
            FrameParam::BackgroundColor => Qbackground_color,
            FrameParam::BorderColor => Qborder_color,
            FrameParam::BorderWidth => Qborder_width,
            FrameParam::BottomDividerWidth => Qbottom_divider_width,
            FrameParam::BufferPredicate => Qbuffer_predicate,
            FrameParam::ChildFrameBorderWidth => Qchild_frame_border_width,
            FrameParam::CursorColor => Qcursor_color,
            FrameParam::CursorType => Qcursor_type,
            FrameParam::Font => Qfont,
            FrameParam::FontBackend => Qfont_backend,
            FrameParam::ForegroundColor => Qforeground_color,
            FrameParam::Fullscreen => Qfullscreen,
            FrameParam::HorizontalScrollBars => Qhorizontal_scroll_bars,
            FrameParam::IconName => Qicon_name,
            FrameParam::IconType => Qicon_type,
            FrameParam::InhibitDoubleBuffering => Qinhibit_double_buffering,
            FrameParam::InternalBorderWidth => Qinternal_border_width,
            FrameParam::LeftFringe => Qleft_fringe,
            FrameParam::LineSpacing => Qline_spacing,
            FrameParam::MenuBarLines => Qmenu_bar_lines,
            FrameParam::MouseColor => Qmouse_color,
            FrameParam::Minibuffer => Qminibuffer,
            FrameParam::MinWidth => Qmin_width,
            FrameParam::MinHeight => Qmin_height,
            FrameParam::Width => Qwidth,
            FrameParam::Height => Qheight,
            FrameParam::Name => Qname,
            FrameParam::NoAcceptFocus => Qno_accept_focus,
            FrameParam::NoFocusOnMap => Qno_focus_on_map,
            FrameParam::NoSpecialGlyphs => Qno_special_glyphs,
            FrameParam::NsAppearance => Qns_appearance,
            FrameParam::NsTransparentTitlebar => Qns_transparent_titlebar,
            FrameParam::OverrideRedirect => Qoverride_redirect,
            FrameParam::ParentFrame => Qparent_frame,
            FrameParam::ParentId => Qparent_id,
            FrameParam::RightDividerWidth => Qright_divider_width,
            FrameParam::RightFringe => Qright_fringe,
            FrameParam::ScreenGamma => Qscreen_gamma,
            FrameParam::ScrollBarBackground => Qscroll_bar_background,
            FrameParam::ScrollBarForeground => Qscroll_bar_foreground,
            FrameParam::ScrollBarHeight => Qscroll_bar_height,
            FrameParam::ScrollBarWidth => Qscroll_bar_width,
            FrameParam::Shaded => Qshaded,
            FrameParam::SkipTaskbar => Qskip_taskbar,
            FrameParam::Sticky => Qsticky,
            FrameParam::TabBarLines => Qtab_bar_lines,
            FrameParam::Title => Qtitle,
            FrameParam::ToolBarLines => Qtool_bar_lines,
            FrameParam::Terminal => Qterminal,
            FrameParam::Display => Qdisplay,
            FrameParam::ToolBarPosition => Qtool_bar_position,
            FrameParam::Undecorated => Qundecorated,
            FrameParam::Unsplittable => Qunsplittable,
            FrameParam::UseFrameSynchronization => Quse_frame_synchronization,
            FrameParam::VerticalScrollBars => Qvertical_scroll_bars,
            FrameParam::Visibility => Qvisibility,
            FrameParam::WaitForWm => Qwait_for_wm,
            FrameParam::ZGroup => Qz_group,
        }
    }
}
