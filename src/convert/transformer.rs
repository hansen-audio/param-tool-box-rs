// Copyright(c) 2023 Hansen Audio.

#[derive(Debug, Clone)]
pub struct Transformer {
    min: f32,
    max: f32,
}

impl Transformer {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn min(&self) -> f32 {
        self.min
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn up_transform(&self, normalized: f32) -> f32 {
        normalized * (self.max - self.min) + self.min
    }

    pub fn down_transform(&self, physical: f32) -> f32 {
        (physical - self.min) / (self.max - self.min)
    }
}
