use nannou::app;
use nannou::draw;
use nannou::prelude::*;
use std::io::{self, Write};
use std::process::Command;
extern crate approx;

pub mod parse_cargo_tree_output;
use parse_cargo_tree_output::{parse_tree, TreeNode};

mod drawing;
use drawing::{draw_tree, DrawCrate, DrawLine, Point};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    tree: TreeNode,
    active: Vec<String>,
    mouse_last: Point,
}

fn get_active() -> Vec<String> {
    // TODO: pgrep rustc to get current compiling modules
    Vec::<_>::new()
}

fn event(_app: &App, _model: &mut Model, event: WindowEvent) {
    // We can `match` on the event to do something different depending on the kind of event.
    match event {
        // Keyboard events
        KeyPressed(_key) => {}
        KeyReleased(_key) => {}

        // Mouse events
        MouseMoved(_pos) => _model.mouse_last = (_pos.x, _pos.y),
        MousePressed(_button) => {
            let (draw_crates, _draw_lines) = draw_tree_defaults(&_model.tree, _app.time);

            for draw_crate in draw_crates {
                let (x1, y1) = _model.mouse_last;
                let (x2, y2) = draw_crate.center;

                if (x2 - x1).powf(2.0) + (y2 - y1).powf(2.0) < draw_crate.radius.powf(2.0) {
                    // TODO: Change tree_current here
                    println!("Pressed on {}", draw_crate.name);
                }
            }
        }
        MouseReleased(_button) => {}
        MouseWheel(_amount, _phase) => {}
        MouseEntered => {}
        MouseExited => {}

        // Touch events
        Touch(_touch) => {}
        TouchPressure(_pressure) => {}

        // Window events
        Moved(_pos) => {}
        Resized(_size) => {}
        HoveredFile(_path) => {}
        DroppedFile(_path) => {}
        HoveredFileCancelled => {}
        Focused => {}
        Unfocused => {}
        Closed => {}
    }
}

fn model(_app: &App) -> Model {
    _app.new_window().event(event).view(view).build().unwrap();

    let output = Command::new("cargo")
        .arg("tree")
        .arg("--prefix")
        .arg("depth")
        .arg("--no-dedupe")
        .output()
        .expect("Cargo tree failed");

    println!("Cargo ran with status: {}", output.status);
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());

    let out = String::from_utf8_lossy(&output.stdout).to_string();

    Model {
        tree: parse_tree(out),
        active: Vec::<_>::new(),
        mouse_last: (0.0, 0.0),
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn draw_tree_defaults(tree: &TreeNode, time: app::DrawScalar) -> (Vec<DrawCrate>, Vec<DrawLine>) {
    draw_tree(
        (0.0, 0.0),
        tree,
        150.0,
        1.0,
        0,
        2.0 * PI,
        time.sin() * 0.1,
        (200, 100, 130),
    )
}

fn draw_dep(time: app::DrawScalar, draw: &draw::Draw, tree: &TreeNode) {
    let (tree_crates, tree_lines) = draw_tree_defaults(tree, time);

    for draw_line in tree_lines {
        draw.line()
            .start(pt2(draw_line.p1.0, draw_line.p1.1))
            .end(pt2(draw_line.p2.0, draw_line.p2.1))
            .weight(2.0)
            .color(srgba(
                draw_line.color.0,
                draw_line.color.1,
                draw_line.color.2,
                127,
            ));
    }

    for draw_crate in tree_crates {
        draw.ellipse()
            .color(srgba(
                draw_crate.color.0,
                draw_crate.color.1,
                draw_crate.color.2,
                255,
            ))
            .x_y(draw_crate.center.0, draw_crate.center.1)
            .w_h(draw_crate.radius * 2.0, draw_crate.radius * 2.0);

        if draw_crate.radius > 5.0 {
            draw.text(&draw_crate.name)
                .color(WHITE)
                .x_y(draw_crate.center.0, draw_crate.center.1)
                .w_h(200.0, 200.0);
        }
    }
}

fn view(_app: &App, _model: &Model, frame: Frame) {
    let draw = _app.draw();

    draw.background().color(BLACK);

    // let centers = [
    //     (500.0, 500.0),
    //     (-500.0, 500.0),
    //     (500.0, -500.0),
    //     (-500.0, -500.0),
    // ];

    // for (child, &center) in _model.tree.children.iter().zip(centers.iter()) {
    //     draw_dep(center, _app.time, &draw, child)
    // }

    draw_dep(_app.time, &draw, &_model.tree);

    draw.to_frame(_app, &frame).unwrap();
}
