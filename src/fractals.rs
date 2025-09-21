use nalgebra::{Vector3, Vector4, Matrix4};
use wasm_bindgen::prelude::*;

// Trait for all fractal types - thinking ahead for extensibility
pub trait FractalGenerator {
    fn distance_estimator(&self, pos: &Vector4<f32>) -> f32;
    fn get_color(&self, iterations: i32, distance: f32, pos: &Vector4<f32>) -> Vector3<f32>;
    fn get_name(&self) -> &'static str;
}

// Enhanced Mandelbulb with time evolution
pub struct Mandelbulb {
    pub power: f32,
    pub iterations: i32,
    pub time: f32,
}

impl FractalGenerator for Mandelbulb {
    fn distance_estimator(&self, pos: &Vector4<f32>) -> f32 {
        let mut z = Vector3::new(pos.x, pos.y, pos.z);
        let mut dr = 1.0f32;
        let mut r = 0.0f32;

        // Time-evolving power
        let dynamic_power = self.power + (self.time * 0.1).sin() * 2.0;

        for _ in 0..self.iterations {
            r = z.norm();
            if r > 2.0 { break; }

            // Spherical coordinates with 4D influence
            let theta = (z.z / r).acos() + pos.w * 0.1 + self.time * 0.05;
            let phi = z.y.atan2(z.x) + self.time * 0.03;

            dr = r.powf(dynamic_power - 1.0) * dynamic_power * dr + 1.0;

            let zr = r.powf(dynamic_power);
            z = Vector3::new(
                zr * theta.sin() * phi.cos(),
                zr * theta.sin() * phi.sin(),
                zr * theta.cos()
            ) + Vector3::new(pos.x, pos.y, pos.z);
        }

        0.5 * r.ln() * r / dr
    }

    fn get_color(&self, iterations: i32, distance: f32, pos: &Vector4<f32>) -> Vector3<f32> {
        // Much more vibrant and dynamic coloring
        let iteration_factor = iterations as f32 / self.iterations as f32;

        // Multi-layered hue calculation for rich colors
        let base_hue = (iteration_factor * 6.0 + self.time * 0.5).sin() * 0.5 + 0.5;
        let depth_hue = (pos.w * 3.0 + self.time * 0.3).cos() * 0.3;
        let position_hue = ((pos.x + pos.y + pos.z) * 0.1 + self.time * 0.1).sin() * 0.2;

        let hue = (base_hue + depth_hue + position_hue).fract();

        // High saturation for vivid colors
        let saturation = 0.8 + (1.0 - distance.min(1.0)) * 0.2;

        // Dynamic brightness with pulsing effect
        let pulse = (self.time * 2.0 + pos.x * 0.5).sin() * 0.3 + 0.7;
        let value = (1.0 - (distance * 4.0).min(0.9)) * pulse;

        hsv_to_rgb(hue, saturation, value)
    }

    fn get_name(&self) -> &'static str { "Mandelbulb" }
}

// Julia4D set - 4D Julia fractals
pub struct Julia4D {
    pub c: Vector4<f32>,
    pub iterations: i32,
    pub time: f32,
}

impl FractalGenerator for Julia4D {
    fn distance_estimator(&self, pos: &Vector4<f32>) -> f32 {
        let mut z = *pos;
        let mut dz = Vector4::new(1.0, 0.0, 0.0, 0.0);

        // Time-evolving Julia constant
        let dynamic_c = Vector4::new(
            self.c.x + (self.time * 0.1).sin() * 0.3,
            self.c.y + (self.time * 0.13).cos() * 0.2,
            self.c.z + (self.time * 0.07).sin() * 0.25,
            self.c.w + (self.time * 0.11).cos() * 0.15
        );

        for _ in 0..self.iterations {
            let r = z.norm();
            if r > 4.0 { break; }

            // 4D quaternion-like multiplication
            dz = self.quat_mult_derivative(&z, &dz) + Vector4::new(1.0, 0.0, 0.0, 0.0);
            z = self.quat_square(&z) + dynamic_c;
        }

        let r = z.norm();
        0.5 * r.ln() * r / dz.norm()
    }

    fn get_color(&self, iterations: i32, distance: f32, pos: &Vector4<f32>) -> Vector3<f32> {
        let angle = (pos.x.atan2(pos.y) + self.time * 0.1) / (2.0 * std::f32::consts::PI);
        let depth = (pos.z + pos.w) * 0.1 + self.time * 0.05;

        let hue = (angle + depth).fract();
        let saturation = (1.0 - distance * 0.5).max(0.2);
        let value = (iterations as f32 / self.iterations as f32).powf(0.7);

        hsv_to_rgb(hue, saturation, value)
    }

    fn get_name(&self) -> &'static str { "Julia4D" }
}

impl Julia4D {
    fn quat_square(&self, q: &Vector4<f32>) -> Vector4<f32> {
        Vector4::new(
            q.x * q.x - q.y * q.y - q.z * q.z - q.w * q.w,
            2.0 * q.x * q.y,
            2.0 * q.x * q.z,
            2.0 * q.x * q.w
        )
    }

    fn quat_mult_derivative(&self, q: &Vector4<f32>, dq: &Vector4<f32>) -> Vector4<f32> {
        Vector4::new(
            2.0 * (q.x * dq.x - q.y * dq.y - q.z * dq.z - q.w * dq.w),
            2.0 * (q.x * dq.y + q.y * dq.x),
            2.0 * (q.x * dq.z + q.z * dq.x),
            2.0 * (q.x * dq.w + q.w * dq.x)
        )
    }
}

// Kaleidoscopic IFS fractal
pub struct KaleidoIFS {
    pub fold_count: i32,
    pub scale: f32,
    pub time: f32,
}

impl FractalGenerator for KaleidoIFS {
    fn distance_estimator(&self, pos: &Vector4<f32>) -> f32 {
        let mut p = Vector3::new(pos.x, pos.y, pos.z);
        let mut scale = 1.0f32;

        for i in 0..self.fold_count {
            // Time-based folding planes
            let angle = self.time * 0.1 + i as f32 * 0.5;
            let fold_normal = Vector3::new(angle.cos(), angle.sin(), (angle * 1.3).sin());

            // Kaleidoscopic folding
            let dot = p.dot(&fold_normal);
            if dot < 0.0 {
                p = p - 2.0 * dot * fold_normal;
            }

            // Box folding
            p = p.map(|x| if x > 1.0 { 2.0 - x } else if x < -1.0 { -2.0 - x } else { x });

            // Spherical folding
            let r2 = p.norm_squared();
            if r2 < 0.25 {
                p = p * 4.0;
                scale *= 4.0;
            } else if r2 < 1.0 {
                p = p / r2;
                scale /= r2;
            }

            // Scale and translate
            let dynamic_scale = self.scale + (self.time * 0.05 + i as f32 * 0.1).sin() * 0.5;
            p = p * dynamic_scale + Vector3::new(
                (self.time * 0.07).sin() * 0.1,
                (self.time * 0.11).cos() * 0.1,
                pos.w * 0.2
            );
            scale *= dynamic_scale;
        }

        (p.norm() - 0.5) / scale.abs()
    }

    fn get_color(&self, iterations: i32, distance: f32, pos: &Vector4<f32>) -> Vector3<f32> {
        let complexity = (iterations as f32 + distance * 10.0) * 0.1;
        let hue = (complexity + self.time * 0.3 + pos.w).fract();
        let saturation = (1.0 - distance * 0.3).max(0.4);
        let value = (0.8 + (complexity * 3.0).sin() * 0.2).max(0.1);

        hsv_to_rgb(hue, saturation, value)
    }

    fn get_name(&self) -> &'static str { "KaleidoIFS" }
}

// Fractal selector based on seed
pub fn create_fractal_from_seed(seed: u32, time: f32) -> Box<dyn FractalGenerator> {
    let fractal_type = seed % 3;

    match fractal_type {
        0 => Box::new(Mandelbulb {
            power: 6.0 + ((seed / 3) % 8) as f32,
            iterations: 8 + ((seed / 24) % 4) as i32,
            time,
        }),
        1 => {
            let c_seed = seed / 100;
            Box::new(Julia4D {
                c: Vector4::new(
                    ((c_seed % 1000) as f32 / 1000.0 - 0.5) * 2.0,
                    (((c_seed / 1000) % 1000) as f32 / 1000.0 - 0.5) * 2.0,
                    (((c_seed / 1000000) % 1000) as f32 / 1000.0 - 0.5) * 2.0,
                    (((c_seed / 1000000000) % 1000) as f32 / 1000.0 - 0.5) * 2.0,
                ),
                iterations: 8 + ((seed / 13) % 6) as i32,
                time,
            })
        },
        _ => Box::new(KaleidoIFS {
            fold_count: 4 + ((seed / 7) % 8) as i32,
            scale: 1.5 + ((seed / 17) % 10) as f32 * 0.3,
            time,
        }),
    }
}

// HSV to RGB conversion for beautiful colors
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Vector3<f32> {
    let c = v * s;
    let h_prime = (h * 6.0) % 6.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Vector3::new(r + m, g + m, b + m)
}

// Audio synthesis from fractal geometry - thinking ahead
pub struct FractalAudioAnalyzer;

impl FractalAudioAnalyzer {
    pub fn extract_frequencies(fractal: &dyn FractalGenerator, sample_points: &[Vector4<f32>]) -> Vec<f32> {
        let mut frequencies = Vec::with_capacity(32);

        // Sample fractal at different points and convert to frequencies
        for (i, point) in sample_points.iter().enumerate() {
            let distance = fractal.distance_estimator(point);
            let color = fractal.get_color(8, distance, point);

            // Convert color and distance to musical frequencies
            let base_freq = 220.0; // A3
            let frequency = base_freq * (1.0 + distance * 0.5) * (1.0 + color.x * 0.3);
            frequencies.push(frequency);
        }

        frequencies
    }

    pub fn create_harmonic_series(fundamental: f32, harmonics: usize) -> Vec<f32> {
        (1..=harmonics).map(|h| fundamental * h as f32).collect()
    }
}