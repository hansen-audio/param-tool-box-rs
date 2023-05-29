// Copyright(c) 2023 Hansen Audio.

pub trait Transform {
    fn max(&self) -> f32;
    fn min(&self) -> f32;
    fn to_physical(&self, val: f32) -> f32;
    fn to_normalized(&self, val: f32) -> f32;
}

pub trait Display {
    fn to_display(&self, value: f32, precision: Option<usize>) -> String;
    fn from_display(&self, value: String) -> f32;
}
