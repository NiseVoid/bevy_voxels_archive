use crate::{RawChunk, Voxel, CHUNK_SIZE, CHUNK_VOXELS};

use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// The resource that stores the entity of every existing chunk, indexed by chunk position
#[derive(Resource, Deref, DerefMut)]
pub struct ChunkMap(HashMap<ChunkPosition, Entity>);

impl Default for ChunkMap {
    fn default() -> Self {
        Self(HashMap::with_capacity(500))
    }
}

/// The position of a chunk, the bounds of valid chunks are the same as the limits of the i8 type
#[derive(
    Component,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Deref,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct ChunkPosition(pub(crate) [i8; 3]);

impl std::ops::Add<[i8; 3]> for ChunkPosition {
    type Output = Self;

    fn add(self, rhs: [i8; 3]) -> Self::Output {
        Self([
            self.0[0] + rhs[0],
            self.0[1] + rhs[1],
            self.0[2] + rhs[2],
        ])
    }
}

impl From<[i8; 3]> for ChunkPosition {
    fn from(value: [i8; 3]) -> Self {
        Self(value)
    }
}

impl ChunkPosition {
    /// Construct a ChunkPosition from the x, y and z coordinates
    pub fn new(x: i8, y: i8, z: i8) -> Self {
        Self([x, y, z])
    }

    /// Get the desired Transform translation for this chunk
    pub fn get_translation(&self) -> Vec3 {
        Vec3::new(
            self.0[0] as f32 * CHUNK_SIZE,
            self.0[1] as f32 * CHUNK_SIZE,
            self.0[2] as f32 * CHUNK_SIZE,
        )
    }

    /// Get the ChunkPos for this translation
    pub fn from_translation(pos: &Vec3) -> Option<Self> {
        let range = (i8::MIN as f32 * CHUNK_SIZE)..(i8::MAX as f32 * CHUNK_SIZE);
        if !range.contains(&pos.x) || !range.contains(&pos.y) || !range.contains(&pos.z) {
            return None;
        }
        Some(Self([
            (pos.x / CHUNK_SIZE).round() as i8,
            (pos.y / CHUNK_SIZE).round() as i8,
            (pos.z / CHUNK_SIZE).round() as i8,
        ]))
    }

    /// Create a ChunkPosition from big endian bytes
    pub fn from_be_bytes(bytes: [u8; 3]) -> Self {
        Self([
            i8::from_be_bytes([bytes[0]]),
            i8::from_be_bytes([bytes[1]]),
            i8::from_be_bytes([bytes[2]]),
        ])
    }

    /// Encode a ChunkPosition to big endian bytes
    pub fn to_be_bytes(&self) -> [u8; 3] {
        [
            self.0[0].to_be_bytes()[0],
            self.0[1].to_be_bytes()[0],
            self.0[2].to_be_bytes()[0],
        ]
    }
}

/// ChunkData stores data for a chunk with Run Lenght Encoding compression.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct ChunkData(SmallVec<[u16; 3]>);

impl From<RawChunk> for ChunkData {
    fn from(value: RawChunk) -> Self {
        Self::from(&value)
    }
}

impl From<&RawChunk> for ChunkData {
    fn from(value: &RawChunk) -> Self {
        let mut buf = SmallVec::new();
        let mut last = Voxel::AIR;
        let mut count = 0u16;
        for (k, v) in value.0.iter().enumerate() {
            if k != 0 && last == v.raw() {
                count += 1;
                continue;
            }
            if count > 1 {
                buf.push(last.raw());
                buf.push(count);
            };
            buf.push(v.raw());
            count = 1;
            last = *v;
        }
        if count > 1 {
            buf.push(last.raw());
            buf.push(count);
        };

        Self(buf)
    }
}

impl ChunkData {
    /// The number of bytes the chunk takes up. Since every value is a u16, this is the length * 2
    pub fn n_bytes(&self) -> usize {
        self.0.len() * 2
    }

    /// Create chunk data for a chunk that only has empty air voxels
    pub fn air() -> Self {
        Self(SmallVec::from_slice(&[
            Voxel::AIR.raw(),
            Voxel::AIR.raw(),
            CHUNK_VOXELS as u16,
        ]))
    }

    /// Expand the ChunkData to a RawChunk, which can then be used to create a chunk mesh or
    /// modify the chunk
    pub fn expand(&self) -> RawChunk {
        let mut buf = RawChunk::empty();

        let len = self.0.len();
        let mut k = 0;
        while k < len {
            let v = self.0[k];
            if k + 2 < len {
                let peek = self.0[k + 1];
                if peek == v {
                    let n = self.0[k + 2] as usize;
                    buf.0.resize(buf.0.len() + n, Voxel::from_raw(v));
                    k += 3;
                    continue;
                }
            }

            buf.0.push(Voxel::from_raw(v));
            k += 1;
        }

        buf
    }
}

#[test]
fn test_rle() {
    let mut input = Vec::with_capacity(20);
    input.extend_from_slice(&[Voxel::new(0, 12); 10]);
    input.push(Voxel::new(0, 0));
    input.extend_from_slice(&[Voxel::new(0, 29); 8]);
    input.push(Voxel::new(0, 1));

    let output = ChunkData::from(RawChunk(input));
    assert_eq!(&output.0.as_slice(), &[12, 12, 10, 0, 29, 29, 8, 1]);
}

#[test]
fn test_rle_all_air_fits_in_smallvec() {
    let mut input = Vec::with_capacity(1024);
    input.extend_from_slice(&[Voxel::AIR; 1024]);

    let output = ChunkData::from(RawChunk(input));
    assert_eq!(output.0.len(), 3);
    assert_eq!(output.0.as_slice(), &[Voxel::AIR.raw(), Voxel::AIR.raw(), 1024]);
}

#[test]
fn test_rle_expand() {
    let mut rle = ChunkData(SmallVec::new());
    rle.0.extend_from_slice(&[1, 1, 2, 3, 3, 4, 5]);

    let output = rle.expand();
    assert_eq!(output.0.as_slice(), &[1, 1, 3, 3, 3, 3, 5]);
    assert_eq!(output.0.capacity(), 8000);
}
