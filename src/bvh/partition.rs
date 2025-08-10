use std::mem::swap;

use crate::{
    aabb::AABB,
    surface::{Hittable, Surface},
    vector::Vector3,
};

#[derive(Debug, Clone, Copy)]
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
    while let Some(left) = iter.find(|e| !pred(e)) {
        if let Some(right) = iter.rfind(|e| pred(e)) {
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
    use std::{
        f64,
        iter::{once, zip},
    };

    use super::*;

    pub fn surface_area_heuristic(
        left: &AABB,
        n_left: usize,
        right: &AABB,
        n_right: usize,
        bounding_box: &AABB,
    ) -> f64 {
        fn surface_area_factor(bounding_box: &AABB) -> f64 {
            let dims = bounding_box.dimensions();
            dims.x * dims.y + dims.x * dims.z + dims.y * dims.z
        }

        let parent_saf = surface_area_factor(bounding_box);
        let p_left = surface_area_factor(left) / parent_saf;
        let p_right = surface_area_factor(right) / parent_saf;

        const ROOT_TEST_COST: f64 = 1.0;

        ROOT_TEST_COST + p_left * n_left as f64 + p_right * n_right as f64
    }

    fn bounding_boxes_prefix_list<'a, I>(bounding_boxes: I) -> impl Iterator<Item = AABB>
    where
        I: Iterator<Item = &'a AABB>,
    {
        let mut acc_forward = AABB::EMPTY;
        bounding_boxes.map(move |forward| {
            acc_forward = AABB::merge(acc_forward.clone(), forward.clone());

            acc_forward.clone()
        })
    }

    #[derive(Debug, Clone)]
    struct SplitVolumes {
        left: AABB,
        right: AABB,
        split_at: f64,
    }

    /// splits[n_left] = (surfaces[..n_left].bounding_box(), surfaces[n_left..].bounding_box())
    fn splits_cache(surfaces: &[Surface], axis: &Axis) -> Box<[SplitVolumes]> {
        let mut sorted_boxes = Vec::from(surfaces)
            .into_iter()
            .map(|surface| surface.bounding_box())
            .collect::<Vec<_>>();

        sorted_boxes.sort_unstable_by(|a, b| {
            let a = get_component(axis, &a.centroid());
            let b = get_component(axis, &b.centroid());

            a.total_cmp(&b)
        });

        let forward_pf = bounding_boxes_prefix_list(sorted_boxes.iter());
        let backward_pf = bounding_boxes_prefix_list(sorted_boxes.iter().rev())
            .collect::<Vec<_>>()
            .into_iter()
            .rev();

        // add sinks at the front and back that capture splits that create 0-element partitions.
        // relevant when splitting in equal-sized buckets.
        once(SplitVolumes {
            left: AABB::EMPTY,
            right: AABB::EMPTY,
            split_at: f64::NEG_INFINITY,
        })
        .chain(
            zip(forward_pf, backward_pf)
                .enumerate()
                .map(|(i, (left, right))| SplitVolumes {
                    left,
                    right,
                    split_at: get_component(axis, &sorted_boxes[i].centroid()),
                }),
        )
        .chain(once(SplitVolumes {
            left: AABB::EMPTY,
            right: AABB::EMPTY,
            split_at: f64::INFINITY,
        }))
        .collect()
    }

    fn partition_impl<'s>(
        surfaces: &'s mut [Surface],
        splitting_planes: impl Iterator<Item = (&'s Axis, f64)>,
    ) -> (&'s mut [Surface], &'s mut [Surface]) {
        let split_at = {
            let x_splits = splits_cache(surfaces, &Axis::X);
            let y_splits = splits_cache(surfaces, &Axis::Y);
            let z_splits = splits_cache(surfaces, &Axis::Z);

            move |axis: &Axis, intercept: f64| {
                let splits = match axis {
                    Axis::X => &x_splits,
                    Axis::Y => &y_splits,
                    Axis::Z => &z_splits,
                };

                let n_left = splits.partition_point(|split| split.split_at < intercept) - 1;

                Some((n_left, splits[n_left].clone()))
            }
        };

        let bounding_box = surfaces.as_ref().bounding_box();
        let n_surfaces = surfaces.len();

        let (axis, split, _cost) = splitting_planes
            .filter_map(|(axis, split)| {
                let (
                    n_left,
                    SplitVolumes {
                        left,
                        right,
                        split_at: _,
                    },
                ) = split_at(axis, split)?;

                if left == AABB::EMPTY || right == AABB::EMPTY {
                    return None;
                }

                Some((
                    axis,
                    split,
                    surface_area_heuristic(
                        &left,
                        n_left,
                        &right,
                        n_surfaces - n_left,
                        &bounding_box,
                    ),
                ))
            })
            .min_by(|(_, _, a), (_, _, b)| a.total_cmp(b))
            .expect("No valid splitting plane");

        partition_in_place(surfaces, |surface| {
            get_component(axis, &surface.bounding_box().centroid()) < split
        })
    }

    pub mod equal_size {
        use super::*;

        pub fn partition(
            surfaces: &mut [Surface],
            buckets: u32,
        ) -> (&mut [Surface], &mut [Surface]) {
            let bounding_box = surfaces.as_ref().bounding_box();

            let splitting_planes = Axis::ALL
                .iter()
                .flat_map(|axis| {
                    let start = get_component(axis, &bounding_box.min());
                    let step = get_component(axis, &bounding_box.dimensions()) / f64::from(buckets);

                    (1..buckets)
                        .map(move |i| start + (f64::from(i) * step))
                        .map(move |split| (axis, split))
                })
                .collect::<Vec<_>>();

            partition_impl(surfaces, splitting_planes.into_iter())
        }
    }

    pub mod per_surface {
        use super::*;

        pub fn partition(surfaces: &mut [Surface]) -> (&mut [Surface], &mut [Surface]) {
            let splitting_planes = surfaces
                .iter()
                .flat_map(|surface| {
                    let centroid = surface.bounding_box().centroid();
                    Axis::ALL
                        .iter()
                        .map(move |axis| (axis, get_component(axis, &centroid)))
                })
                .collect::<Vec<_>>();

            partition_impl(surfaces, splitting_planes.into_iter())
        }
    }
}
