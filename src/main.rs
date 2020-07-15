use nannou::prelude::*;

fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}

struct Model {
    angle: f64,
    r: f64,
    spin_r: f64
}

fn model(_app: &App) -> Model {
    Model {spin_r: 0.0, angle: 0.0, r: 50.0}
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    _model.angle += 0.1;
    _model.spin_r += 0.1;
}

fn view(_app: &App, _model: &Model, frame: Frame){
    let draw = _app.draw();

    draw.background().color(BLACK);

    draw.ellipse().x_y((_model.angle.cos() * _model.spin_r) as f32,
                       (_model.angle.sin() * _model.spin_r) as f32).
        w_h((_model.r * 2.0) as f32, (_model.r * 2.0) as f32).color(STEELBLUE);

    draw.to_frame(_app, &frame).unwrap();
}
