use anyhow::Result;
use crossterm::terminal;
use pixel_loop::canvas::{Canvas, CrosstermCanvas, RenderableCanvas};
use pixel_loop::color::{Color, HslColor};
use pixel_loop::input::{CrosstermInputState, KeyboardKey, KeyboardState};
use pixel_loop::rand::Rng;
use pixel_loop::EngineEnvironment;
use std::time::Duration;

struct Particle {
    position: (f64, f64),
    dimensions: (u32, u32),
    lifetime: f64,
    fading: f64,
    speed: (f64, f64),
    acceleration: (f64, f64),
    color: Color,
}

impl Particle {
    pub fn new(x: i64, y: i64, width: u32, height: u32, color: Color) -> Self {
        Self {
            position: (x as f64, y as f64),
            dimensions: (width, height),
            lifetime: 1.0,
            fading: 0.01,
            speed: (0.0, 0.0),
            acceleration: (0.0, 0.0),
            color,
        }
    }

    pub fn with_fading(self, fading: f64) -> Self {
        Self { fading, ..self }
    }

    pub fn with_speed(self, x: f64, y: f64) -> Self {
        Self {
            speed: (x, y),
            ..self
        }
    }

    pub fn with_acceleration(self, x: f64, y: f64) -> Self {
        Self {
            acceleration: (x, y),
            ..self
        }
    }

    pub fn draw<C: Canvas>(&self, canvas: &mut C) {
        if self.lifetime <= 0.0 {
            return;
        }
        canvas.filled_rect(
            self.position.0.round() as i64,
            self.position.1.round() as i64,
            self.dimensions.0,
            self.dimensions.1,
            &Color::from_rgb(
                (self.color.r as f64 * self.lifetime)
                    .round()
                    .clamp(0.0, 255.0) as u8,
                (self.color.g as f64 * self.lifetime)
                    .round()
                    .clamp(0.0, 255.0) as u8,
                (self.color.b as f64 * self.lifetime)
                    .round()
                    .clamp(0.0, 255.0) as u8,
            ),
        );
    }

    pub fn update(&mut self) {
        if self.lifetime <= 0.0 {
            return;
        }
        self.speed = (
            self.speed.0 + self.acceleration.0,
            self.speed.1 + self.acceleration.1,
        );
        self.lifetime -= self.fading;
        self.position = (
            self.position.0 + self.speed.0,
            self.position.1 + self.speed.1,
        );
    }

    pub fn is_dead(&self) -> bool {
        self.lifetime <= 0.0
    }
}

struct Firework {
    rocket: Option<Particle>,
    effect: Vec<Particle>,
    base_color: HslColor,
}

impl Firework {
    pub fn new(x: i64, y: i64, y_speed: f64, effect_color: Color) -> Self {
        Self {
            rocket: Some(
                Particle::new(x, y, 1, 3, Color::from_rgb(255, 255, 255))
                    .with_acceleration(0.0, 0.02)
                    .with_speed(0.0, y_speed)
                    .with_fading(0.0),
            ),
            effect: vec![],
            base_color: effect_color.as_hsl(),
        }
    }

    pub fn draw<C: Canvas>(&self, canvas: &mut C) {
        if let Some(ref rocket) = self.rocket {
            rocket.draw(canvas);
        }

        for particle in self.effect.iter() {
            particle.draw(canvas);
        }
    }

    pub fn update(&mut self, ee: &mut EngineEnvironment) {
        if let Some(ref mut rocket) = self.rocket {
            rocket.update();
            if rocket.speed.1 > -0.3 {
                for _ in 0..25 {
                    self.effect.push(
                        Particle::new(
                            rocket.position.0.round() as i64,
                            rocket.position.1.round() as i64,
                            1,
                            1,
                            HslColor::new(
                                self.base_color.h,
                                (self.base_color.s + (ee.rand.gen::<f64>() - 0.5) * 2.0 * 20.0)
                                    .clamp(0.0, 100.0),
                                (self.base_color.s + (ee.rand.gen::<f64>() - 0.5) * 2.0 * 40.0)
                                    .clamp(0.0, 100.0),
                            )
                            .into(),
                        )
                        .with_acceleration(0.0, 0.02)
                        .with_speed(
                            1.5 * (ee.rand.gen::<f64>() - 0.5),
                            1.5 * (ee.rand.gen::<f64>() - 0.9),
                        ),
                    );
                }
                self.rocket = None;
            }
        }

        for particle in self.effect.iter_mut() {
            particle.update();
        }
    }

    pub fn is_dead(&self) -> bool {
        self.rocket.is_none() && self.effect.iter().all(|effect| effect.is_dead())
    }
}

struct State {
    fireworks: Vec<Firework>,
}

impl State {
    fn new() -> Self {
        Self { fireworks: vec![] }
    }
}

fn main() -> Result<()> {
    let (terminal_width, terminal_height) = terminal::size()?;
    let width = terminal_width;
    let height = terminal_height * 2;
    let mut canvas = CrosstermCanvas::new(width, height);
    canvas.set_refresh_limit(120);
    let mut state = State::new();
    let input = CrosstermInputState::new();

    pixel_loop::run(60, state, input, canvas, update, render)?;
    Ok(())
}

fn update(
    env: &mut EngineEnvironment,
    state: &mut State,
    input: &CrosstermInputState,
    canvas: &mut CrosstermCanvas,
) -> Result<()> {
    if input.is_key_pressed(KeyboardKey::Q) {
        std::process::exit(0);
    }

    state.fireworks.retain(|firework| {
        !firework.is_dead()
    });

    if env.rand.gen::<f64>() < 0.10 {
        state.fireworks.push(Firework::new(
            (env.rand.gen::<u32>() % canvas.width()) as i64,
            canvas.height() as i64,
            -1.0 + env.rand.gen::<f64>() * -1.0,
            Color::from_rgb(
                env.rand.gen::<u8>(),
                env.rand.gen::<u8>(),
                env.rand.gen::<u8>(),
            ),
        ));
    }

    for firework in state.fireworks.iter_mut() {
        firework.update(env);
    }
    Ok(())
}

fn render(
    _env: &mut EngineEnvironment,
    state: &mut State,
    _input: &CrosstermInputState,
    canvas: &mut CrosstermCanvas,
    _dt: Duration,
) -> Result<()> {
    canvas.clear_screen(&Color::from_rgb(0, 0, 0));

    for firework in state.fireworks.iter() {
        firework.draw(canvas);
    }

    canvas.render()?;
    Ok(())
}
