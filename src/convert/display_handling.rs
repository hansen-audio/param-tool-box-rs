// Copyright(c) 2023 Hansen Audio.

pub trait DisplayHandling {
    const DEFAULT_PRECISION: usize = 2;
    fn to_display(&self, physical: f32, precision: Option<usize>) -> String;
    fn from_display(&self, physical: String) -> f32;
}
