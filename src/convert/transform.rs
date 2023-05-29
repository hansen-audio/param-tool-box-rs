// Copyright(c) 2023 Hansen Audio.

pub trait Transform {
    fn max(&self) -> f32;
    fn min(&self) -> f32;
    fn to_physical(&self, normalized: f32) -> f32;
    fn to_normalized(&self, physical: f32) -> f32;
}

pub trait Display {
    fn to_display(&self, physical: f32, precision: Option<usize>) -> String;
    fn from_display(&self, physical: String) -> f32;
}
