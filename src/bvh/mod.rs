use crate::{
    aabb::AABB,
    geometry::Hit,
    interval::Interval,
    material::Material,
    ray::Ray,
    surface::{Hittable, Surface},
};

mod partition;

/// Strategies for partitioning the surfaces in a given bounding volume.
pub enum PartitionBy {
    /// Sort surfaces by position along the longest axis and place half in each subtree.
    LongestAxisBisectSlice,

    /// Partition by position relative to midpoint of total bounding box's longest axis.
    LongestAxisMidpoint,

    /// At each volume split, choose the splitting plane that minimizes cost
    /// as defined by the surface area heuristic.
    ///
    /// The bucketing strategy controls what candidate splitting planes are evaluated.
    SurfaceAreaHeuristic(SAHBucketStrategy),
}

/// Strategies for identifying candidate splitting planes
pub enum SAHBucketStrategy {
    /// n equal-sized buckets
    EqualSize(u32),

    /// try splitting at each surface
    PerSurface,
}

impl PartitionBy {
    fn partition<'s>(&self, surfaces: &'s mut [Surface]) -> (&'s mut [Surface], &'s mut [Surface]) {
        match self {
            PartitionBy::LongestAxisBisectSlice => partition::longest_axis_bisect_slice(surfaces),
            PartitionBy::LongestAxisMidpoint => partition::longest_axis_midpoint(surfaces),
            PartitionBy::SurfaceAreaHeuristic(bucket_strategy) => match bucket_strategy {
                SAHBucketStrategy::EqualSize(buckets) => {
                    partition::sah::equal_size::partition(surfaces, *buckets)
                }
                SAHBucketStrategy::PerSurface => partition::sah::per_surface::partition(surfaces),
            },
        }
    }
}

#[derive(PartialEq, Debug)]
enum Node {
    Placeholder,
    /// right_idx, bounding_box
    Internal(Option<usize>, AABB),
    Leaf(Surface),
}

impl Node {
    fn bounding_box(&self) -> AABB {
        match self {
            Node::Placeholder => {
                unreachable!("No code path should ever get the bounding box of a placeholder node")
            }
            Node::Internal(_, aabb) => aabb.clone(),
            Node::Leaf(surface) => surface.bounding_box(),
        }
    }
}

pub struct BVH {
    tree: Box<[Node]>,
}

impl BVH {
    pub fn from_slice(mut surfaces: Box<[Surface]>, partition_by: &PartitionBy) -> Self {
        if surfaces.is_empty() {
            return Self { tree: Box::new([]) };
        }

        let tree = build_tree_rec(
            partition_by,
            Vec::with_capacity(2 * surfaces.len()),
            &mut surfaces,
        )
        .into_boxed_slice();

        Self { tree }
    }
}

fn build_tree_rec(
    partition_by: &PartitionBy,
    mut partial_nodes: Vec<Node>,
    surfaces: &mut [Surface],
) -> Vec<Node> {
    if surfaces.len() == 1 {
        partial_nodes.push(Node::Leaf(surfaces[0].clone()));
    } else if surfaces.len() == 2 {
        let (left_singleton, right_singleton) = partition_by.partition(surfaces);

        let left = left_singleton[0].clone();
        let right = right_singleton[0].clone();

        partial_nodes.push(Node::Internal(
            Some(partial_nodes.len() + 2),
            AABB::merge(left.bounding_box(), right.bounding_box()),
        ));
        partial_nodes.push(Node::Leaf(left));
        partial_nodes.push(Node::Leaf(right));
    } else {
        let (left, right) = partition_by.partition(surfaces);

        let parent_idx = partial_nodes.len();
        partial_nodes.push(Node::Placeholder);

        partial_nodes = build_tree_rec(partition_by, partial_nodes, left);
        let right_idx = partial_nodes.len();
        partial_nodes = build_tree_rec(partition_by, partial_nodes, right);

        partial_nodes[parent_idx] = Node::Internal(
            Some(right_idx),
            AABB::merge(
                partial_nodes[parent_idx + 1].bounding_box(),
                partial_nodes[right_idx].bounding_box(),
            ),
        )
    }

    partial_nodes
}

impl Hittable for BVH {
    fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<(Hit, Material)> {
        let mut stack = vec![0];
        let mut acc: Option<(Hit, Material)> = None;
        let mut shrunken_ray_t = ray_t.clone();

        while let Some(i) = stack.pop() {
            let curr = &self.tree[i];

            if !curr.bounding_box().hit(ray, &shrunken_ray_t) {
                continue;
            }

            match curr {
                Node::Placeholder => unreachable!(),
                Node::Internal(maybe_right_idx, _) => {
                    if let Some(right_idx) = maybe_right_idx {
                        stack.push(*right_idx);
                    }

                    if i + 1 < self.tree.len() {
                        stack.push(i + 1)
                    }
                }
                Node::Leaf(surface) => {
                    if let Some((hit, material)) = surface.hit(ray, &shrunken_ray_t) {
                        if let Some((nearest_hit, nearest_material)) = acc
                            && hit.t > nearest_hit.t
                        {
                            // no-op, acc is best hit
                            acc = Some((nearest_hit, nearest_material));
                        } else {
                            shrunken_ray_t.max = hit.t;
                            acc = Some((hit, material));
                        }
                    }
                }
            }
        }

        acc
    }

    fn bounding_box(&self) -> AABB {
        if self.tree.is_empty() {
            AABB::EMPTY
        } else {
            self.tree[0].bounding_box()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{geometry::Geometry, material::Material, surface::Surface, vector::Vector3};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bisect_balanced() {
        let top_left = Surface::new(
            Geometry::sphere(Vector3::new(-2.0, 1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let bottom_left = Surface::new(
            Geometry::sphere(Vector3::new(-2.0, -1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let top_right = Surface::new(
            Geometry::sphere(Vector3::new(2.0, 1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let bottom_right = Surface::new(
            Geometry::sphere(Vector3::new(2.0, -1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let scene = [
            top_left.clone(),
            bottom_left.clone(),
            top_right.clone(),
            bottom_right.clone(),
        ];

        let expected_nodes = [
            // Node 0: Internal(4, bounding_box_of_all) - root splits list sorted along x-axis
            Node::Internal(Some(4), scene.as_slice().bounding_box()),
            // Node 1: Internal(2, bounding_box_left) - left side splits list sorted along y-axis
            Node::Internal(
                Some(3),
                AABB::merge(bottom_left.bounding_box(), top_left.bounding_box()),
            ),
            Node::Leaf(bottom_left.clone()),
            Node::Leaf(top_left.clone()),
            // Node 4: Internal(6, bounding_box_right) - right side splits list sorted along y-axis
            Node::Internal(
                Some(6),
                AABB::merge(bottom_right.bounding_box(), top_right.bounding_box()),
            ),
            Node::Leaf(bottom_right.clone()),
            Node::Leaf(top_right.clone()),
        ];

        let actual_bvh = BVH::from_slice(Box::from(scene), &PartitionBy::LongestAxisBisectSlice);

        assert_eq!(Box::from(expected_nodes), actual_bvh.tree)
    }

    #[test]
    fn test_bisect_imbalanced() {
        let top_left = Surface::new(
            Geometry::sphere(Vector3::new(-2.0, 1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let bottom_left = Surface::new(
            Geometry::sphere(Vector3::new(-2.0, -1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let bottom_right = Surface::new(
            Geometry::sphere(Vector3::new(2.0, -1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let scene = [top_left.clone(), bottom_left.clone(), bottom_right.clone()];

        let expected_nodes = [
            // Node 0: Internal(2, bounding_box_of_all) - root splits list sorted along x-axis
            // but because splitting [1, 2, 3] down the "middle" returns ([1], [2, 3]),
            // this tree is expectedly suboptimal.
            Node::Internal(Some(2), scene.as_slice().bounding_box()),
            Node::Leaf(top_left.clone()),
            Node::Internal(
                Some(4),
                AABB::merge(bottom_left.bounding_box(), bottom_right.bounding_box()),
            ),
            Node::Leaf(bottom_left.clone()),
            Node::Leaf(bottom_right.clone()),
        ];

        let actual_bvh = BVH::from_slice(Box::from(scene), &PartitionBy::LongestAxisBisectSlice);

        assert_eq!(Box::from(expected_nodes), actual_bvh.tree)
    }

    #[test]
    fn test_midpoint_balanced() {
        let top_left = Surface::new(
            Geometry::sphere(Vector3::new(-2.0, 1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let bottom_left = Surface::new(
            Geometry::sphere(Vector3::new(-2.0, -1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let top_right = Surface::new(
            Geometry::sphere(Vector3::new(2.0, 1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let bottom_right = Surface::new(
            Geometry::sphere(Vector3::new(2.0, -1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let scene = [
            top_left.clone(),
            bottom_left.clone(),
            top_right.clone(),
            bottom_right.clone(),
        ];

        let expected_nodes = [
            // Node 0: Internal(4, bounding_box_of_all) - root splits scene at x=0
            Node::Internal(Some(4), scene.as_slice().bounding_box()),
            // Node 1: Internal(2, bounding_box_left) - left side splits scene at y=0
            Node::Internal(
                Some(3),
                AABB::merge(bottom_left.bounding_box(), top_left.bounding_box()),
            ),
            Node::Leaf(bottom_left.clone()),
            Node::Leaf(top_left.clone()),
            // Node 4: Internal(6, bounding_box_right) - right side splits scene at y=0
            Node::Internal(
                Some(6),
                AABB::merge(bottom_right.bounding_box(), top_right.bounding_box()),
            ),
            Node::Leaf(bottom_right.clone()),
            Node::Leaf(top_right.clone()),
        ];

        let actual_bvh = BVH::from_slice(Box::from(scene), &PartitionBy::LongestAxisMidpoint);

        assert_eq!(Box::from(expected_nodes), actual_bvh.tree)
    }

    #[test]
    fn test_midpoint_on_earth() {
        let ground = Surface::new(
            Geometry::sphere(Vector3::new(0.0, -1000.0, 0.0), 1000.0).unwrap(),
            Material::Dielectric {
                refraction_index: 1.2,
            },
        );
        let left = Surface::new(
            Geometry::sphere(Vector3::new(-2.0, 1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let right = Surface::new(
            Geometry::sphere(Vector3::new(2.0, 1.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let scene = [left.clone(), right.clone(), ground.clone()];

        let expected_nodes = [
            Node::Internal(Some(2), scene.as_slice().bounding_box()),
            // expect to split into [[ground], [left, right]] first. ground is naturally less than midpoint of longest axis, y-axis.
            Node::Leaf(ground.clone()),
            // [left, right] longest axis is x
            Node::Internal(
                Some(4),
                AABB::merge(left.bounding_box(), right.bounding_box()),
            ),
            Node::Leaf(left.clone()),
            Node::Leaf(right.clone()),
        ];

        let actual_bvh = BVH::from_slice(Box::from(scene), &PartitionBy::LongestAxisMidpoint);

        assert_eq!(Box::from(expected_nodes), actual_bvh.tree)
    }

    #[test]
    fn test_demo_sah_vs_midpoint() {
        // Three spheres placed to show SAH advantage over midpoint splitting:

        let small_left = Surface::new(
            Geometry::sphere(Vector3::new(-10.0, 10.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let large_center = Surface::new(
            Geometry::sphere(Vector3::new(-1.0, 0.0, 0.0), 3.0).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );
        let small_right = Surface::new(
            Geometry::sphere(Vector3::new(10.0, 0.0, 0.0), 0.5).unwrap(),
            Material::Dielectric {
                refraction_index: 1.0,
            },
        );

        let scene = [
            small_left.clone(),
            large_center.clone(),
            small_right.clone(),
        ];

        assert!(
            partition::sah::surface_area_heuristic(
                &[small_left.clone(), large_center.clone()],
                &[small_right.clone()],
                &scene.as_slice().bounding_box()
            ) > partition::sah::surface_area_heuristic(
                &[small_right.clone(), large_center.clone()],
                &[small_left.clone()],
                &scene.as_slice().bounding_box()
            )
        );

        // midpoint splitting produces suboptimal pairing in this test case
        let midpoint_expected = [
            Node::Internal(Some(4), scene.as_slice().bounding_box()),
            // Left group: small_left + large_center (huge bbox spanning x=[-10.5,2] y=[-3,10.5])
            Node::Internal(
                Some(3),
                AABB::merge(small_left.bounding_box(), large_center.bounding_box()),
            ),
            Node::Leaf(large_center.clone()),
            Node::Leaf(small_left.clone()),
            // Right group: just small_right
            Node::Leaf(small_right.clone()),
        ];

        let midpoint_bvh =
            BVH::from_slice(Box::from(scene.clone()), &PartitionBy::LongestAxisMidpoint);
        assert_eq!(Box::from(midpoint_expected), midpoint_bvh.tree);

        let sah_expected = [
            Node::Internal(Some(4), scene.as_slice().bounding_box()),
            Node::Internal(
                Some(3),
                [small_right.clone(), large_center.clone()]
                    .as_slice()
                    .bounding_box(),
            ),
            Node::Leaf(large_center.clone()),
            Node::Leaf(small_right.clone()),
            Node::Leaf(small_left.clone()),
        ];

        let sah_bvh = BVH::from_slice(
            Box::from(scene),
            &PartitionBy::SurfaceAreaHeuristic(SAHBucketStrategy::EqualSize(8)),
        );

        assert_eq!(Box::from(sah_expected), sah_bvh.tree)
    }
}
