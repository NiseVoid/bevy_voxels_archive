use fast_surface_nets::ndshape::{ConstShape3u32, Shape};

use crate::{Voxel, CHUNK_BOUNDS, CHUNK_VOXELS};

/// RawChunk is the raw data of a chunk. This is not how chunks are stored, and is only kept in
/// memory while it is being modified or used to create a chunk mesh
pub struct RawChunk(pub(crate) Vec<Voxel>);

pub(crate) const CHUNK_SHAPE: ConstShape3u32<CHUNK_BOUNDS, CHUNK_BOUNDS, CHUNK_BOUNDS> =
    ConstShape3u32::<CHUNK_BOUNDS, CHUNK_BOUNDS, CHUNK_BOUNDS>;

impl RawChunk {
    pub(crate) fn empty() -> Self {
        Self(Vec::with_capacity(CHUNK_VOXELS))
    }

    /// Get a RawChunk made from only empty air voxels
    pub fn air() -> Self {
        Self(vec![Voxel::AIR; CHUNK_VOXELS])
    }

    /// Get the voxel at the specified coordinates
    pub fn get_voxel(&self, x: u32, y: u32, z: u32) -> Voxel {
        let idx = CHUNK_SHAPE.linearize([x, y, z]);
        self.0[idx as usize]
    }

    /// Get a mutable reference to the voxel at the specified coordinates
    pub fn get_mut_voxel(&mut self, x: u32, y: u32, z: u32) -> &mut Voxel {
        let idx = CHUNK_SHAPE.linearize([x, y, z]);
        &mut self.0[idx as usize]
    }

    /// Set the voxel at the specified coordinates to the given Voxel
    pub fn set_voxel(&mut self, x: u32, y: u32, z: u32, voxel: Voxel) {
        let idx = CHUNK_SHAPE.linearize([x, y, z]);
        self.0[idx as usize] = voxel;
    }
}
