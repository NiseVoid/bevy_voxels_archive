# Archived voxels crate for bevy

A voxel crate for smooth voxel terrain using SDFs and the `fast_surface_nets` crate.

## The architecture

Voxel data gets stores as chunks, the chunks get some very simple compression with Run-Length Encoding. Each chunk is stored as an entity, and a `ChunkMap` resource is used to look up the entity for a given `ChunkPosition`.

Modifications to the chunk data can be batched and works by adding or subtracting signed distance functions to the voxel grid, on sdfs these operations can be as simple as a min or max operation. More specifically a smooth min and smooth max function are used which take a smoothing factor in the range 0 < factor <= 1.

## License

Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
