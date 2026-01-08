use glyphon::fontdb;

/// Stores a handle to cached text which can be repositioned
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextHandle(pub(crate) usize);

/// A simplified font face info struct
#[derive(Clone, Debug, Default)]
pub struct FontHandle {
    /// Unique font face ID from fontdb
    pub id: fontdb::ID,

    /// Primary family name (English US or first available)
    pub family_name: String,

    /// PostScript name for precise font matching
    pub postscript_name: String,

    /// Font style (normal, italic, oblique)
    pub style: fontdb::Style,

    /// Font weight (100-900)
    pub weight: fontdb::Weight,

    /// Font stretch (condensed, normal, expanded)
    pub stretch: fontdb::Stretch,

    /// Whether the font is monospaced
    pub is_monospaced: bool,

    /// Face index in the source (for font collections)
    pub face_index: u32,
}

impl From<&fontdb::FaceInfo> for FontHandle {
    fn from(face: &fontdb::FaceInfo) -> Self {
        Self {
            id: face.id,
            family_name: face
                .families
                .first()
                .map(|(name, _)| name.clone())
                .unwrap_or_default(),
            postscript_name: face.post_script_name.clone(),
            style: face.style,
            weight: face.weight,
            stretch: face.stretch,
            is_monospaced: face.monospaced,
            face_index: face.index,
        }
    }
}

impl FontHandle {
    /// Get a display-friendly description of the font
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.family_name, self.postscript_name)
    }

    /// Check if this font matches a specific family name
    pub fn matches_family(&self, name: &str) -> bool {
        self.family_name.eq_ignore_ascii_case(name)
    }

    /// Check if this is a bold font (weight >= 700)
    pub fn is_bold(&self) -> bool {
        self.weight.0 >= 700
    }

    /// Check if this is an italic or oblique font
    pub fn is_italic(&self) -> bool {
        matches!(self.style, fontdb::Style::Italic | fontdb::Style::Oblique)
    }
}

pub(crate) enum BufferRef {
    Cached(TextHandle),
    Immediate(usize),
}

// Stored in cached_buffers - persists across frames
pub(crate) struct CachedTextEntry {
    pub(crate) buffer: glyphon::Buffer,
    pub(crate) size: f32, // Original size, for re-shaping if text content changes
}

// Stored in queued_renders - what to render this frame, cleared each frame
// Owned version of a TextArea to avoid lifetime issues prior to rendering
pub(crate) struct QueuedText {
    pub(crate) buffer_ref: BufferRef, // Which buffer to use
    pub(crate) position: glam::Vec2,  // Where on screen
    pub(crate) color: glyphon::Color, // What color
    pub(crate) scale: f32,            // Size multiplier
    pub(crate) bounds: Option<glyphon::TextBounds>, // Clipping
}
