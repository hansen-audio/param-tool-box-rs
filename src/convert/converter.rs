// Copyright(c) 2023 Hansen Audio.

use crate::convert::{display_handling::DisplayHandling, scaler::Scaler, transformer::Transformer};

#[derive(Debug, Clone)]
pub struct Converter {
    transformer: Transformer,
    scaler: Scaler,
    is_int: bool,
}

impl Converter {
    pub fn new(min: f32, max: f32, mid: Option<f32>, is_int: bool) -> Self {
        Self {
            transformer: Transformer::new(min, max),
            scaler: Scaler::new(min, max, mid),
            is_int,
        }
    }

    pub fn is_int(&self) -> bool {
        self.is_int
    }
}

impl Converter {
    pub fn to_physical(&self, normalized: f32) -> f32 {
        let mut tmp_normalized = normalized.clamp(0., 1.);
        tmp_normalized = self.scaler.scale_up(tmp_normalized);

        let physical = self.transformer.up_transform(tmp_normalized);
        physical.round_cond(self.is_int)
    }

    pub fn to_normalized(&self, physical: f32) -> f32 {
        let tmp_physical = physical.clamp_inv(self.transformer.min(), self.transformer.max());

        let normalized = self
            .transformer
            .down_transform(tmp_physical.round_cond(self.is_int));

        self.scaler.scale_down(normalized)
    }

    pub fn min(&self) -> f32 {
        self.transformer.min()
    }

    pub fn max(&self) -> f32 {
        self.transformer.max()
    }
}

impl DisplayHandling for Converter {
    fn to_display(&self, physical: f32, precision: Option<usize>) -> String {
        let clamped_val = physical.clamp_inv(self.min(), self.max());

        const DEFAULT_PRECISION: usize = 2;
        format!(
            "{1:.0$}",
            precision.unwrap_or(DEFAULT_PRECISION),
            clamped_val
        )
    }

    fn from_display(&self, physical: String) -> f32 {
        let value = physical.parse::<f32>();

        match value {
            Ok(val) => val,
            Err(_) => self.min(),
        }
    }
}

trait F32Extra {
    fn clamp_inv(&self, min: f32, max: f32) -> f32;
    fn round_cond(&self, yes: bool) -> f32;
}

impl F32Extra for f32 {
    fn clamp_inv(&self, min: f32, max: f32) -> f32 {
        let is_inverted = max < min;
        return if is_inverted {
            self.clamp(max, min)
        } else {
            self.clamp(min, max)
        };
    }

    fn round_cond(&self, yes: bool) -> f32 {
        if yes {
            self.round()
        } else {
            *self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_scale_to_physical() {
        const TEST_VALS: [f32; 5] = [50., 25., 75., 66., 33.];
        for expected in TEST_VALS {
            let log_scale = Converter::new(0., 100., Some(expected), false);
            let res = log_scale.to_physical(0.5);
            //println!("{:#?}", res);
            assert_eq!(res, expected);
        }
    }

    #[test]
    fn test_log_scale_physical_negative() {
        const EXPECTED: f32 = 3.;

        let log_scale = Converter::new(-96., 6., Some(EXPECTED), false);
        let res = log_scale.to_physical(0.5);

        assert_eq!(res, EXPECTED);
    }

    #[test]
    fn test_log_scale_physical_negative_min() {
        const EXPECTED: f32 = -96.;

        let log_scale = Converter::new(EXPECTED, 6., None, false);
        let res = log_scale.to_physical(0.);

        assert_eq!(res, EXPECTED);
    }

    #[test]
    fn test_log_scale_physical_negative_max() {
        const EXPECTED: f32 = 6.;

        let log_scale = Converter::new(-96., EXPECTED, None, false);
        let res = log_scale.to_physical(1.);

        assert_eq!(res, EXPECTED);
    }

    #[test]
    fn test_log_scale_physical_lfo() {
        const TEST_VALS: [[f32; 2]; 3] = [[1., 30.], [0., 0.01], [0.5, 1.]];

        let log_scale = Converter::new(0.01, 30., Some(1.), false);
        for [norm, phys] in TEST_VALS {
            let res = log_scale.to_physical(norm);
            assert_eq!(res, phys);
        }
    }

    #[test]
    fn test_log_scale_physical_lfo_max_min() {
        const TEST_VALS: [[f32; 2]; 3] = [[1., 0.01], [0., 30.], [0.5, 1.]];

        let log_scale = Converter::new(30., 0.01, Some(1.), false);
        for [norm, phys] in TEST_VALS {
            let res = log_scale.to_physical(norm);
            let eps = 0.000001 as f32;
            let sub = (res - phys).abs();
            assert!(sub < eps);
        }
    }

    #[test]
    fn test_lin_scale() {
        let lin_scale = Converter::new(0., 100., None, false);
        let res = lin_scale.to_physical(0.5);
        //println!("{:#?}", res);
        assert_eq!(res, 50.);
    }

    #[test]
    fn test_lin_scale_inverted() {
        let lin_scale = Converter::new(0., 100., None, false);
        let res = lin_scale.to_normalized(50.0);
        // println!("{:#?}", res);
        assert_eq!(res, 0.5);
    }

    #[test]
    fn test_round_up() {
        let transform = Converter::new(0., 100., None, true);
        let phys = transform.to_physical(0.505);
        // println!("{:#?}", res);
        assert_eq!(phys, 51.);
    }

    #[test]
    fn test_round_down() {
        let transform = Converter::new(0., 100., None, true);
        let phys = transform.to_physical(0.5049);
        // println!("{:#?}", res);
        assert_eq!(phys, 50.);
    }

    #[test]
    fn test_display_w_precision() {
        let transform = Converter::new(0., 100., None, false);
        let phys = transform.to_display(50., Some(2));
        // println!("{:#?}", res);
        assert_eq!(phys, "50.00");
    }
}
