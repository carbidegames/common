#[derive(Debug)]
pub enum Error {
    /// The UI already has a root.
    RootAlreadyExists,
    /// The UI doesn't have a root when it's required for the operation.
    NoRoot,
    Resource(ResourceError),
    Rendering(RenderingError),
}

#[derive(Debug)]
pub enum ResourceError {
    InvalidId,
    Other { resource: Option<String>, error: String },
}

#[derive(Debug)]
pub enum RenderingError {
    /// A panel was too large for the renderer. This likely means you've got a Max size on a panel
    /// without anything else constraining its size.
    PanelTooLarge,
    Other(Box<::std::error::Error>)
}
