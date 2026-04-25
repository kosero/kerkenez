#[derive(Debug)]
pub enum KerkenezError {
    ShaderCompileError(String),
    ShaderLinkError(String),
    TextureLoadError(String),
    FramebufferIncomplete(String),
    ResourceCreationError(String),
}

impl std::fmt::Display for KerkenezError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KerkenezError::ShaderCompileError(msg) => write!(f, "Shader Compile Error: {}", msg),
            KerkenezError::ShaderLinkError(msg) => write!(f, "Shader Link Error: {}", msg),
            KerkenezError::TextureLoadError(msg) => write!(f, "Texture Load Error: {}", msg),
            KerkenezError::FramebufferIncomplete(msg) => {
                write!(f, "Framebuffer Incomplet: {}", msg)
            }
            KerkenezError::ResourceCreationError(msg) => {
                write!(f, "Resource Creation Error: {}", msg)
            }
        }
    }
}

impl std::error::Error for KerkenezError {}
