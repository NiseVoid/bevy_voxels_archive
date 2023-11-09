//! This module is responsbile for creating a smooth mesh for a chunk
//! It uses the fast_surface_nets crate to generate meshes

use crate::{
    ChunkData, ChunkMap, ChunkPosition, RawChunk, Voxel, CHUNK_BOUNDS, CHUNK_SIDES, CHUNK_SIZE,
    VOXEL_SIZE,
};

use bevy::prelude::{Deref, DerefMut, Query};
pub use fast_surface_nets::SurfaceNetsBuffer;
use fast_surface_nets::{
    ndshape::{ConstShape3u32, ConstShape3u8, Shape},
    surface_nets,
};

/// Data about surrounding chunks of data
#[derive(Default)]
pub struct SurroundingChunks([Option<RawChunk>; 3 * 3 * 3]);

impl SurroundingChunks {
    const SHAPE: ConstShape3u8<3, 3, 3> = ConstShape3u8::<3, 3, 3>;
    const LAST_CHUNK: i32 = CHUNK_SIDES as i32 + 1;

    fn clear(&mut self) {
        for chunk in &mut self.0 {
            *chunk = None;
        }
    }

    /// Get the voxel at the specified coordinates
    pub fn get_voxel(&self, xyz: [i32; 3]) -> Voxel {
        let mut iter = xyz.iter().map(|v| {
            if *v <= 0 {
                0
            } else if *v >= Self::LAST_CHUNK {
                2
            } else {
                1
            }
        });
        let chunk_pos: [u8; 3] = [
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ];
        let chunk_idx = Self::SHAPE.linearize(chunk_pos);

        let Some(ref chunk) = self.0[chunk_idx as usize] else {return Voxel::AIR;};

        let mut iter = xyz.iter().map(|v| {
            (if *v <= 0 {
                CHUNK_SIDES as i32 - 1 + v
            } else if *v >= Self::LAST_CHUNK {
                *v - Self::LAST_CHUNK
            } else {
                v - 1
            }) as u32
        });

        chunk.get_voxel(iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap())
    }
}

/// Grid holds the data to used to generate a chunk mesh
#[derive(Deref, DerefMut)]
pub struct Grid(Vec<Voxel>);

impl Grid {
    const BOUNDS: u32 = CHUNK_BOUNDS + 2;
    /// The shape of the grid
    pub const SHAPE: ConstShape3u32<{ Self::BOUNDS }, { Self::BOUNDS }, { Self::BOUNDS }> =
        ConstShape3u32::<{ Self::BOUNDS }, { Self::BOUNDS }, { Self::BOUNDS }>;
}

impl Default for Grid {
    fn default() -> Self {
        Self(Vec::with_capacity(Self::SHAPE.usize()))
    }
}

/// Generate the mesh for a chunk, which is returned as a Vec of vertices and a Vec of indices
/// This function queries and expands the necessary chunk data itself and just needs the chunk map
/// and position of the chunk that needs a mesh
pub fn generate_chunk(
    buffer: &mut SurfaceNetsBuffer,
    data: &mut SurroundingChunks,
    grid: &mut Grid,
    chunk_pos: ChunkPosition,
    chunk_map: &ChunkMap,
    query: &Query<&ChunkData>,
) {
    data.clear();
    grid.clear();

    for i in 0..SurroundingChunks::SHAPE.usize() {
        let [x, y, z] = SurroundingChunks::SHAPE.delinearize(i as u8);
        let desired_pos = chunk_pos + [-1 + x as i8, -1 + y as i8, -1 + z as i8];
        let Some(chunk_entity) = chunk_map.get(&desired_pos) else {continue;};
        let Ok(chunk) = query.get(*chunk_entity) else {continue;};
        data.0[i] = Some(chunk.expand());
    }

    for i in 0..Grid::SHAPE.usize() {
        let xyz = Grid::SHAPE.delinearize(i as u32);
        grid.push(data.get_voxel([
            xyz[0] as i32,
            xyz[1] as i32,
            xyz[2] as i32,
        ]));
    }

    surface_nets(grid.as_slice(), &Grid::SHAPE, [0; 3], [(CHUNK_SIDES + 1) as u32; 3], buffer);
    for pos in buffer.positions.iter_mut() {
        pos[0] = pos[0] * VOXEL_SIZE - CHUNK_SIZE / 2.;
        pos[1] = pos[1] * VOXEL_SIZE - CHUNK_SIZE / 2.;
        pos[2] = pos[2] * VOXEL_SIZE - CHUNK_SIZE / 2.;
    }
}
