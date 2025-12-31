use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextError {
    // === Initialization Errors ===
    #[error("Failed to find text buffer within cache")]
    NoBufferFound,

    #[error("Text prepare error: {0}")]
    TextPrepare(#[from] glyphon::PrepareError),

    #[error("Text render error: {0}")]
    TextRender(#[from] glyphon::RenderError),

    #[error("Failed to load font: {0}")]
    FontLoad(std::io::Error),
    
    #[error("Failed to load font: no valid font faces found")]
    FontNotFound,
}
