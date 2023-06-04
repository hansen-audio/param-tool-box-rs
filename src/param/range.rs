// Copyright(c) 2023 Hansen Audio.

#[derive(Debug, Clone, Copy)]
pub struct Range {
    min: f32,
    max: f32,
    range: f32,
    range_inv: f32,
    is_inverted: bool,
}

impl Range {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            range: max - min,
            range_inv: Self::invert(min, max),
            is_inverted: max < min,
        }
    }

    fn invert(min: f32, max: f32) -> f32 {
        1. / (max - min)
    }

    pub fn min(&self) -> f32 {
        self.min
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn range(&self) -> f32 {
        self.range
    }

    pub fn range_inv(&self) -> f32 {
        self.range_inv
    }

    pub fn is_inverted(&self) -> bool {
        self.is_inverted
    }
}
