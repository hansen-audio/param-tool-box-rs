// Copyright(c) 2021 Hansen Audio.

use crate::convert::transform::Display;
use crate::convert::transform::Transform;

#[derive(Debug, Clone)]
pub struct Converter {
    min: f32,
    max: f32,
    a: Option<f32>,
    is_int: bool,
}

impl Converter {
    pub fn new(min: f32, max: f32, mid: Option<f32>, is_int: bool) -> Self {
        Self {
            min,
            max,
            a: calc_opt_transform_factor(min, max, mid),
            is_int,
        }
    }

    pub fn is_int(&self) -> bool {
        self.is_int
    }

    fn up_transform(&self, norm: f32) -> f32 {
        norm * (self.max - self.min) + self.min
    }

    fn down_transform(&self, phys: f32) -> f32 {
        (phys - self.min) / (self.max - self.min)
    }
}

impl Transform for Converter {
    fn to_physical(&self, normalized: f32) -> f32 {
        let clamped_normalized = normalized.clamp(0., 1.);

        // TODO: Can this be optimized at compile time?
        let scaled_normalized = match self.a {
            Some(a) => up_scale(clamped_normalized, a),
            None => clamped_normalized,
        };

        let physical = self.up_transform(scaled_normalized);
        if self.is_int {
            physical.round()
        } else {
            physical
        }
    }

    fn to_normalized(&self, physcial: f32) -> f32 {
        let mut clamped_physical = physcial.clamp(self.min, self.max);
        if self.is_int {
            clamped_physical = clamped_physical.round()
        };

        let normalized = self.down_transform(clamped_physical);

        // TODO: Can this be optimized at compile time?
        match self.a {
            Some(a) => down_scale(normalized, a),
            None => normalized,
        }
    }

    fn min(&self) -> f32 {
        self.min
    }

    fn max(&self) -> f32 {
        self.max
    }
}

impl Display for Converter {
    fn to_display(&self, val: f32, precision: Option<usize>) -> String {
        let clamped_val = val.clamp(self.min(), self.max());

        const DEFAULT_PRECISION: usize = 2;
        format!(
            "{1:.0$}",
            precision.unwrap_or(DEFAULT_PRECISION),
            clamped_val
        )
    }

    fn from_display(&self, value: String) -> f32 {
        let parse_result = value.parse::<f32>();

        match parse_result {
            Ok(val) => val,
            Err(_) => self.min(),
        }
    }
}

fn calc_opt_transform_factor(min: f32, max: f32, mid: Option<f32>) -> Option<f32> {
    let a = match mid {
        Some(opt_mid) => Some(calc_transform_factor(min, max, opt_mid)),
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

fn up_scale(val: f32, a: f32) -> f32 {
    val / (val + a * (val - 1.0))
}

fn down_scale(val: f32, a: f32) -> f32 {
    -1.0 / ((1.0 / val - 1.0) / a - 1.0)
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
