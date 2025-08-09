use std::mem::swap;

use crate::{
    aabb::AABB,
    surface::{Hittable, Surface},
    vector::Vector3,
};

#[derive(Debug)]
enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub const ALL: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
}

fn get_component(axis: &Axis, v: &Vector3) -> f64 {
    match axis {
        Axis::X => v.x,
        Axis::Y => v.y,
        Axis::Z => v.z,
    }
}

fn longest_axis(bounding_box: &AABB) -> &Axis {
    Axis::ALL
        .iter()
        .max_by(|&axis_a, &axis_b| {
            let a = get_component(axis_a, &bounding_box.max())
                - get_component(axis_a, &bounding_box.min());
            let b = get_component(axis_b, &bounding_box.max())
                - get_component(axis_b, &bounding_box.min());

            a.total_cmp(&b)
        })
        .unwrap() // iterator is obviously non-empty
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

pub fn longest_axis_bisect_slice(surfaces: &mut [Surface]) -> (&mut [Surface], &mut [Surface]) {
    let bounding_box = surfaces.as_ref().bounding_box();
    let longest_axis = longest_axis(&bounding_box);

    surfaces.sort_unstable_by(|a, b| {
        get_component(longest_axis, &a.bounding_box().min())
            .total_cmp(&get_component(longest_axis, &b.bounding_box().min()))
    });

    surfaces.split_at_mut(surfaces.len() / 2)
}

pub fn longest_axis_midpoint(surfaces: &mut [Surface]) -> (&mut [Surface], &mut [Surface]) {
    let bounding_box = surfaces.as_ref().bounding_box();
    let longest_axis = longest_axis(&bounding_box);
    let midpoint = get_component(longest_axis, &bounding_box.centroid());

    partition_in_place(surfaces, |surface| {
        get_component(longest_axis, &surface.bounding_box().centroid()) < midpoint
    })
}

pub mod sah {
    use super::*;

    pub fn surface_area_heuristic(left: &[Surface], right: &[Surface], bounding_box: &AABB) -> f64 {
        fn surface_area_factor(bounding_box: &AABB) -> f64 {
            let dims = bounding_box.dimensions();
            dims.x * dims.y + dims.x * dims.z + dims.y * dims.z
        }

        let parent_saf = surface_area_factor(&bounding_box);
        let p_left = surface_area_factor(&left.bounding_box()) / parent_saf;
        let p_right = surface_area_factor(&right.bounding_box()) / parent_saf;

        const ROOT_TEST_COST: f64 = 1.0;

        ROOT_TEST_COST + p_left * left.len() as f64 + p_right * right.len() as f64
    }

    pub mod equal_size {
        use super::*;

        pub fn partition(
            surfaces: &mut [Surface],
            buckets: u32,
        ) -> (&mut [Surface], &mut [Surface]) {
            // TODO: bounding box prefix list and consolidate bucket-independent pieces
            let bounding_box = surfaces.as_ref().bounding_box();

            let (axis, split, _cost) = Axis::ALL
                .iter()
                .map(|axis| {
                    let start = get_component(axis, &bounding_box.min());
                    let step = get_component(axis, &bounding_box.dimensions()) / f64::from(buckets);

                    let (best_split, min_cost) = (1..buckets)
                        .map(|i| start + (f64::from(i) * step))
                        .map(|split| {
                            let (left, right) = partition_in_place(surfaces, |surface| {
                                get_component(axis, &surface.bounding_box().centroid()) < split
                            });

                            if left.is_empty() || right.is_empty() {
                                return (split, f64::INFINITY);
                            }

                            let cost = surface_area_heuristic(left, right, &bounding_box);

                            (split, cost)
                        })
                        .min_by(|(_, a), (_, b)| a.total_cmp(b))
                        .unwrap_or((0.0, f64::INFINITY));

                    (axis, best_split, min_cost)
                })
                .min_by(|(_, _, a), (_, _, b)| a.total_cmp(b))
                .unwrap();

            partition_in_place(surfaces, |surface| {
                get_component(axis, &surface.bounding_box().centroid()) < split
            })
        }
    }

    pub mod per_surface {
        use super::*;

        pub fn partition(_surfaces: &mut [Surface]) -> (&mut [Surface], &mut [Surface]) {
            todo!("SAHBucketStrategy::PerSurface not yet implemented")
        }
    }
}
