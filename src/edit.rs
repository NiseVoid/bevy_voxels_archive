//! This module contains logic to edit the voxel grid

use crate::{ChunkData, ChunkMap, ChunkPosition, RawChunk, Voxel, CHUNK_SIDES, VOXEL_SIZE};

use bevy::{prelude::*, utils::HashMap};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

struct ModifiedChunk {
    entity: Option<Entity>,
    index: usize,
}

/// A type to apply modifications to the voxel grid. For optimal performance all ready terrain
/// modifications should be applied with the same ChunkModifier at once
#[derive(Default)]
pub struct ChunkModifier {
    modified: HashMap<ChunkPosition, ModifiedChunk>,
    chunks: Vec<RawChunk>,
}

impl ChunkModifier {
    /// Apply the calculated modifications to the bevy [World] trough [Commands]
    pub fn apply(&self, commands: &mut Commands) {
        for (pos, data) in self.modified.iter() {
            let entity = data.entity;
            let data = &self.chunks[data.index];
            if let Some(entity) = entity {
                commands.entity(entity).insert(ChunkData::from(data));
            } else {
                commands.spawn((*pos, ChunkData::from(data)));
            }
        }
    }

    fn get_voxel(
        &mut self,
        chunk_pos: ChunkPosition,
        chunk_map: &mut ChunkMap,
        mut chunks_getter: impl FnMut(Entity) -> RawChunk,
        relative_x: i32,
        relative_y: i32,
        relative_z: i32,
    ) -> Option<&mut Voxel> {
        const SIZE: i32 = CHUNK_SIDES as i32;
        let mut chunk_pos = IVec3::new(
            chunk_pos[0] as i32,
            chunk_pos[1] as i32,
            chunk_pos[2] as i32,
        );
        let orig_pos = chunk_pos;
        chunk_pos += IVec3::new(
            (relative_x as f32 / SIZE as f32).floor() as i32,
            (relative_y as f32 / SIZE as f32).floor() as i32,
            (relative_z as f32 / SIZE as f32).floor() as i32,
        );

        if chunk_pos.x < i8::MIN as i32
            || chunk_pos.x > i8::MAX as i32
            || chunk_pos.y < i8::MIN as i32
            || chunk_pos.y > i8::MAX as i32
            || chunk_pos.z < i8::MIN as i32
            || chunk_pos.z > i8::MAX as i32
        {
            return None;
        }

        let offset = orig_pos - chunk_pos;
        let chunk_pos = ChunkPosition::new(chunk_pos.x as i8, chunk_pos.y as i8, chunk_pos.z as i8);
        let relative_x = relative_x + offset.x * SIZE;
        let relative_y = relative_y + offset.y * SIZE;
        let relative_z = relative_z + offset.z * SIZE;

        let chunk_data = match self.modified.get_mut(&chunk_pos) {
            Some(chunk) => &mut self.chunks[chunk.index],
            None => {
                let (chunk_entity, chunk_data) = match chunk_map.get(&chunk_pos) {
                    Some(entity) => (Some(*entity), chunks_getter(*entity)),
                    None => (None, RawChunk::air()),
                };
                let index = self.chunks.len();
                self.chunks.push(chunk_data);
                let _ = self.modified.insert_unique_unchecked(
                    chunk_pos,
                    ModifiedChunk {
                        entity: chunk_entity,
                        index,
                    },
                );
                &mut self.chunks[index]
            }
        };

        Some(chunk_data.get_mut_voxel(relative_x as u32, relative_y as u32, relative_z as u32))
    }

    /// Apply a [SignedDistanceFunction] to the voxel grid at the specified position relative to
    /// the given [ChunkPosition]
    pub fn apply_sdf(
        &mut self,
        chunk_pos: ChunkPosition,
        chunk_map: &mut ChunkMap,
        mut chunks_getter: impl FnMut(Entity) -> RawChunk,
        sdf: impl SignedDistanceFunction,
        mode: Mode,
        smoothness: f32,
        relative_pos: Vec3,
    ) {
        let (aabb_min, aabb_max) = sdf.aabb();

        let aabb_min = (aabb_min + relative_pos) / VOXEL_SIZE;
        // let relative_pos = relative_pos + aabb_min.fract() * VOXEL_SIZE;
        let aabb_min = aabb_min.floor();
        let aabb_min = IVec3::new(aabb_min.x as i32, aabb_min.y as i32, aabb_min.z as i32) - 1;

        let aabb_max = ((aabb_max + relative_pos) / VOXEL_SIZE).ceil();
        let aabb_max = IVec3::new(aabb_max.x as i32, aabb_max.y as i32, aabb_max.z as i32) + 1;

        for x in aabb_min.x..aabb_max.x {
            for y in aabb_min.y..aabb_max.y {
                for z in aabb_min.z..aabb_max.z {
                    let Some(voxel) =
                        self.get_voxel(chunk_pos, chunk_map, &mut chunks_getter, x, y, z)
                    else {
                        continue;
                    };
                    let cur_value = f32::from(*voxel);
                    let new_value = sdf
                        .sdf(Vec3::new(
                            // TODO: Figure out a cleaner solution than this offset
                            (x + 1) as f32 * VOXEL_SIZE - relative_pos.x,
                            (y + 1) as f32 * VOXEL_SIZE - relative_pos.y,
                            (z + 1) as f32 * VOXEL_SIZE - relative_pos.z,
                        ))
                        .clamp(-1., 1.);
                    let value = match mode {
                        Mode::Add => smin(cur_value, new_value, smoothness),
                        Mode::Remove => smax(cur_value, -new_value, smoothness),
                    };
                    *voxel = voxel.with_value_f32(value.clamp(-1., 1.));
                }
            }
        }
    }
}

// Polynomial smin from https://iquilezles.org/articles/smin
#[inline(always)]
fn smooth(a: f32, b: f32, k: f32) -> f32 {
    let h = (k - (a - b).abs()).max(0.0);
    h * h * 0.25 / k
}

#[inline(always)]
fn smin(a: f32, b: f32, k: f32) -> f32 {
    a.min(b) - smooth(a, b, k)
}

#[inline(always)]
fn smax(a: f32, b: f32, k: f32) -> f32 {
    a.max(b) + smooth(a, b, k)
}

/// A trait for a signed distance function
#[enum_dispatch]
pub trait SignedDistanceFunction {
    /// Get the sdf value at the provided position, with the shape at 0,0,0
    fn sdf(&self, pos: Vec3) -> f32;
    /// Get the bounding box for the shape
    fn aabb(&self) -> (Vec3, Vec3);
}

/// An enum with SDF variants, used to pass the SignedDistanceFunction trait around without Box or
/// dynamic dispatch
#[derive(Debug)]
#[enum_dispatch(SignedDistanceFunction)]
pub enum Sdf {
    /// A sphere
    Sphere(SphereSdf),
    /// A box
    Box(BoxSdf),
    /// A vertical cylinder
    Cylinder(CylinderSdf),
}

/// A signed distance sphere
#[derive(Debug)]
pub struct SphereSdf(pub f32);

impl SignedDistanceFunction for SphereSdf {
    fn sdf(&self, pos: Vec3) -> f32 {
        pos.length() - self.0
    }

    fn aabb(&self) -> (Vec3, Vec3) {
        (Vec3::splat(-self.0), Vec3::splat(self.0))
    }
}

/// A signed distance box
#[derive(Debug)]
pub struct BoxSdf(pub Vec3);

impl SignedDistanceFunction for BoxSdf {
    fn sdf(&self, pos: Vec3) -> f32 {
        let q = pos.abs() - self.0;
        q.max(Vec3::ZERO).length() + q.x.max(q.y).max(q.z).min(0.0)
    }

    fn aabb(&self) -> (Vec3, Vec3) {
        (-self.0, self.0)
    }
}

/// A signed distance cylinder
#[derive(Debug)]
pub struct CylinderSdf {
    /// The radius of the cylinder
    pub radius: f32,
    /// The height of the cylinder
    pub height: f32,
}

impl SignedDistanceFunction for CylinderSdf {
    fn sdf(&self, pos: Vec3) -> f32 {
        let d = Vec2::new(pos.xz().length(), pos.y).abs() - Vec2::new(self.radius, self.height);
        d.x.max(d.y).min(0.0) + d.max(Vec2::ZERO).length()
    }

    fn aabb(&self) -> (Vec3, Vec3) {
        (
            Vec3::new(-self.radius, -self.height / 2., -self.radius),
            Vec3::new(self.radius, self.height / 2., self.radius),
        )
    }
}

#[test]
fn test_sphere_sdf() {
    let sphere = SphereSdf(5.);
    assert_eq!(-5., sphere.sdf(Vec3::new(0., 0., 0.)));
    assert_eq!(-2., sphere.sdf(Vec3::new(2., 2., 2.)).round());
    assert_eq!(4., sphere.sdf(Vec3::new(5., 5., 5.)).round());
    assert_eq!(16., sphere.sdf(Vec3::new(20., 3., 7.)).round());
}

/// The mode to use for the editing operation
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Mode {
    /// Add the [SignedDistanceFunction] to the voxel grid
    Add,
    /// Remove the [SignedDistanceFunction] from the voxel grid
    Remove,
}

#[test]
fn test_modify_single_chunk() {
    let mut modifier = ChunkModifier::default();

    let mut chunk_map = ChunkMap::default();
    let mut world = World::default();
    let mut query = world.query::<&ChunkData>();

    modifier.apply_sdf(
        ChunkPosition::new(-2, 1, 5),
        &mut chunk_map,
        |entity| query.get(&world, entity).unwrap().expand(),
        SphereSdf(4.),
        Mode::Add,
        0.01,
        Vec3::new(6., 7., 8.),
    );

    modifier.apply_sdf(
        ChunkPosition::new(-2, 1, 5),
        &mut chunk_map,
        |entity| query.get(&world, entity).unwrap().expand(),
        SphereSdf(3.),
        Mode::Remove,
        0.01,
        Vec3::new(8., 6., 6.),
    );

    assert_eq!(1, modifier.modified.len());
    assert_eq!(1, modifier.chunks.len());
    assert!(modifier
        .modified
        .contains_key(&ChunkPosition::new(-2, 1, 5)));
}

#[test]
fn test_modify_two_chunk_border() {
    let mut modifier = ChunkModifier::default();

    let mut chunk_map = ChunkMap::default();
    let mut world = World::default();
    let mut query = world.query::<&ChunkData>();

    modifier.apply_sdf(
        ChunkPosition::new(0, 0, 0),
        &mut chunk_map,
        |entity| query.get(&world, entity).unwrap().expand(),
        SphereSdf(2.),
        Mode::Add,
        0.01,
        Vec3::new(1., 10., 10.),
    );

    assert_eq!(2, modifier.modified.len());
    assert_eq!(2, modifier.chunks.len());
    assert!(modifier.modified.contains_key(&ChunkPosition::new(0, 0, 0)));
    assert!(modifier
        .modified
        .contains_key(&ChunkPosition::new(-1, 0, 0)));
}

#[test]
fn test_modify_big_sdf() {
    let mut modifier = ChunkModifier::default();

    let mut chunk_map = ChunkMap::default();
    let mut world = World::default();
    let mut query = world.query::<&ChunkData>();

    modifier.apply_sdf(
        ChunkPosition::new(0, 0, 0),
        &mut chunk_map,
        |entity| query.get(&world, entity).unwrap().expand(),
        SphereSdf(11.),
        Mode::Add,
        0.01,
        Vec3::new(10., 10., 10.),
    );

    assert_eq!(27, modifier.modified.len());
    assert_eq!(27, modifier.chunks.len());
    for x in -1..1 {
        for y in -1..1 {
            for z in -1..1 {
                assert!(modifier.modified.contains_key(&ChunkPosition::new(x, y, z)))
            }
        }
    }
}

#[test]
fn ignore_out_of_bounds_edits() {
    let mut modifier = ChunkModifier::default();

    let mut chunk_map = ChunkMap::default();
    let mut world = World::default();
    let mut query = world.query::<&ChunkData>();

    modifier.apply_sdf(
        ChunkPosition::new(i8::MIN, i8::MIN, i8::MIN),
        &mut chunk_map,
        |entity| query.get(&world, entity).unwrap().expand(),
        SphereSdf(1.),
        Mode::Add,
        0.01,
        Vec3::new(-5., -5., -5.),
    );

    modifier.apply_sdf(
        ChunkPosition::new(i8::MAX, i8::MAX, i8::MAX),
        &mut chunk_map,
        |entity| query.get(&world, entity).unwrap().expand(),
        SphereSdf(1.),
        Mode::Add,
        0.01,
        Vec3::new(20., 20., 20.),
    );

    assert_eq!(0, modifier.modified.len());
    assert_eq!(0, modifier.chunks.len());
}
