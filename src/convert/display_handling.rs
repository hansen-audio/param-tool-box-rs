// Copyright(c) 2023 Hansen Audio.

pub trait DisplayHandling {
    fn to_display(&self, physical: f32, precision: Option<usize>) -> String;
    fn from_display(&self, physical: String) -> f32;
}
