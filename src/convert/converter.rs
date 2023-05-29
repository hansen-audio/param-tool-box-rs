// Copyright(c) 2023 Hansen Audio.

use crate::convert::display_handling::DisplayHandling;

#[derive(Debug, Clone, Copy)]
pub struct Converter {
    context: Context,
}

impl Converter {
    pub fn new(min: f32, max: f32, mid: Option<f32>, is_int: bool) -> Self {
        Self {
            context: Context::new(min, max, mid, is_int),
        }
    }

    pub fn is_int(&self) -> bool {
        self.context.is_int
    }
}

impl Converter {
    pub fn to_physical(&self, normalized: f32) -> f32 {
        Physicalizer::new(normalized, self.context)
            .clamp()
            .scale()
            .transform()
            .round()
    }

    pub fn to_normalized(&self, physical: f32) -> f32 {
        Normalizer::new(physical, self.context)
            .clamp()
            .round()
            .transform()
            .scale()
    }

    pub fn min(&self) -> f32 {
        self.context.min
    }

    pub fn max(&self) -> f32 {
        self.context.max
    }
}

impl DisplayHandling for Converter {
    fn to_display(&self, physical: f32, precision: Option<usize>) -> String {
        const DEFAULT_PRECISION: usize = 2;
        format!("{1:.0$}", precision.unwrap_or(DEFAULT_PRECISION), physical)
    }

    fn from_display(&self, physical: String) -> f32 {
        let value = physical.parse::<f32>();

        match value {
            Ok(val) => val,
            Err(_) => self.min(),
        }
    }
}

#[derive(Debug, Clone)]
struct Normalizer {
    value: f32,
    context: Context,
}

impl Normalizer {
    fn new(value: f32, context: Context) -> Self {
        Self { value, context }
    }

    fn clamp(&mut self) -> &mut Self {
        if self.context.is_inverted() {
            self.value = self.value.clamp(self.context.max, self.context.min);
        } else {
            self.value = self.value.clamp(self.context.min, self.context.max);
        };
        self
    }

    fn scale(&mut self) -> f32 {
        match self.context.scale_factor {
            Some(a) => -1.0 / ((1.0 / self.value - 1.0) / a - 1.0),
            None => self.value,
        }
    }

    fn transform(&mut self) -> &mut Self {
        self.value = (self.value - self.context.min) * self.context.range_inv;
        self
    }

    fn round(&mut self) -> &mut Self {
        if self.context.is_int {
            self.value = self.value.round();
        }

        self
    }
}

#[derive(Debug, Clone)]
struct Physicalizer {
    value: f32,
    context: Context,
}

impl Physicalizer {
    fn new(value: f32, context: Context) -> Self {
        Self { value, context }
    }

    fn clamp(&mut self) -> &mut Self {
        self.value = self.value.clamp(0., 1.);
        self
    }

    fn scale(&mut self) -> &mut Self {
        self.value = match self.context.scale_factor {
            Some(a) => self.value / (self.value + a * (self.value - 1.0)),
            None => self.value,
        };

        self
    }

    fn transform(&mut self) -> &mut Self {
        self.value = self.value * (self.context.range) + self.context.min;

        self
    }

    fn round(&mut self) -> f32 {
        if self.context.is_int {
            self.value = self.value.round();
        }

        self.value
    }
}

#[derive(Debug, Clone, Copy)]
struct Context {
    min: f32,
    max: f32,
    range: f32,
    range_inv: f32,
    is_int: bool,
    scale_factor: Option<f32>,
}

impl Context {
    fn new(min: f32, max: f32, mid: Option<f32>, is_int: bool) -> Self {
        Self {
            min,
            max,
            range: max - min,
            range_inv: 1. / (max - min),
            is_int,
            scale_factor: Self::calc_opt_transform_factor(min, max, mid),
        }
    }

    fn is_inverted(&self) -> bool {
        self.max < self.min
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

trait Physical {
    fn clamp_inv(&self, min: f32, max: f32) -> f32;
}

impl Physical for f32 {
    fn clamp_inv(&self, min: f32, max: f32) -> f32 {
        let is_inverted = max < min;
        return if is_inverted {
            self.clamp(max, min)
        } else {
            self.clamp(min, max)
        };
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
