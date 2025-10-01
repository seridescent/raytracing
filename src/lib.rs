pub mod aabb;
pub mod bvh;
pub mod camera;
pub mod geometry;
pub mod interval;
pub mod material;
pub mod ray;
pub mod runner;
pub mod surface;
pub mod vector;

/// Return the number of logical CPUs visible to this process.
/// https://github.com/ashvardanian/fork_union/blob/c1939907ce74113048e01aa0f5263362128cd9a2/scripts/nbody.rs#L107
#[inline]
pub fn hw_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}
