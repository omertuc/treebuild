use nannou::prelude::*;
use rand::Rng;
use std::io::{self, Write};
use std::process::Command;
extern crate approx;

#[derive(Debug)]
struct TreeNode {
    name: String,
    children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
struct FlatEntry {
    depth: usize,
    name: String,
}

fn parse(raw: String) -> Vec<FlatEntry> {
    let mut result = Vec::<FlatEntry>::new();
    for line in raw.lines() {
        let idx = line
            .chars()
            .take_while(|x| x.is_ascii_digit())
            .collect::<String>()
            .parse::<usize>()
            .unwrap();

        let start = line.find(|c: char| !c.is_ascii_digit()).unwrap();
        let stop = line.find(|c: char| c.is_whitespace()).unwrap();

        let package = &line[start..stop];

        result.push(FlatEntry {
            depth: idx,
            name: package.to_string(),
        });
    }
    result
}

fn tree(flat: Vec<FlatEntry>) -> TreeNode {
    let root = &flat[0];
    let candidates = &flat[1..];

    TreeNode {
        name: root.name.to_string(),
        children: candidates
            .iter()
            .take_while(|child| child.depth > root.depth)
            .enumerate()
            .filter_map(|(idx, child)| {
                if child.depth == root.depth + 1 {
                    Some(tree(candidates[idx..].to_vec()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    }
}

fn parse_tree(raw: String) -> TreeNode {
    println!("{}", raw);
    tree(parse(raw))
}

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    tree: TreeNode,
    active: Vec<String>,
}

fn get_active() -> Vec<String> {
    Vec::<_>::new()
}

fn model(_app: &App) -> Model {
    let output = Command::new("cargo")
        .arg("tree")
        .arg("--prefix")
        .arg("depth")
        .arg("--no-dedupe")
        .output()
        .expect("Cargo tree failed");

    println!("Cargo ran with status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());

    let out = String::from_utf8_lossy(&output.stdout).to_string();

    Model {
        tree: parse_tree(out),
        active: Vec::<_>::new(),
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

type Point = (f32, f32);

fn get_satellites(
    center: Point,
    root_radius: f32,
    in_radius: f32,
    amount: usize,
    phase: f32,
) -> (f32, Vec<(Point, f32)>) {
    let diff_angle = (2.0 * PI) / (amount as f32);

    (
        f32::min(
            in_radius * (2.0.sqrt()) * (1.0 - diff_angle.cos()).sqrt() / 2.0,
            root_radius * 0.7,
        ),
        (0..amount)
            .map(|idx| idx as f32)
            .map(|idx| phase + idx * diff_angle)
            .map(|angle| {
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

#[test]
fn check_satt() {
    get_satellites((0.0, 0.0), 0.5, 1.0, 4, PI / 2.0)
        .1
        .iter()
        .zip(&vec![(0.0, 1.0), (1.0, 0.0), (0.0, 1.0), (1.0, 0.0)])
        .for_each(|(a, b)| {
            abs_diff_eq!(a.0, b.0);
            abs_diff_eq!(a.1, b.1);
        })
}

struct DrawCrate {
    x: f32,
    y: f32,
    radius: f32,
    r: u8,
    g: u8,
    b: u8,
    name: String,
}

fn draw_tree(
    center: Point,
    tree: &TreeNode,
    radius: f32,
    phase: f32,
    depth: usize,
) -> Vec<DrawCrate> {
    let mut result = Vec::<DrawCrate>::new();

    result.push(DrawCrate {
        x: center.0,
        y: center.1,
        radius: radius,
        r: 40 + ((210.0 * (1.0 - (depth as f32 / 15.0))) as u8),
        g: 100 + ((150.0 * (depth as f32 / 20.0)) as u8),
        b: 220,
        name: tree.name.clone(),
    });

    let (new_radius, sats) = get_satellites(
        (center.0, center.1),
        radius,
        radius * 4.0,
        tree.children.len(),
        phase,
    );

    sats.iter()
        .zip(tree.children.iter())
        .for_each(|((point, point_phase), child)| {
            result.extend(draw_tree(
                (point.0, point.1),
                &child,
                new_radius,
                point_phase,
                depth + 1,
            ))
        });

    result
}

fn view(_app: &App, _model: &Model, frame: Frame) {
    let draw = _app.draw();

    draw.background().color(BLACK);

    for draw_crate in draw_tree((0.0, 0.0), &_model.tree, 100.0, 0.0, 0) {
        draw.ellipse()
            .color(srgba(draw_crate.r, draw_crate.g, draw_crate.b, 127))
            .x_y(draw_crate.x, draw_crate.y)
            .w_h(draw_crate.radius * 2.0, draw_crate.radius * 2.0);
    }

    draw.to_frame(_app, &frame).unwrap();
}
