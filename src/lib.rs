//! This crate holds all shared logic for the voxel system.
//! This voxel system, like most others, works with chunks. Each chunk is stored with
//! Run Length Encoding compression, and only expanded when the actual data is needed
//!
//! Voxels hold two values:
//! - A material, which decides the color and texture it is rendered with
//! - A value, used as a Signed Distance Field to create a smooth mesh

#![warn(missing_docs)]
#![allow(clippy::too_many_arguments)]

mod voxel;
pub use voxel::Voxel;

mod raw;
pub use raw::RawChunk;

pub mod surface_nets;

mod storage;
pub use storage::{ChunkData, ChunkMap, ChunkPosition};

pub mod edit;

use bevy::prelude::*;
use fast_surface_nets::ndshape::ConstShape3u8;
pub use fast_surface_nets::ndshape::{RuntimeShape, Shape};

/// A shape used when fetching and storing the needed chunk and all surrounding chunks
pub const FETCH_SHAPE: ConstShape3u8<3, 3, 3> = ConstShape3u8::<3, 3, 3>;

/// The size of each voxel, in meters.
/// Each side of the voxel has the same size, making is a perfect cube
pub const VOXEL_SIZE: f32 = 0.75;
/// The number of voxels per side of a chunk
pub const CHUNK_SIDES: usize = 20;
pub(crate) const CHUNK_BOUNDS: u32 = CHUNK_SIDES as u32;
/// The size of a chunk, in meters
pub const CHUNK_SIZE: f32 = CHUNK_SIDES as f32 * VOXEL_SIZE;
/// The number of voxels per chunk, since every chunk is a cube of voxels this is
/// just CHUNK_SIDES^3
const CHUNK_VOXELS: usize = CHUNK_SIDES * CHUNK_SIDES * CHUNK_SIDES;
