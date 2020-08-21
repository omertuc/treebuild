use nannou::app;
use nannou::draw;
use nannou::prelude::*;
use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::{collections::HashSet, rc::Rc};
extern crate approx;
use std::ops::Sub;

pub mod parse_cargo_tree_output;
use parse_cargo_tree_output::{parse_tree, TreeNode};

mod drawing;
use drawing::{draw_tree, DrawCrate, DrawLine, Point};

mod active;

pub struct Model {
    tree: Rc<TreeNode>,
    mouse_last: Point,
    active_tree: Rc<TreeNode>,
    completed: HashSet<String>,
    currently_active: HashSet<String>,
    previously_active: HashSet<String>,
}

pub fn launch(cargo_command: Vec<&str>) {
    let build_args: Vec<_> = cargo_command
        .iter()
        .map(|x| x.to_string())
        .chain(env::args().skip(2))
        .collect();

    Command::new("cargo").args(build_args).spawn().expect("Failed to run cargo");

    nannou::app(model).update(update).run();
}

fn event(_app: &App, _model: &mut Model, event: WindowEvent) {
    // We can `match` on the event to do something different depending on the kind of event.
    match event {
        // Keyboard events
        KeyPressed(_key) => _model.active_tree = Rc::clone(&_model.tree),
        KeyReleased(_key) => {}

        // Mouse events
        MouseMoved(_pos) => _model.mouse_last = (_pos.x, _pos.y),
        MousePressed(_button) => {}
        MouseReleased(_button) => {
            let (draw_crates, _draw_lines) = draw_tree_defaults(
                Rc::clone(&_model.active_tree),
                _app.time,
                &HashSet::new(),
                &HashSet::new(),
            );

            for draw_crate in draw_crates {
                let (x1, y1) = _model.mouse_last;
                let (x2, y2) = draw_crate.center;

                if (x2 - x1).powf(2.0) + (y2 - y1).powf(2.0) < draw_crate.radius.powf(2.0) {
                    _model.active_tree = Rc::clone(&draw_crate.tree);
                }
            }
        }
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

pub fn model(_app: &App) -> Model {
    _app.new_window().event(event).view(view).build().unwrap();

    let output = Command::new("cargo")
        .arg("tree")
        .arg("--prefix")
        .arg("depth")
        .arg("--no-dedupe")
        .output()
        .expect("Cargo tree failed");

    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());

    let out = String::from_utf8_lossy(&output.stdout).to_string();

    let parsed_tree = &parse_tree(out);

    Model {
        tree: Rc::clone(&parsed_tree),
        mouse_last: (0.0, 0.0),
        active_tree: Rc::clone(&parsed_tree),
        completed: HashSet::<_>::new(),
        currently_active: HashSet::<_>::new(),
        previously_active: HashSet::<_>::new(),
    }
}

pub fn update(_app: &App, _model: &mut Model, _update: Update) {
    _model.currently_active = active::get_active();

    _model
        .previously_active
        .extend(_model.currently_active.clone());
    _model.completed = _model.previously_active.sub(&_model.currently_active);
}

fn draw_tree_defaults(
    tree: Rc<TreeNode>,
    time: app::DrawScalar,
    completed: &HashSet<String>,
    active: &HashSet<String>,
) -> (Vec<DrawCrate>, Vec<DrawLine>) {
    draw_tree(
        (0.0, 0.0),
        tree,
        150.0,
        1.0,
        0,
        2.0 * PI,
        time.sin() * 0.1,
        (200, 100, 130),
        &completed,
        &active,
    )
}

fn draw_dep(
    completed: &HashSet<String>,
    active: &HashSet<String>,
    time: app::DrawScalar,
    draw: &draw::Draw,
    tree: Rc<TreeNode>,
) {
    let (tree_crates, tree_lines) = draw_tree_defaults(tree, time, &completed, &active);

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
                127,
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

    draw_dep(
        &_model.completed,
        &_model.currently_active,
        _app.time,
        &draw,
        Rc::clone(&_model.active_tree),
    );

    draw.to_frame(_app, &frame).unwrap();
}
