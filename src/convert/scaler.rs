// Copyright(c) 2023 Hansen Audio.

#[derive(Debug, Clone)]
pub struct Scaler {
    a: Option<f32>,
}

impl Scaler {
    pub fn new(min: f32, max: f32, mid: Option<f32>) -> Self {
        Self {
            a: Self::calc_opt_transform_factor(min, max, mid),
        }
    }

    pub fn scale_up(&self, value: f32) -> f32 {
        match self.a {
            Some(a) => value / (value + a * (value - 1.0)),
            None => value,
        }
    }

    pub fn scale_down(&self, value: f32) -> f32 {
        match self.a {
            Some(a) => -1.0 / ((1.0 / value - 1.0) / a - 1.0),
            None => value,
        }
    }

    fn calc_opt_transform_factor(min: f32, max: f32, mid: Option<f32>) -> Option<f32> {
        let a = match mid {
            Some(opt_mid) => Some(Self::calc_transform_factor(min, max, opt_mid)),
            None => None,
        };
        a
    }

    fn calc_transform_factor(min: f32, max: f32, mid: f32) -> f32 {
        let y_norm = (mid - min) / (max - min);
        let x_norm = 0.5;
        let t = -1.0 / (((x_norm / y_norm - x_norm) / (x_norm - 1.0)) - 1.0);

        1. - (1. / t)
    }
}
