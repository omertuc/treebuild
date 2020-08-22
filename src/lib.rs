use nannou::app;
use nannou::draw;
use nannou::prelude::*;
use std::env;
use std::io::BufRead;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::{collections::HashSet, rc::Rc};
extern crate approx;
use itertools::Itertools;
use std::{ops::Sub, sync::mpsc::channel, thread};

pub mod parse_cargo_tree_output;
use parse_cargo_tree_output::{parse_tree, TreeNode};

mod drawing;
use drawing::{draw_tree, DrawCrate, DrawLine, Point};
use io::BufReader;

#[macro_use]
extern crate lazy_static;

mod active;

pub struct Model {
    tree: Rc<TreeNode>,
    mouse_last: Point,
    active_tree: Rc<TreeNode>,
    completed: HashSet<String>,
    receiver: std::sync::mpsc::Receiver<String>,
}

lazy_static! {
    static ref COMPLETED_RECEIVER: Mutex<Option<std::sync::mpsc::Receiver<String>>> =
        Mutex::new(None);
}

pub fn launch(cargo_command: Vec<&'static str>) {
    let sender = {
        let (sender, receiver) = channel();

        *COMPLETED_RECEIVER.lock().unwrap() = Some(receiver);

        sender
    };

    thread::spawn(move || {
        let build_args: Vec<_> = cargo_command
            .iter()
            .map(|x| x.to_string())
            .chain(env::args().skip(2))
            .collect();

        let mut cargo_proc = Command::new("cargo")
            .args(build_args)
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to run cargo");

        if let Some(ref mut stderr) = cargo_proc.stderr {
            let lines = BufReader::new(stderr).lines();

            let mut last_line: Option<String> = None;

            for line in lines {
                if let Some(last_line) = last_line {
                    // TODO: Let other task know of completed_crate
                    let completed_crate = last_line
                        .trim()
                        .split(' ')
                        .skip(1)
                        .take(1)
                        .join(" ")
                        .replace("_", "-");

                    sender.send(completed_crate).expect("Can't seem to send to channel");
                }

                last_line = Some(line.unwrap());
            }
        }
    });

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
        .arg("-e=no-dev")
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
        receiver: COMPLETED_RECEIVER.lock().unwrap().take().unwrap(),
    }
}

pub fn update(_app: &App, _model: &mut Model, _update: Update) {
    if let Ok(completed_crate) = _model.receiver.try_recv() {
        _model.completed.insert(completed_crate);
    }
}

fn draw_tree_defaults(
    tree: Rc<TreeNode>,
    time: app::DrawScalar,
    completed: &HashSet<String>,
    active: &HashSet<String>,
) -> (Vec<DrawCrate>, Vec<DrawLine>) {
    let transition = time.sin().abs(); 
    
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
        transition
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

    let active_crates = active::get_active();
    let actually_completed = _model.completed.sub(&active_crates);

    draw_dep(
        &actually_completed,
        &active::get_active(),
        _app.time,
        &draw,
        Rc::clone(&_model.active_tree),
    );

    draw.to_frame(_app, &frame).unwrap();
}
