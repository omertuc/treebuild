use crate::parse_cargo_tree_output::TreeNode;
use std::rc::Rc;

pub type Point = (f32, f32);
pub type Color = (u8, u8, u8);

fn get_satellites(
    center: Point,
    root_radius: f32,
    in_radius: f32,
    amount: usize,
    phase: f32,
    sky: f32,
) -> (f32, Vec<(Point, f32)>) {
    let diff_angle = sky / (amount as f32);

    (
        f32::min(
            in_radius * (2.0f32.sqrt()) * (1.0 - diff_angle.cos()).sqrt() / 2.0,
            root_radius * 0.7,
        ),
        (0u32..(amount as u32))
            // 0 1 2 3 5 6 7 ... to 0 1 1 2 2 3 3 ...
            .map(|idx| ((idx as f32) / 2.0).ceil())
            // Alternate sign
            .zip([1, -1].iter().cycle())
            .map(|(idx, &sign)| idx * (sign as f32))
            // Get final angle for this satellite
            .map(|alternating_idx| phase + alternating_idx * diff_angle)
            // Get final cartesian coordinates of this satellite, also append angle
            .map(|angle: f32| {
                (
                    (
                        center.0 + angle.cos() * in_radius,
                        center.1 + angle.sin() * in_radius,
                    ),
                    angle,
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub struct DrawCrate {
    pub center: Point,
    pub radius: f32,
    pub color: Color,
    pub name: String,
    pub tree: Rc<TreeNode>
}

pub struct DrawLine {
    pub p1: Point,
    pub p2: Point,
    pub color: Color,
}

pub fn draw_tree(
    center: Point,
    tree: Rc<TreeNode>,
    radius: f32,
    phase: f32,
    depth: usize,
    sky: f32,
    phase_accum: f32,
    color: Color,
) -> (Vec<DrawCrate>, Vec<DrawLine>) {
    let mut crate_draws = Vec::<DrawCrate>::new();
    let mut line_draws = Vec::<DrawLine>::new();

    crate_draws.push(DrawCrate {
        center: center,
        radius: radius,
        color: color,
        name: tree.name.clone(),
        tree: Rc::clone(&tree) 
    });

    let child_count = tree.children.len();

    let (new_radius, sats) = get_satellites(
        (center.0, center.1),
        radius,
        radius * 2.0,
        child_count,
        phase + phase_accum,
        sky,
    );

    sats.into_iter()
        .zip(tree.children.iter())
        .for_each(|((point, point_phase), child)| {
            let (child_crate_draws, child_line_draws) = draw_tree(
                {
                    if child.children.len() < 5 {
                        (point.0, point.1)
                    } else {
                        (
                            point.0 + new_radius * point_phase.cos(),
                            point.1 + new_radius * point_phase.sin(),
                        )
                    }
                },
                &child,
                new_radius,
                point_phase,
                depth + 1,
                {
                    if child.children.len() < 5 {
                        std::f32::consts::PI / 2.0
                    } else {
                        std::f32::consts::PI * 1.5
                    }
                },
                phase_accum,
                (220, 130, 110),
            );

            line_draws.push(DrawLine {
                p1: center,
                p2: point,
                color: (255, 255, 255),
            });

            crate_draws.extend(child_crate_draws);
            line_draws.extend(child_line_draws);
        });

    (crate_draws, line_draws)
}
