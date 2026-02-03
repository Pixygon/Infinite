/// A loaded mesh asset (renderer-agnostic). Contains raw vertex data extracted
/// from a glTF file.
#[derive(Debug, Clone)]
pub struct MeshAsset {
    pub name: String,
    pub primitives: Vec<MeshPrimitive>,
}

/// A single draw primitive within a mesh.
#[derive(Debug, Clone)]
pub struct MeshPrimitive {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tex_coords: Option<Vec<[f32; 2]>>,
    pub colors: Option<Vec<[f32; 4]>>,
    pub indices: Option<Vec<u32>>,
}
