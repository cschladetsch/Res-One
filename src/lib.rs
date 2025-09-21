mod fractals;
mod audio;
mod user;
mod network;

use wasm_bindgen::prelude::*;
use web_sys::{WebGlRenderingContext as GL, WebGlProgram, WebGlShader};
use nalgebra::Vector4;
use fractals::*;
use audio::AudioEngine;
use user::UserState;

#[wasm_bindgen]
pub struct Resonant {
    gl: GL,
    program: WebGlProgram,
    user_state: UserState,
    audio_engine: AudioEngine,
    time: f32,
    fractal_type: String,
    last_wake_time: f64,
}

#[wasm_bindgen]
impl Resonant {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<Resonant, JsValue> {
        // Setup panic hook for better debugging
        console_error_panic_hook::set_once();

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into()?;

        // Setup WebGL with better error handling
        let gl = canvas
            .get_context("webgl")?
            .ok_or("WebGL not supported")?
            .dyn_into::<GL>()?;

        // Enable depth testing for better 3D rendering
        gl.enable(GL::DEPTH_TEST);
        gl.depth_func(GL::LEQUAL);

        let program = Self::create_shader_program(&gl)?;

        // Initialize user state with persistence
        let user_state = UserState::new()?;

        // Initialize audio engine
        let audio_engine = AudioEngine::new()?;

        // Detect if this is a wake-up or just app open
        let last_wake_time = Self::detect_wake_time();

        Ok(Resonant {
            gl,
            program,
            user_state,
            audio_engine,
            time: 0.0,
            fractal_type: "Unknown".to_string(),
            last_wake_time,
        })
    }

    pub fn render(&mut self, delta_time: f32) {
        self.time += delta_time * 0.001;

        // Get today's fractal based on user ID + date + wake time
        let current_fractal = self.user_state.get_current_fractal(self.time);
        self.fractal_type = current_fractal.get_name().to_string();

        let gl = &self.gl;
        gl.clear_color(0.0, 0.0, 0.02, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        gl.use_program(Some(&self.program));

        // Setup uniforms
        self.setup_uniforms(&*current_fractal);

        // Draw fullscreen quad with vertices
        self.draw_quad();

        // Update audio based on fractal state
        self.update_audio(&*current_fractal);
    }

    fn setup_uniforms(&self, fractal: &dyn FractalGenerator) {
        let gl = &self.gl;

        // Time uniform
        if let Some(loc) = gl.get_uniform_location(&self.program, "u_time") {
            gl.uniform1f(Some(&loc), self.time);
        }

        // Seed uniform
        if let Some(loc) = gl.get_uniform_location(&self.program, "u_seed") {
            gl.uniform1i(Some(&loc), self.user_state.get_seed() as i32);
        }

        // Fractal type uniform
        if let Some(loc) = gl.get_uniform_location(&self.program, "u_fractal_type") {
            let fractal_id = match fractal.get_name() {
                "Mandelbulb" => 0,
                "Julia4D" => 1,
                "KaleidoIFS" => 2,
                _ => 0,
            };
            gl.uniform1i(Some(&loc), fractal_id);
        }

        // Transform matrix from user interactions
        if let Some(loc) = gl.get_uniform_location(&self.program, "u_transform") {
            let transform = self.user_state.get_current_transform();
            let matrix_array: [f32; 16] = transform.as_slice().try_into().unwrap_or([
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ]);
            gl.uniform_matrix4fv_with_f32_array(Some(&loc), false, &matrix_array);
        }
    }

    fn draw_quad(&self) {
        // For now, use the built-in triangle strip
        // Later we'll add proper vertex buffers for mobile optimization
        self.gl.draw_arrays(GL::TRIANGLE_STRIP, 0, 4);
    }

    fn update_audio(&mut self, fractal: &dyn FractalGenerator) {
        // Sample fractal at key points for audio generation
        let sample_points = vec![
            Vector4::new(1.0, 0.0, 0.0, self.time * 0.1),
            Vector4::new(0.0, 1.0, 0.0, self.time * 0.13),
            Vector4::new(0.0, 0.0, 1.0, self.time * 0.17),
            Vector4::new((self.time * 0.1).sin(), (self.time * 0.1).cos(), 0.0, 0.5),
        ];

        let frequencies = FractalAudioAnalyzer::extract_frequencies(fractal, &sample_points);
        self.audio_engine.update_frequencies(&frequencies);
    }

    pub fn apply_gesture(&mut self, gesture_type: &str, intensity: f32, direction: f32) -> Result<(), JsValue> {
        // Convert gesture to mathematical transform
        let transform = match gesture_type {
            "swipe" => self.create_rotation_transform(direction, intensity),
            "pinch" => self.create_scale_transform(intensity),
            "tilt" => self.create_tilt_transform(direction, intensity),
            "smile" => self.create_brightness_transform(intensity),
            _ => return Ok(()),
        };

        self.user_state.apply_transform(transform);

        // Trigger audio feedback
        self.audio_engine.play_gesture_feedback(gesture_type, intensity)?;

        Ok(())
    }

    fn create_rotation_transform(&self, direction: f32, intensity: f32) -> nalgebra::Matrix4<f32> {
        use nalgebra::Matrix4;
        let angle = direction * intensity * 0.1;
        Matrix4::new_rotation(nalgebra::Vector3::new(0.0, 0.0, angle))
    }

    fn create_scale_transform(&self, intensity: f32) -> nalgebra::Matrix4<f32> {
        use nalgebra::Matrix4;
        let scale = 1.0 + intensity * 0.2;
        Matrix4::new_scaling(scale)
    }

    fn create_tilt_transform(&self, direction: f32, intensity: f32) -> nalgebra::Matrix4<f32> {
        use nalgebra::Matrix4;
        let angle = direction * intensity * 0.05;
        Matrix4::new_rotation(nalgebra::Vector3::new(angle, 0.0, 0.0))
    }

    fn create_brightness_transform(&self, intensity: f32) -> nalgebra::Matrix4<f32> {
        use nalgebra::Matrix4;
        // Brightness affects the 4th dimension
        Matrix4::new_translation(&nalgebra::Vector3::new(0.0, 0.0, intensity * 0.1))
    }

    pub fn get_share_url(&self) -> String {
        format!("{}?seed={}&user={}&time={}",
            "https://resonant.app",
            self.user_state.get_seed(),
            self.user_state.get_user_id(),
            self.last_wake_time as u64
        )
    }

    pub fn get_fractal_info(&self) -> String {
        serde_json::json!({
            "type": self.fractal_type,
            "seed": self.user_state.get_seed(),
            "complexity": self.user_state.get_complexity_score(),
            "interactions_today": self.user_state.get_interaction_count(),
            "audio_frequencies": self.audio_engine.get_current_frequencies()
        }).to_string()
    }

    pub fn freeze_fractal(&mut self) -> Result<String, JsValue> {
        let frozen = self.user_state.freeze_current_fractal(self.fractal_type.clone())?;
        Ok(serde_json::to_string(&frozen).unwrap())
    }

    pub fn battle_fractals(&self, opponent_data: &str) -> Result<String, JsValue> {
        let result = self.user_state.battle_against_fractal(opponent_data)?;
        Ok(serde_json::to_string(&result).unwrap())
    }

    fn detect_wake_time() -> f64 {
        // Simple heuristic: if it's been more than 4 hours since last activity,
        // this is probably a wake-up
        js_sys::Date::now()
    }

    fn create_shader_program(gl: &GL) -> Result<WebGlProgram, JsValue> {
        let vert_shader = Self::compile_shader(gl, GL::VERTEX_SHADER, VERTEX_SHADER)?;
        let frag_shader = Self::compile_shader(gl, GL::FRAGMENT_SHADER, FRAGMENT_SHADER)?;

        let program = gl.create_program().ok_or("Failed to create program")?;
        gl.attach_shader(&program, &vert_shader);
        gl.attach_shader(&program, &frag_shader);
        gl.link_program(&program);

        if !gl.get_program_parameter(&program, GL::LINK_STATUS).as_bool().unwrap_or(false) {
            return Err(JsValue::from_str(&format!(
                "Shader program link failed: {}",
                gl.get_program_info_log(&program).unwrap_or_default()
            )));
        }

        Ok(program)
    }

    fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, JsValue> {
        let shader = gl.create_shader(shader_type).ok_or("Unable to create shader")?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        if !gl.get_shader_parameter(&shader, GL::COMPILE_STATUS).as_bool().unwrap_or(false) {
            return Err(JsValue::from_str(&format!(
                "Shader compile failed: {}",
                gl.get_shader_info_log(&shader).unwrap_or_default()
            )));
        }

        Ok(shader)
    }
}


const VERTEX_SHADER: &str = r#"
attribute vec2 a_position;
void main() {
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
precision mediump float;

uniform float u_time;
uniform int u_seed;
uniform int u_fractal_type;
uniform mat4 u_transform;

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

float mandelbulb(vec3 pos, float time, float seed) {
    vec3 z = pos;
    float dr = 1.0;
    float r = 0.0;
    float power = 6.0 + sin(time * 0.1 + seed * 0.001) * 3.0;

    for(int i = 0; i < 10; i++) {
        r = length(z);
        if(r > 2.0) break;

        float theta = acos(z.z/r) + time * 0.05;
        float phi = atan(z.y, z.x) + seed * 0.001;
        dr = pow(r, power - 1.0) * power * dr + 1.0;

        float zr = pow(r, power);
        z = zr * vec3(
            sin(theta) * cos(phi),
            sin(theta) * sin(phi),
            cos(theta)
        ) + pos;
    }

    return 0.5 * log(r) * r / dr;
}

float julia4d(vec3 pos, float time, float seed) {
    vec4 z = vec4(pos, sin(time * 0.1) * 0.5);
    vec4 c = vec4(
        sin(seed * 0.001) * 0.7,
        cos(seed * 0.0013) * 0.5,
        sin(time * 0.1 + seed * 0.002) * 0.3,
        cos(time * 0.07 + seed * 0.0017) * 0.4
    );

    for(int i = 0; i < 8; i++) {
        if(dot(z, z) > 4.0) break;

        float x = z.x * z.x - z.y * z.y - z.z * z.z - z.w * z.w + c.x;
        float y = 2.0 * z.x * z.y + c.y;
        float zz = 2.0 * z.x * z.z + c.z;
        float w = 2.0 * z.x * z.w + c.w;
        z = vec4(x, y, zz, w);
    }

    return length(z.xyz) - 1.0;
}

float kaleidoIFS(vec3 pos, float time, float seed) {
    vec3 p = pos;
    float scale = 1.0;

    for(int i = 0; i < 5; i++) {
        float angle = time * 0.1 + float(i) * 0.5 + seed * 0.01;
        vec3 n = normalize(vec3(cos(angle), sin(angle), sin(angle * 1.3)));

        float d = dot(p, n);
        if(d < 0.0) p -= 2.0 * d * n;

        p = clamp(p, -1.0, 1.0) * 2.0 - p;

        float r2 = dot(p, p);
        if(r2 < 0.25) {
            p *= 4.0;
            scale *= 4.0;
        } else if(r2 < 1.0) {
            p /= r2;
            scale /= r2;
        }

        float s = 1.6 + sin(time * 0.05 + float(i) * 0.1) * 0.2;
        p = p * s;
        scale *= s;
    }

    return (length(p) - 0.5) / abs(scale);
}

void main() {
    vec2 resolution = vec2(640.0, 480.0);
    vec2 uv = (gl_FragCoord.xy - 0.5 * resolution) / min(resolution.x, resolution.y);

    vec3 ray_origin = vec3(uv * 2.5, -4.0);
    vec3 ray_dir = normalize(vec3(uv * 0.6, 1.0));

    float t = 0.0;
    vec3 color = vec3(0.0);
    float seed = float(u_seed);
    int steps = 0;

    for(int i = 0; i < 80; i++) {
        vec3 pos = ray_origin + ray_dir * t;
        float dist = 0.0;

        if(u_fractal_type == 0) {
            dist = mandelbulb(pos, u_time, seed);
        } else if(u_fractal_type == 1) {
            dist = julia4d(pos, u_time, seed);
        } else {
            dist = kaleidoIFS(pos, u_time, seed);
        }

        if(dist < 0.002) {
            float glow = float(steps) / 80.0;

            vec3 baseColor = vec3(0.5);
            if(u_fractal_type == 0) {
                float hue = sin(glow * 3.14 + u_time * 0.2 + seed * 0.01) * 0.3 + 0.1;
                baseColor = hsv2rgb(vec3(hue, 0.8, 0.9));
            } else if(u_fractal_type == 1) {
                float hue = cos(glow * 2.0 + u_time * 0.3) * 0.3 + 0.6;
                baseColor = hsv2rgb(vec3(hue, 0.7, 0.8));
            } else {
                float hue = fract(glow * 2.0 + u_time * 0.1 + length(pos) * 0.1);
                baseColor = hsv2rgb(vec3(hue, 0.9, 0.9));
            }

            float lighting = 0.7 + 0.3 * sin(glow * 10.0);
            color = baseColor * lighting * (1.0 - glow * 0.5);
            break;
        }

        t += dist * 0.9;
        steps = i;
        if(t > 15.0) break;
    }

    if(length(color) < 0.01) {
        float bg = length(uv) * 0.1;
        color = vec3(bg * 0.05, bg * 0.1, bg * 0.2);
    }

    gl_FragColor = vec4(color, 1.0);
}
"#;