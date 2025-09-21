use wasm_bindgen::prelude::*;
use web_sys::{Storage, Window};
use nalgebra::Matrix4;
use serde::{Serialize, Deserialize};
use crate::fractals::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct FrozenFractal {
    pub seed: u32,
    pub fractal_type: String,
    pub transform_matrix: Vec<f32>, // 4x4 matrix flattened
    pub complexity_score: f32,
    pub timestamp: u64,
    pub interaction_count: u32,
}

#[derive(Serialize, Deserialize)]
pub struct BattleResult {
    pub winner: FrozenFractal,
    pub score_self: f32,
    pub score_opponent: f32,
    pub resonance_factor: f32,
}

pub struct UserState {
    user_id: String,
    current_seed: u32,
    current_transform: Matrix4<f32>,
    daily_interactions: u32,
    storage: Storage,
    frozen_fractals: Vec<FrozenFractal>,
}

impl UserState {
    pub fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("No window available")?;
        let storage = window.local_storage()?.ok_or("No localStorage available")?;

        // Get or create user ID
        let user_id = Self::get_or_create_user_id(&storage)?;

        // Generate today's seed
        let current_seed = Self::generate_daily_seed(&user_id);

        // Load or initialize transform
        let current_transform = Self::load_transform(&storage, current_seed)?;

        // Load interaction count
        let daily_interactions = Self::load_daily_interactions(&storage)?;

        // Load frozen fractals
        let frozen_fractals = Self::load_frozen_fractals(&storage)?;

        Ok(UserState {
            user_id,
            current_seed,
            current_transform,
            daily_interactions,
            storage,
            frozen_fractals,
        })
    }

    fn get_or_create_user_id(storage: &Storage) -> Result<String, JsValue> {
        match storage.get_item("resonant_user_id")? {
            Some(id) => Ok(id),
            None => {
                // Generate unique user ID
                let id = format!("user_{}", js_sys::Math::random() * 1000000.0);
                storage.set_item("resonant_user_id", &id)?;
                Ok(id)
            }
        }
    }

    fn generate_daily_seed(user_id: &str) -> u32 {
        let date = js_sys::Date::new_0();
        let day_code = (date.get_full_year() * 10000 + date.get_month() * 100 + date.get_date()) as u32;

        // Simple hash of user_id + date
        let mut hash = 0u32;
        for byte in user_id.bytes().chain(day_code.to_string().bytes()) {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }

    fn load_transform(storage: &Storage, seed: u32) -> Result<Matrix4<f32>, JsValue> {
        let key = format!("resonant_transform_{}", seed);
        match storage.get_item(&key)? {
            Some(data) => {
                let matrix_data: Vec<f32> = serde_json::from_str(&data).unwrap_or_default();
                if matrix_data.len() == 16 {
                    Ok(Matrix4::from_row_slice(&matrix_data))
                } else {
                    Ok(Matrix4::identity())
                }
            }
            None => Ok(Matrix4::identity()),
        }
    }

    fn load_daily_interactions(storage: &Storage) -> Result<u32, JsValue> {
        let today = Self::get_date_string();
        let key = format!("resonant_interactions_{}", today);
        match storage.get_item(&key)? {
            Some(count_str) => Ok(count_str.parse().unwrap_or(0)),
            None => Ok(0),
        }
    }

    fn load_frozen_fractals(storage: &Storage) -> Result<Vec<FrozenFractal>, JsValue> {
        match storage.get_item("resonant_frozen_fractals")? {
            Some(data) => Ok(serde_json::from_str(&data).unwrap_or_default()),
            None => Ok(Vec::new()),
        }
    }

    fn get_date_string() -> String {
        let date = js_sys::Date::new_0();
        format!("{}-{:02}-{:02}",
            date.get_full_year(),
            date.get_month() + 1,
            date.get_date()
        )
    }

    pub fn get_current_fractal(&self, time: f32) -> Box<dyn FractalGenerator> {
        create_fractal_from_seed(self.current_seed, time)
    }

    pub fn get_seed(&self) -> u32 {
        self.current_seed
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    pub fn get_current_transform(&self) -> Matrix4<f32> {
        self.current_transform
    }

    pub fn get_complexity_score(&self) -> f32 {
        // Calculate complexity based on transform matrix and interactions
        let matrix_complexity = self.current_transform.determinant().abs().ln().max(0.0);
        let interaction_bonus = (self.daily_interactions as f32).sqrt() * 0.1;
        matrix_complexity + interaction_bonus
    }

    pub fn get_interaction_count(&self) -> u32 {
        self.daily_interactions
    }

    pub fn apply_transform(&mut self, transform: Matrix4<f32>) {
        // Accumulate transform
        self.current_transform = self.current_transform * transform;

        // Increment interaction count
        self.daily_interactions += 1;

        // Save to storage
        let _ = self.save_state();
    }

    fn save_state(&self) -> Result<(), JsValue> {
        // Save transform
        let transform_key = format!("resonant_transform_{}", self.current_seed);
        let matrix_data: Vec<f32> = self.current_transform.as_slice().to_vec();
        let transform_json = serde_json::to_string(&matrix_data).unwrap();
        self.storage.set_item(&transform_key, &transform_json)?;

        // Save interaction count
        let today = Self::get_date_string();
        let interactions_key = format!("resonant_interactions_{}", today);
        self.storage.set_item(&interactions_key, &self.daily_interactions.to_string())?;

        // Save frozen fractals
        let frozen_json = serde_json::to_string(&self.frozen_fractals).unwrap();
        self.storage.set_item("resonant_frozen_fractals", &frozen_json)?;

        Ok(())
    }

    pub fn freeze_current_fractal(&mut self, fractal_type: String) -> Result<FrozenFractal, JsValue> {
        let frozen = FrozenFractal {
            seed: self.current_seed,
            fractal_type,
            transform_matrix: self.current_transform.as_slice().to_vec(),
            complexity_score: self.get_complexity_score(),
            timestamp: js_sys::Date::now() as u64,
            interaction_count: self.daily_interactions,
        };

        // Only keep the best 10 frozen fractals
        self.frozen_fractals.push(frozen.clone());
        self.frozen_fractals.sort_by(|a, b| b.complexity_score.partial_cmp(&a.complexity_score).unwrap());
        if self.frozen_fractals.len() > 10 {
            self.frozen_fractals.truncate(10);
        }

        self.save_state()?;
        Ok(frozen)
    }

    pub fn battle_against_fractal(&self, opponent_json: &str) -> Result<BattleResult, JsValue> {
        let opponent: FrozenFractal = serde_json::from_str(opponent_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Create current fractal for battle
        let current = FrozenFractal {
            seed: self.current_seed,
            fractal_type: "Current".to_string(),
            transform_matrix: self.current_transform.as_slice().to_vec(),
            complexity_score: self.get_complexity_score(),
            timestamp: js_sys::Date::now() as u64,
            interaction_count: self.daily_interactions,
        };

        // Battle algorithm: complexity + resonance
        let self_score = current.complexity_score + self.calculate_resonance(&current, &opponent);
        let opponent_score = opponent.complexity_score + self.calculate_resonance(&opponent, &current);

        let resonance_factor = self.calculate_resonance(&current, &opponent);

        let winner = if self_score > opponent_score {
            current
        } else {
            opponent
        };

        Ok(BattleResult {
            winner,
            score_self: self_score,
            score_opponent: opponent_score,
            resonance_factor,
        })
    }

    fn calculate_resonance(&self, fractal_a: &FrozenFractal, fractal_b: &FrozenFractal) -> f32 {
        // Calculate mathematical resonance between two fractals
        let matrix_a = Matrix4::from_row_slice(&fractal_a.transform_matrix);
        let matrix_b = Matrix4::from_row_slice(&fractal_b.transform_matrix);

        // Calculate trace similarity (simplified resonance measure)
        let trace_a = matrix_a.trace();
        let trace_b = matrix_b.trace();
        let trace_similarity = 1.0 / (1.0 + (trace_a - trace_b).abs());

        // Seed harmony (how well seeds work together)
        let seed_diff = (fractal_a.seed as f32 - fractal_b.seed as f32).abs();
        let seed_harmony = 1.0 / (1.0 + seed_diff / 1000.0);

        (trace_similarity + seed_harmony) * 0.5
    }

    pub fn get_best_frozen_fractal(&self) -> Option<&FrozenFractal> {
        self.frozen_fractals.first()
    }

    pub fn reset_daily_state(&mut self) -> Result<(), JsValue> {
        // Reset for new day
        self.current_seed = Self::generate_daily_seed(&self.user_id);
        self.current_transform = Matrix4::identity();
        self.daily_interactions = 0;
        self.save_state()
    }
}