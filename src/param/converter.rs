// Copyright(c) 2023 Hansen Audio.

use std::ops::Neg;

use crate::param::display_handling::DisplayHandling;
use crate::param::range::Range;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum Kind {
    Float,
    Int,
}

#[derive(Debug, Clone, Copy)]
pub struct Converter {
    range: Range,
    kind: Kind,
    scale_factor: Option<f32>,
}

impl Converter {
    pub fn new(min: f32, max: f32, mid: Option<f32>, kind: Kind) -> Self {
        Self {
            range: Range::new(min, max),
            kind,
            scale_factor: Self::calc_opt_transform_factor(min, max, mid),
        }
    }

    pub fn to_physical(&self, normalized: f32) -> f32 {
        Physicalizer::new(normalized)
            .clamp()
            .scale(self.scale_factor)
            .transform(self.range)
            .round(self.kind)
    }

    pub fn to_normalized(&self, physical: f32) -> f32 {
        Normalizer::new(physical)
            .clamp(self.range)
            .round(self.kind)
            .transform(self.range)
            .scale(self.scale_factor)
    }

    pub fn min(&self) -> f32 {
        self.range.min()
    }

    pub fn max(&self) -> f32 {
        self.range.max()
    }

    pub fn num_steps(&self) -> usize {
        match self.kind() {
            Kind::Float => 0,
            Kind::Int => (self.range.range()) as usize,
        }
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    fn calc_opt_transform_factor(min: f32, max: f32, mid: Option<f32>) -> Option<f32> {
        let a = match mid {
            Some(opt_mid) => Some(Self::calc_transform_factor(min, max, opt_mid)),
            None => None,
        };
        a
    }

    fn calc_transform_factor(min: f32, max: f32, mid: f32) -> f32 {
        let y = (mid - min) / (max - min);
        let x = 0.5;
        // let t = -1.0 / (((x / y - x) / (x - 1.0)) - 1.0);
        let t = y.neg() * (x - 1.) / (x - 2. * x * y + y);

        1. - (1. / t)
    }
}

impl Range {
    pub fn clamp(&self, value: f32) -> f32 {
        if self.is_inverted() {
            value.clamp(self.max(), self.min())
        } else {
            value.clamp(self.min(), self.max())
        }
    }
}

#[derive(Debug, Clone)]
struct Normalizer {
    value: f32,
}

impl Normalizer {
    fn new(value: f32) -> Self {
        Self { value }
    }

    fn clamp(&mut self, range: Range) -> &mut Self {
        self.value = range.clamp(self.value);

        self
    }

    fn scale(&mut self, scale_factor: Option<f32>) -> f32 {
        match scale_factor {
            //Some(a) => -1.0 / ((1.0 / self.value - 1.0) / a - 1.0),
            Some(a) => (a.neg() * self.value) / ((a.neg() * self.value) - self.value + 1.0),
            None => self.value,
        }
    }

    fn transform(&mut self, range: Range) -> &mut Self {
        self.value = (self.value - range.min()) * range.range_inv();
        self
    }

    fn round(&mut self, kind: Kind) -> &mut Self {
        self.value = match kind {
            Kind::Float => self.value,
            Kind::Int => self.value.round(),
        };

        self
    }
}

#[derive(Debug, Clone)]
struct Physicalizer {
    value: f32,
}

impl Physicalizer {
    const NORM_MIN: f32 = 0.;
    const NORM_MAX: f32 = 1.;
    fn new(value: f32) -> Self {
        Self { value }
    }

    fn clamp(&mut self) -> &mut Self {
        self.value = self.value.clamp(Self::NORM_MIN, Self::NORM_MAX);
        self
    }

    fn scale(&mut self, scale_factor: Option<f32>) -> &mut Self {
        self.value = match scale_factor {
            Some(a) => self.value / (self.value + a * (self.value - Self::NORM_MAX)),
            None => self.value,
        };

        self
    }

    fn transform(&mut self, value_range: Range) -> &mut Self {
        self.value = self.value * (value_range.range()) + value_range.min();

        self
    }

    fn round(&mut self, kind: Kind) -> f32 {
        self.value = match kind {
            Kind::Float => self.value,
            Kind::Int => self.value.round(),
        };

        self.value
    }
}

impl DisplayHandling for Converter {
    fn to_display(&self, physical: f32, precision: Option<usize>) -> String {
        format!(
            "{1:.0$}",
            precision.unwrap_or(Self::DEFAULT_PRECISION),
            physical
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_scale_to_physical() {
        const TEST_VALS: [f32; 5] = [50., 25., 75., 66., 33.];
        for expected in TEST_VALS {
            let log_scale = Converter::new(0., 100., Some(expected), Kind::Float);
            let res = log_scale.to_physical(0.5);
            //println!("{:#?}", res);
            assert_eq!(res, expected);
        }
    }

    #[test]
    fn test_log_scale_physical_negative() {
        const EXPECTED: f32 = 3.;

        let log_scale = Converter::new(-96., 6., Some(EXPECTED), Kind::Float);
        let res = log_scale.to_physical(0.5);

        assert_eq!(res, EXPECTED);
    }

    #[test]
    fn test_log_scale_physical_negative_min() {
        const EXPECTED: f32 = -96.;

        let log_scale = Converter::new(EXPECTED, 6., None, Kind::Float);
        let res = log_scale.to_physical(0.);

        assert_eq!(res, EXPECTED);
    }

    #[test]
    fn test_log_scale_physical_negative_max() {
        const EXPECTED: f32 = 6.;

        let log_scale = Converter::new(-96., EXPECTED, None, Kind::Float);
        let res = log_scale.to_physical(1.);

        assert_eq!(res, EXPECTED);
    }

    #[test]
    fn test_log_scale_physical_lfo() {
        const TEST_VALS: [[f32; 2]; 3] = [[1., 30.], [0., 0.01], [0.5, 1.]];

        let log_scale = Converter::new(0.01, 30., Some(1.), Kind::Float);
        for [norm, phys] in TEST_VALS {
            let res = log_scale.to_physical(norm);
            assert_eq!(res, phys);
        }
    }

    #[test]
    fn test_log_scale_physical_lfo_max_min() {
        const TEST_VALS: [[f32; 2]; 3] = [[1., 0.01], [0., 30.], [0.5, 1.]];

        let log_scale = Converter::new(30., 0.01, Some(1.), Kind::Float);
        for [norm, phys] in TEST_VALS {
            let res = log_scale.to_physical(norm);
            let eps = 0.000001 as f32;
            let sub = (res - phys).abs();
            assert!(sub < eps);
        }
    }

    #[test]
    fn test_lin_scale() {
        let lin_scale = Converter::new(0., 100., None, Kind::Float);
        let res = lin_scale.to_physical(0.5);
        //println!("{:#?}", res);
        assert_eq!(res, 50.);
    }

    #[test]
    fn test_lin_scale_inverted() {
        let lin_scale = Converter::new(0., 100., None, Kind::Float);
        let res = lin_scale.to_normalized(50.0);
        // println!("{:#?}", res);
        assert_eq!(res, 0.5);
    }

    #[test]
    fn test_round_up() {
        let transform = Converter::new(0., 100., None, Kind::Int);
        let phys = transform.to_physical(0.505);
        // println!("{:#?}", res);
        assert_eq!(phys, 51.);
    }

    #[test]
    fn test_round_down() {
        let transform = Converter::new(0., 100., None, Kind::Int);
        let phys = transform.to_physical(0.5049);
        // println!("{:#?}", res);
        assert_eq!(phys, 50.);
    }

    #[test]
    fn test_display_w_precision() {
        let transform = Converter::new(0., 100., None, Kind::Float);
        let phys = transform.to_display(50., Some(2));
        // println!("{:#?}", res);
        assert_eq!(phys, "50.00");
    }

    #[test]
    fn test_math_equality() {
        let val = 0.567 as f32;
        let a = 0.234;
        let x = -1.0 / ((1.0 / val - 1.0) / a - 1.0);
        let y = (a.neg() * val) / ((a.neg() * val) - val + 1.0);

        let z = (x - y).abs();
        let eps = 0.000001 as f32;
        assert!(z < eps);
    }

    #[test]
    fn test_scale_factor_simplification() {
        let min = 0. as f32;
        let max = 100. as f32;
        let mid = 0.25 as f32;
        let y = (mid - min) / (max - min);
        let x = 0.5;

        let t = -1.0 / (((x / y - x) / (x - 1.0)) - 1.0);
        let s = y.neg() * (x - 1.) / (x - 2. * x * y + y);
        let z = (t - s).abs();
        let eps = 0.000001 as f32;
        assert!(z < eps);
    }
}
