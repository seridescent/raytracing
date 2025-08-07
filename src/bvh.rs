use std::mem::swap;

use crate::{
    aabb::AABB,
    geometry::Hit,
    interval::Interval,
    material::Material,
    ray::Ray,
    surface::{Hittable, Surface},
    vector::Vector3,
};

pub enum PartitionBy {
    /// Sort surfaces by position along the longest axis and place half in each subtree.
    LongestAxisBisectSlice,

    /// Partition by position relative to midpoint of total bounding box's longest axis.
    LongestAxisMidpoint,
}

impl PartitionBy {
    fn position_along_longest_axis_fn(bounding_box: &AABB) -> fn(&Vector3) -> f64 {
        enum Axis {
            X,
            Y,
            Z,
        }

        let (longest_axis, _) = [
            (Axis::X, bounding_box.max().x - bounding_box.min().x),
            (Axis::Y, bounding_box.max().y - bounding_box.min().y),
            (Axis::Z, bounding_box.max().z - bounding_box.min().z),
        ]
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap(); // iterator is obviously non-empty

        match longest_axis {
            Axis::X => |bounding_box| bounding_box.x,
            Axis::Y => |bounding_box| bounding_box.y,
            Axis::Z => |bounding_box| bounding_box.z,
        }
    }

    fn partition<'s>(&self, surfaces: &'s mut [Surface]) -> (&'s mut [Surface], &'s mut [Surface]) {
        match self {
            PartitionBy::LongestAxisBisectSlice => {
                let bounding_box = surfaces.as_ref().bounding_box();
                let key_fn = Self::position_along_longest_axis_fn(&bounding_box);

                surfaces.sort_unstable_by(|a, b| {
                    key_fn(&a.bounding_box().min()).total_cmp(&key_fn(&b.bounding_box().min()))
                });

                surfaces.split_at_mut(surfaces.len() / 2)
            }
            PartitionBy::LongestAxisMidpoint => {
                let bounding_box = surfaces.as_ref().bounding_box();
                let key_fn = Self::position_along_longest_axis_fn(&bounding_box);
                let midpoint = key_fn(&bounding_box.centroid());

                Self::partition_in_place(surfaces, |surface| {
                    key_fn(&surface.bounding_box().centroid()) < midpoint
                })
            }
        }
    }

    fn partition_in_place(
        surfaces: &mut [Surface],
        pred: impl Fn(&Surface) -> bool,
    ) -> (&mut [Surface], &mut [Surface]) {
        let mut iter = surfaces.iter_mut();
        while let Some(left) = iter.find(|e| !pred(*e)) {
            if let Some(right) = iter.rfind(|e| pred(*e)) {
                swap(left, right);
            } else {
                break;
            }
        }

        surfaces.split_at_mut(surfaces.partition_point(pred))
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

        let tree = Self::build_tree_rec(
            partition_by,
            Vec::with_capacity(2 * surfaces.len()),
            &mut surfaces,
        )
        .into_boxed_slice();

        Self { tree }
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

            partial_nodes = Self::build_tree_rec(partition_by, partial_nodes, left);
            let right_idx = partial_nodes.len();
            partial_nodes = Self::build_tree_rec(partition_by, partial_nodes, right);

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
            Node::Internal(
                Some(4),
                AABB::merge(top_left.bounding_box(), bottom_right.bounding_box()),
            ),
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
            Node::Internal(
                Some(2),
                AABB::merge(top_left.bounding_box(), bottom_right.bounding_box()),
            ),
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
}
