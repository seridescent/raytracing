use std::error::Error;
use std::time::Instant;

use crate::bvh::{BVH, PartitionBy, SAHBucketStrategy};
use crate::camera::Camera;
use crate::surface::Surface;

pub struct RenderRunner {
    pub camera: Camera,
    pub partition_strategy: PartitionBy,
}

impl Default for RenderRunner {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            partition_strategy: PartitionBy::SurfaceAreaHeuristic(SAHBucketStrategy::PerSurface),
        }
    }
}

impl RenderRunner {
    pub fn run(self, surfaces: Box<[Surface]>) -> Result<(), Box<dyn Error>> {
        let start_time = Instant::now();

        let bvh_start_time = Instant::now();
        let world = BVH::from_slice(surfaces, &self.partition_strategy);
        let bvh_time = bvh_start_time.elapsed();

        let render_start_time = Instant::now();
        self.camera.initialize().render(&world);
        let render_time = render_start_time.elapsed();

        let total_time = start_time.elapsed();

        eprintln!(
            "\n\nDone!\nTotal runtime: {total_time:#?}\nBVH construction: {bvh_time:#?}\nRendering: {render_time:#?}",
        );

        Ok(())
    }
}
