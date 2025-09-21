use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use crate::user::FrozenFractal;

#[derive(Serialize, Deserialize, Clone)]
pub struct FractalMessage {
    pub sender_id: String,
    pub fractal_data: FrozenFractal,
    pub transform_echo: Option<Vec<f32>>, // 4x4 matrix if this is a response
    pub timestamp: u64,
    pub message_type: MessageType,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum MessageType {
    Morning,     // Daily fractal share
    Echo,        // Response with transform
    Battle,      // Challenge for fractal battle
    Resonance,   // Special resonance moment
}

#[derive(Serialize, Deserialize)]
pub struct NetworkState {
    pub connected_peers: Vec<String>,
    pub pending_messages: Vec<FractalMessage>,
    pub last_sync: u64,
}

pub struct NetworkManager {
    user_id: String,
    connection_state: NetworkState,
}

impl NetworkManager {
    pub fn new(user_id: String) -> Self {
        NetworkManager {
            user_id,
            connection_state: NetworkState {
                connected_peers: Vec::new(),
                pending_messages: Vec::new(),
                last_sync: js_sys::Date::now() as u64,
            },
        }
    }

    // Generate shareable URL with embedded fractal data
    pub fn create_share_url(&self, fractal: &FrozenFractal, domain: &str) -> String {
        let encoded_data = self.encode_fractal_for_url(fractal);
        format!("{}?f={}&from={}", domain, encoded_data, self.user_id)
    }

    // Decode fractal from URL parameter
    pub fn decode_share_url(&self, url_param: &str) -> Result<FrozenFractal, JsValue> {
        let decoded = self.decode_fractal_from_url(url_param)?;
        Ok(decoded)
    }

    // Simplified URL encoding for fractal data
    fn encode_fractal_for_url(&self, fractal: &FrozenFractal) -> String {
        // Create compact representation
        let compact = CompactFractal {
            seed: fractal.seed,
            fractal_type: match fractal.fractal_type.as_str() {
                "Mandelbulb" => 0,
                "Julia4D" => 1,
                "KaleidoIFS" => 2,
                _ => 0,
            },
            complexity: (fractal.complexity_score * 100.0) as u16,
            interactions: fractal.interaction_count.min(255) as u8,
        };

        // Convert to base64
        let json = serde_json::to_string(&compact).unwrap();
        base64_encode(&json)
    }

    fn decode_fractal_from_url(&self, encoded: &str) -> Result<FrozenFractal, JsValue> {
        let json = base64_decode(encoded)
            .map_err(|_| JsValue::from_str("Invalid fractal data"))?;

        let compact: CompactFractal = serde_json::from_str(&json)
            .map_err(|_| JsValue::from_str("Invalid fractal format"))?;

        Ok(FrozenFractal {
            seed: compact.seed,
            fractal_type: match compact.fractal_type {
                0 => "Mandelbulb".to_string(),
                1 => "Julia4D".to_string(),
                2 => "KaleidoIFS".to_string(),
                _ => "Mandelbulb".to_string(),
            },
            transform_matrix: vec![1.0; 16], // Default identity matrix
            complexity_score: compact.complexity as f32 / 100.0,
            timestamp: js_sys::Date::now() as u64,
            interaction_count: compact.interactions as u32,
        })
    }

    // Send fractal to friends (placeholder for future P2P)
    pub fn broadcast_morning_fractal(&mut self, fractal: &FrozenFractal) -> Result<(), JsValue> {
        let message = FractalMessage {
            sender_id: self.user_id.clone(),
            fractal_data: fractal.clone(),
            transform_echo: None,
            timestamp: js_sys::Date::now() as u64,
            message_type: MessageType::Morning,
        };

        // For now, just store in pending messages
        // In future: send via WebRTC or WebSocket
        self.connection_state.pending_messages.push(message);

        // Log to console for debugging
        web_sys::console::log_1(&JsValue::from_str(&format!(
            "Broadcasting morning fractal: seed={}", fractal.seed
        )));

        Ok(())
    }

    // Respond to received fractal with transform
    pub fn send_echo_response(&mut self, original_fractal: &FrozenFractal,
                             transform_matrix: &[f32]) -> Result<(), JsValue> {
        let message = FractalMessage {
            sender_id: self.user_id.clone(),
            fractal_data: original_fractal.clone(),
            transform_echo: Some(transform_matrix.to_vec()),
            timestamp: js_sys::Date::now() as u64,
            message_type: MessageType::Echo,
        };

        self.connection_state.pending_messages.push(message);

        web_sys::console::log_1(&JsValue::from_str(&format!(
            "Sending echo response to fractal: seed={}", original_fractal.seed
        )));

        Ok(())
    }

    // Check for resonance moments (when multiple people are active)
    pub fn check_resonance_window(&self) -> bool {
        let now = js_sys::Date::now() as u64;
        let time_window = 300_000; // 5 minutes in milliseconds

        // Check if there are recent messages from multiple users
        let recent_senders: std::collections::HashSet<String> = self.connection_state
            .pending_messages
            .iter()
            .filter(|msg| now - msg.timestamp < time_window)
            .map(|msg| msg.sender_id.clone())
            .collect();

        recent_senders.len() >= 2
    }

    // Generate time-limited share token
    pub fn create_temporary_share_token(&self, fractal: &FrozenFractal,
                                       duration_hours: u32) -> String {
        let expires = js_sys::Date::now() as u64 + (duration_hours as u64 * 3600 * 1000);

        let token_data = ShareToken {
            fractal_seed: fractal.seed,
            expires,
            creator: self.user_id.clone(),
        };

        let json = serde_json::to_string(&token_data).unwrap();
        base64_encode(&json)
    }

    // Validate and use share token
    pub fn validate_share_token(&self, token: &str) -> Result<u32, JsValue> {
        let json = base64_decode(token)
            .map_err(|_| JsValue::from_str("Invalid token"))?;

        let token_data: ShareToken = serde_json::from_str(&json)
            .map_err(|_| JsValue::from_str("Invalid token format"))?;

        let now = js_sys::Date::now() as u64;
        if now > token_data.expires {
            return Err(JsValue::from_str("Token expired"));
        }

        Ok(token_data.fractal_seed)
    }

    pub fn get_pending_messages(&self) -> &[FractalMessage] {
        &self.connection_state.pending_messages
    }

    pub fn clear_old_messages(&mut self, max_age_hours: u32) {
        let cutoff = js_sys::Date::now() as u64 - (max_age_hours as u64 * 3600 * 1000);
        self.connection_state.pending_messages.retain(|msg| msg.timestamp > cutoff);
    }
}

#[derive(Serialize, Deserialize)]
struct CompactFractal {
    seed: u32,
    fractal_type: u8,
    complexity: u16,
    interactions: u8,
}

#[derive(Serialize, Deserialize)]
struct ShareToken {
    fractal_seed: u32,
    expires: u64,
    creator: String,
}

// Simple base64 encoding/decoding for URL safety
fn base64_encode(data: &str) -> String {
    // Simplified base64 implementation for demonstration
    // In production, use a proper base64 library or browser API
    js_sys::encode_uri_component(data).as_string().unwrap()
}

fn base64_decode(encoded: &str) -> Result<String, ()> {
    // Simplified decode - in production use proper base64
    match js_sys::decode_uri_component(encoded) {
        Ok(decoded) => Ok(decoded.as_string().unwrap()),
        Err(_) => Err(()),
    }
}