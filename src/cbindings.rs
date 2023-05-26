// Copyright(c) 2023 Hansen Audio.

// #[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Percent{
    min: f32,
    max: f32
}

impl Percent {
    pub fn new() -> Self { Self { min: 0., max: 100. } }
}

trait Transform {
    fn to_phyical(&self, normalized: f32) -> f32;
    fn to_normalized(&self, physical: f32) -> f32;
}

impl Transform for Percent {
    fn to_phyical(&self, normalized: f32) -> f32 {
        normalized * (self.max - self.min) + self.min
    }

    fn to_normalized(&self, physical: f32) -> f32 {
        (physical - self.min) / (self.max - self.min)
    }
}

#[no_mangle]
pub unsafe extern "C" fn to_physical(normalized: f32, percent: &mut Percent) -> f32{
    percent.to_phyical(normalized)
}