use serde::Deserialize;

#[derive(Clone, Debug, Default)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub phi: f32,
    pub r: i32,
    pub l: i32,
    pub n: i32,
}

pub type Particles = Vec<Particle>;

#[derive(Clone, Debug, Deserialize)]
pub struct Parameters {
    pub velocity: f32,
    pub radius: f32,
    pub alpha: f32,
    pub beta: f32,
    pub dpe: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            velocity: 0.67,
            radius: 5.0,
            alpha: std::f32::consts::PI,
            beta: 17.0 / 180.0 * std::f32::consts::PI,
            dpe: 0.09,
            x_max: 250.0,
            y_max: 250.0,
        }
    }
}
