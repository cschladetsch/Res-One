use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, AudioNode, OscillatorNode, GainNode, AudioDestinationNode};

pub struct AudioEngine {
    context: AudioContext,
    oscillators: Vec<OscillatorNode>,
    gain_nodes: Vec<GainNode>,
    master_gain: GainNode,
    current_frequencies: Vec<f32>,
    is_playing: bool,
}

impl AudioEngine {
    pub fn new() -> Result<Self, JsValue> {
        // Create audio context
        let context = AudioContext::new()?;

        // Create master gain node
        let master_gain = context.create_gain()?;
        master_gain.connect_with_audio_node(&context.destination())?;
        master_gain.gain().set_value(0.1); // Start quiet

        Ok(AudioEngine {
            context,
            oscillators: Vec::new(),
            gain_nodes: Vec::new(),
            master_gain,
            current_frequencies: Vec::new(),
            is_playing: false,
        })
    }

    pub fn update_frequencies(&mut self, frequencies: &[f32]) {
        // Only update if frequencies have changed significantly
        if self.frequencies_changed(frequencies) {
            self.current_frequencies = frequencies.to_vec();
            self.restart_oscillators();
        }
    }

    fn frequencies_changed(&self, new_frequencies: &[f32]) -> bool {
        if self.current_frequencies.len() != new_frequencies.len() {
            return true;
        }

        self.current_frequencies.iter()
            .zip(new_frequencies.iter())
            .any(|(old, new)| (old - new).abs() > 5.0) // 5Hz threshold
    }

    fn restart_oscillators(&mut self) {
        // Stop existing oscillators
        self.stop_all();

        // Clear vectors
        self.oscillators.clear();
        self.gain_nodes.clear();

        // Create new oscillators for current frequencies
        for (i, &frequency) in self.current_frequencies.iter().enumerate() {
            if let Ok((osc, gain)) = self.create_oscillator(frequency, i) {
                self.oscillators.push(osc);
                self.gain_nodes.push(gain);
            }
        }

        self.is_playing = true;
    }

    fn create_oscillator(&self, frequency: f32, index: usize) -> Result<(OscillatorNode, GainNode), JsValue> {
        let oscillator = self.context.create_oscillator()?;
        let gain = self.context.create_gain()?;

        // Set frequency (clamp to audible range)
        let clamped_freq = frequency.max(80.0).min(2000.0);
        oscillator.frequency().set_value(clamped_freq);

        // Use default sine wave for now
        // TODO: Fix waveform setting when Web API is updated

        // Set gain based on frequency (lower frequencies louder)
        let gain_value = (1.0 / (1.0 + frequency / 400.0)) * 0.1;
        gain.gain().set_value(gain_value);

        // Connect oscillator -> gain -> master gain
        oscillator.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&self.master_gain)?;

        // Start oscillator
        oscillator.start()?;

        Ok((oscillator, gain))
    }

    pub fn play_gesture_feedback(&self, gesture_type: &str, intensity: f32) -> Result<(), JsValue> {
        // Create a short feedback sound based on gesture
        let oscillator = self.context.create_oscillator()?;
        let gain = self.context.create_gain()?;

        let frequency = match gesture_type {
            "swipe" => 440.0 + intensity * 200.0,
            "pinch" => 880.0 - intensity * 400.0,
            "tilt" => 330.0 + intensity * 100.0,
            "smile" => 550.0 + intensity * 300.0,
            _ => 440.0,
        };

        oscillator.frequency().set_value(frequency);
        // Using default sine wave

        // Quick envelope: attack -> decay
        let now = self.context.current_time();
        gain.gain().set_value(0.0);
        gain.gain().linear_ramp_to_value_at_time(intensity * 0.2, now + 0.05)?;
        gain.gain().linear_ramp_to_value_at_time(0.0, now + 0.3)?;

        oscillator.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&self.master_gain)?;

        oscillator.start()?;
        oscillator.stop_with_when(now + 0.3)?;

        Ok(())
    }

    pub fn get_current_frequencies(&self) -> Vec<f32> {
        self.current_frequencies.clone()
    }

    pub fn set_master_volume(&self, volume: f32) {
        self.master_gain.gain().set_value(volume.max(0.0).min(1.0));
    }

    pub fn stop_all(&mut self) {
        for oscillator in &self.oscillators {
            let _ = oscillator.stop();
        }
        self.is_playing = false;
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stop_all();
    }
}