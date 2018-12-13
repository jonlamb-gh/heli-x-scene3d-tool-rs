#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TerrainMode {
    /// Colored points and lines
    Wireframe,
    /// Colored points
    Points,
    /// Solid, colored material
    Solid,
    /// Solid, color provided by the heightmap image itself
    Textured,
    /// Solid, color provided by the alpha map channels
    Alphamap,
    /* TODO - need to load and blend the alphamap channels
     * Filled, color provided by the color channels in the alpha map texture
     * file
     * Channels? */
}

impl TerrainMode {
    pub fn next(&self) -> Self {
        match *self {
            TerrainMode::Wireframe => TerrainMode::Points,
            TerrainMode::Points => TerrainMode::Solid,
            TerrainMode::Solid => TerrainMode::Textured,
            TerrainMode::Textured => TerrainMode::Alphamap,
            TerrainMode::Alphamap => TerrainMode::Wireframe,
        }
    }
}
