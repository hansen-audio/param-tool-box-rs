// Copyright(c) 2023 Hansen Audio.

#[derive(Debug, Clone)]
pub struct Transformer {
    min: f32,
    max: f32,
    range: f32,
    range_inv: f32,
}

impl Transformer {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            range: max - min,
            range_inv: 1. / (max - min),
        }
    }

    pub fn min(&self) -> f32 {
        self.min
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn up_transform(&self, normalized: f32) -> f32 {
        normalized * self.range + self.min
    }

    pub fn down_transform(&self, physical: f32) -> f32 {
        (physical - self.min) * self.range_inv
    }
}
