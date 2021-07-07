mod soup;
mod style;

use futures::prelude::*;
use rand::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iced::button::{self, Button};
use iced::canvas::{Cache, Canvas, Cursor, Frame, Geometry, Path};
use iced::slider::{self, Slider};
use iced::time;
use iced::{
    Align, Application, Clipboard, Color, Column, Command, Container, Element, HorizontalAlignment,
    Length, Point, Rectangle, Row, Settings, Subscription, VerticalAlignment,
};

use crate::soup::*;

#[derive(Clone, Debug)]
enum Message {
    Tick,
    TogglePlay,
    Randomize,
    DpeChanged(f32),
    TickAmountChanged(i32),
    Randomized { duration: Duration },
    Ticked { duration: Duration },
    ParametersChanged(Parameters),
}

#[derive(Default)]
struct Controls {
    play_button: button::State,
    next_button: button::State,
    randomize_button: button::State,
    dpe_slider: slider::State,
    tick_amount_slider: slider::State,
}

#[derive(Default)]
struct Pps {
    state: State,
    controls: Controls,
}

impl Pps {
    fn tick(&mut self) -> Option<impl Future<Output = Message>> {
        if self.state.is_ticking {
            return None;
        }

        self.state.is_ticking = true;

        let amount = self.state.tick_amount;
        let parameters = self.state.parameters.clone();
        let particles = self.state.particles.clone();

        Some(async move {
            let start = Instant::now();
            let particles = &mut *particles.lock().unwrap();

            for _ in 0..amount {
                let mut blocks =
                    vec![
                        vec![Vec::new(); (parameters.x_max / parameters.radius).ceil() as usize];
                        (parameters.y_max / parameters.radius).ceil() as usize
                    ];

                // reset neighbor counts & segregate them into block
                for (id, particle) in particles.iter_mut().enumerate() {
                    particle.n = 0;
                    particle.r = 0;
                    particle.l = 0;
                    let x_block = (particle.x / parameters.radius).floor() as usize;
                    let y_block = (particle.y / parameters.radius).floor() as usize;
                    blocks[y_block][x_block].push(id);
                }

                // calculating neighbor counts
                for (y_block, row) in blocks.iter().enumerate() {
                    for (x_block, block) in row.iter().enumerate() {
                        let x_sub = if x_block == 0 {
                            row.len() - 1
                        } else {
                            x_block - 1
                        };
                        let x_sup = if x_block == row.len() - 1 {
                            0
                        } else {
                            x_block + 1
                        };
                        let y_sub = if y_block == 0 {
                            blocks.len() - 1
                        } else {
                            y_block - 1
                        };
                        let y_sup = if y_block == blocks.len() - 1 {
                            0
                        } else {
                            y_block + 1
                        };
                        for id1 in block {
                            for block in [
                                &blocks[y_sub][x_sub],
                                &blocks[y_sub][x_block],
                                &blocks[y_sub][x_sup],
                                &blocks[y_block][x_sub],
                                &blocks[y_block][x_block],
                                &blocks[y_block][x_sup],
                                &blocks[y_sup][x_sub],
                                &blocks[y_sup][x_block],
                                &blocks[y_sup][x_sup],
                            ]
                            .iter()
                            {
                                for id2 in block.iter() {
                                    if id1 < id2 {
                                        let dx = wrap2(
                                            particles[*id2].x - particles[*id1].x,
                                            parameters.x_max,
                                        );
                                        let dy = wrap2(
                                            particles[*id2].y - particles[*id1].y,
                                            parameters.y_max,
                                        );
                                        let ds2 = dx * dx + dy * dy;
                                        if ds2 < parameters.radius * parameters.radius {
                                            particles[*id1].n += 1;
                                            particles[*id2].n += 1;
                                            if dy.atan2(dx) > particles[*id1].phi {
                                                // particle 2 is to the right of particle 1
                                                particles[*id1].r += 1;
                                            } else {
                                                // particle 2 is to the left of particle 1
                                                particles[*id1].l += 1;
                                            }
                                            if dy.atan2(dx) < particles[*id2].phi {
                                                // particle 1 is to the right of particle 2
                                                particles[*id2].r += 1;
                                            } else {
                                                // particle 1 is to the left of particle 2
                                                particles[*id2].l += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // update movement
                for particle in particles.iter_mut() {
                    use std::f32::consts::PI;
                    const TWO_PI: f32 = 2.0 * PI;
                    particle.phi += parameters.alpha
                        + parameters.beta
                            * (particle.r + particle.l) as f32
                            * (particle.r - particle.l).signum() as f32;
                    particle.phi = wrap(particle.phi + PI, TWO_PI) - PI;
                    particle.x = wrap(
                        particle.x + particle.phi.cos() * parameters.velocity,
                        parameters.x_max,
                    );
                    particle.y = wrap(
                        particle.y + particle.phi.sin() * parameters.velocity,
                        parameters.y_max,
                    );
                }
            }

            let duration = start.elapsed();

            Message::Ticked { duration }
        })
    }

    fn randomize(&mut self) -> Option<impl Future<Output = Message>> {
        let parameters = self.state.parameters.clone();
        let particles = self.state.particles.clone();

        Some(async move {
            let start = Instant::now();
            let particles = &mut *particles.lock().unwrap();
            particles.clear();

            let particle_count =
                (parameters.x_max * parameters.y_max * parameters.dpe).round() as i32;

            for _ in 0..particle_count {
                let x = (random::<f32>()) * parameters.x_max;
                let y = (random::<f32>()) * parameters.y_max;
                let phi = (random::<f32>() - 0.5) * std::f32::consts::PI * 2.0;
                particles.push(Particle {
                    x,
                    y,
                    phi,
                    ..Default::default()
                });
            }

            let duration = start.elapsed();

            Message::Randomized { duration: duration }
        })
    }
}

impl Application for Pps {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self { ..Self::default() },
            Command::perform(
                tokio::fs::read_to_string("parameters.json"),
                |result| match result {
                    Ok(parameters) => match serde_json::from_str::<Parameters>(&parameters) {
                        Ok(parameters) => Message::ParametersChanged(parameters),
                        Err(_) => Message::ParametersChanged(Parameters::default()),
                    },
                    Err(_) => Message::ParametersChanged(Parameters::default()),
                },
            ),
        )
    }

    fn title(&self) -> String {
        String::from("Primordial Particle System")
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.state.is_playing {
            time::every(Duration::from_millis(100)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::TogglePlay => {
                self.state.is_playing = !self.state.is_playing;
            }
            Message::Tick => {
                if let Some(task) = self.tick() {
                    return Command::perform(task, |message| message);
                }
            }
            Message::Randomize => {
                if let Some(task) = self.randomize() {
                    return Command::perform(task, |message| message);
                }
            }
            Message::DpeChanged(dpe) => {
                self.state.parameters.dpe = dpe;
            }
            Message::TickAmountChanged(tick_amount) => {
                self.state.tick_amount = tick_amount;
            }
            Message::Randomized { duration } => {
                self.state.timestep = 0;
                self.state.last_tick_duration = duration;
                self.state.cache.clear();
            }
            Message::Ticked { duration } => {
                self.state.is_ticking = false;
                self.state.timestep += self.state.tick_amount;
                self.state.last_tick_duration = duration;
                self.state.cache.clear();
            }
            Message::ParametersChanged(parameters) => {
                self.state.parameters = parameters;
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let playback_controls = Row::new()
            .spacing(10)
            .push(
                Button::new(
                    &mut self.controls.play_button,
                    iced::widget::Text::new(if self.state.is_playing {
                        "Stop"
                    } else {
                        "Play"
                    }),
                )
                .on_press(Message::TogglePlay)
                .style(style::Button),
            )
            .push(
                Button::new(
                    &mut self.controls.next_button,
                    iced::widget::Text::new("Next"),
                )
                .on_press(Message::Tick)
                .style(style::Button),
            )
            .push(
                Button::new(
                    &mut self.controls.randomize_button,
                    iced::widget::Text::new("Randomize"),
                )
                .on_press(Message::Randomize)
                .style(style::Button),
            );

        let dpe_controls = Row::new()
            .spacing(10)
            .push(
                Slider::new(
                    &mut self.controls.dpe_slider,
                    0.0..=0.2,
                    self.state.parameters.dpe as f32,
                    |dpe| Message::DpeChanged(dpe),
                )
                .step(0.01)
                .width(Length::Units(200))
                .style(style::Slider),
            )
            .push(
                iced::widget::Text::new(format!("DPE = {:.2}", self.state.parameters.dpe)).size(16),
            )
            .align_items(Align::Center);

        let tick_amount_controls = Row::new()
            .spacing(10)
            .push(
                Slider::new(
                    &mut self.controls.tick_amount_slider,
                    1..=50,
                    self.state.tick_amount,
                    |tick_amount| Message::TickAmountChanged(tick_amount),
                )
                .step(1)
                .width(Length::Units(200))
                .style(style::Slider),
            )
            .push(iced::widget::Text::new(format!("Speed = {}", self.state.tick_amount)).size(16))
            .align_items(Align::Center);

        let controls = Row::new()
            .spacing(20)
            .push(playback_controls)
            .push(dpe_controls)
            .push(tick_amount_controls);

        let content = Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Align::Center)
            .push(
                Canvas::new(&mut self.state)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .push(controls);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Container)
            .into()
    }
}

struct State {
    particles: Arc<Mutex<Particles>>,
    parameters: Parameters,
    is_playing: bool,
    is_ticking: bool,
    tick_amount: i32,
    timestep: i32,
    last_tick_duration: Duration,
    cache: Cache,
}

impl Default for State {
    fn default() -> Self {
        Self {
            particles: Arc::new(Mutex::new(Particles::default())),
            parameters: Parameters::default(),
            is_playing: false,
            is_ticking: false,
            tick_amount: 1,
            timestep: 0,
            last_tick_duration: Duration::default(),
            cache: Cache::default(),
        }
    }
}

impl<'a> iced::canvas::Program<Message> for State {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let circle_radius = 30.0 / (self.parameters.y_max * self.parameters.x_max).ln();
        let particles = &*self.particles.lock().unwrap();

        let start = Instant::now();

        let soup_geometry = self.cache.draw(bounds.size(), |frame| {
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            frame.fill(&background, Color::WHITE);

            for particle in particles.iter() {
                let color = {
                    if particle.n < 13 {
                        Color::new(0.0, 0.9, 0.0, 1.0)
                    } else if particle.n <= 15 {
                        Color::new(0.25, 0.15, 0.0, 1.0)
                    } else if particle.n <= 35 {
                        Color::new(0.0, 0.0, 1.0, 1.0)
                    } else {
                        Color::new(1.0, 1.0, 0.0, 1.0)
                    }
                };
                let circle = Path::circle(
                    Point::new(
                        particle.x / self.parameters.x_max * bounds.width as f32,
                        particle.y / self.parameters.y_max * bounds.height as f32,
                    ),
                    circle_radius,
                );
                frame.fill(&circle, color);
            }
        });

        let duration = start.elapsed();

        let overlay = {
            let mut frame = Frame::new(bounds.size());

            let text = iced::canvas::Text {
                color: Color::BLACK,
                size: 14.0,
                position: Point::new(frame.width(), frame.height()),
                horizontal_alignment: HorizontalAlignment::Right,
                vertical_alignment: VerticalAlignment::Bottom,
                ..Default::default()
            };

            frame.fill_text(iced::canvas::Text {
                content: format! {
                    "timestep = {}\nlast_tick_duration = {:?}\nDraw duration: {:?}\nParticle count: {}",
                    self.timestep,
                    self.last_tick_duration,
                    duration,
                    particles.len(),
                },
                ..text
            });

            frame.into_geometry()
        };

        vec![soup_geometry, overlay]
    }
}

fn wrap(v: f32, max: f32) -> f32 {
    if v < 0.0 && v > -max * f32::EPSILON {
        0.0
    } else {
        v - (v / max).floor() * max
    }
}

fn wrap2(v: f32, max: f32) -> f32 {
    v - (v / max).round() * max
}

fn main() -> iced::Result {
    Pps::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}
