#[derive(Debug)]
pub enum EngineError {
    ShaderCompileError(String),
    ShaderLinkError(String),
    TextureLoadError(String),
    FramebufferIncomplete(String),
    ResourceCreationError(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::ShaderCompileError(msg) => write!(f, "Shader Compile Error: {}", msg),
            EngineError::ShaderLinkError(msg) => write!(f, "Shader Link Error: {}", msg),
            EngineError::TextureLoadError(msg) => write!(f, "Texture Load Error: {}", msg),
            EngineError::FramebufferIncomplete(msg) => write!(f, "Framebuffer Incomplete: {}", msg),
            EngineError::ResourceCreationError(msg) => {
                write!(f, "Resource Creation Error: {}", msg)
            }
        }
    }
}

impl std::error::Error for EngineError {}
