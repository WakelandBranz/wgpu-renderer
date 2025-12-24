/// Stores a handle to cached text which can be repositioned
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextHandle(pub(crate) usize);

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
    pub(crate) buffer_ref: BufferRef,               // Which buffer to use
    pub(crate) position: glam::Vec2,                // Where on screen
    pub(crate) color: glyphon::Color,               // What color
    pub(crate) scale: f32,                          // Size multiplier
    pub(crate) bounds: Option<glyphon::TextBounds>, // Clipping
}
