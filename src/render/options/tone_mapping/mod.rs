use std::fmt::Display;

use crate::{
    animation::lerp,
    matrix::Mat4,
    vec::vec3::{self, Vec3},
};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum ToneMappingOperator {
    #[default]
    Reinhard,
    ReinhardExtended(f32),
    ReinhardExtendedLuminance(f32),
    ReinhardJodie,
    HableFilmic,
    ACES,
    ACESApproximate,
    Exposure(f32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct CastToneMappingOperatorToUsizeError;

impl TryInto<usize> for ToneMappingOperator {
    type Error = CastToneMappingOperatorToUsizeError;

    fn try_into(self) -> Result<usize, Self::Error> {
        match self {
            ToneMappingOperator::Reinhard => Ok(0),
            ToneMappingOperator::ReinhardExtended(_) => Ok(1),
            ToneMappingOperator::ReinhardExtendedLuminance(_) => Ok(2),
            ToneMappingOperator::ReinhardJodie => Ok(3),
            ToneMappingOperator::HableFilmic => Ok(4),
            ToneMappingOperator::ACES => Ok(5),
            ToneMappingOperator::ACESApproximate => Ok(6),
            ToneMappingOperator::Exposure(_) => Ok(7),
        }
    }
}

impl Display for ToneMappingOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ToneMappingOperator::Reinhard => "Reinhard",
                ToneMappingOperator::ReinhardExtended(_) => "ReinhardExtended",
                ToneMappingOperator::ReinhardExtendedLuminance(_) => "ReinhardExtendedLuminance",
                ToneMappingOperator::ReinhardJodie => "ReinhardJodie",
                ToneMappingOperator::HableFilmic => "HableFilmic",
                ToneMappingOperator::ACES => "ACES",
                ToneMappingOperator::ACESApproximate => "ACESApproximate",
                ToneMappingOperator::Exposure(_) => "Exposure",
            }
        )
    }
}

impl ToneMappingOperator {
    pub fn map(&self, hdr: Vec3) -> Vec3 {
        // See: https://64.github.io/tonemapping/

        match self {
            ToneMappingOperator::Reinhard => reinhard(hdr),
            ToneMappingOperator::ReinhardExtended(white_squared) => {
                reinhard_extended(hdr, *white_squared)
            }
            ToneMappingOperator::ReinhardExtendedLuminance(luminance_white_squared) => {
                reinhard_extended_luminance(hdr, *luminance_white_squared)
            }
            ToneMappingOperator::ReinhardJodie => reinhard_jodie(hdr),
            ToneMappingOperator::HableFilmic => hable_filmic(hdr),
            ToneMappingOperator::ACES => aces(hdr),
            ToneMappingOperator::ACESApproximate => aces_approximate(hdr),
            ToneMappingOperator::Exposure(exposure) => exposure_exponentiated(hdr, *exposure),
        }
    }
}

pub static TONE_MAPPING_OPERATORS: [ToneMappingOperator; 8] = [
    ToneMappingOperator::Reinhard,
    ToneMappingOperator::ReinhardExtended(1.0),
    ToneMappingOperator::ReinhardExtendedLuminance(1.0),
    ToneMappingOperator::ReinhardJodie,
    ToneMappingOperator::HableFilmic,
    ToneMappingOperator::ACES,
    ToneMappingOperator::ACESApproximate,
    ToneMappingOperator::Exposure(1.0),
];

fn reinhard(hdr: Vec3) -> Vec3 {
    // See:
    // https://www-old.cs.utah.edu/docs/techreports/2002/pdf/UUCS-02-001.pdf

    hdr / (hdr + vec3::ONES)
}

fn reinhard_extended(hdr: Vec3, white_squared: f32) -> Vec3 {
    // See:
    // https://www-old.cs.utah.edu/docs/techreports/2002/pdf/UUCS-02-001.pdf

    let numerator = hdr * (vec3::ONES + hdr * (1.0 / white_squared));

    let denominator = vec3::ONES + hdr;

    numerator / denominator
}

fn reinhard_extended_luminance(hdr: Vec3, luminance_white_squared: f32) -> Vec3 {
    // See:
    // https://imdoingitwrong.wordpress.com/2010/08/19/why-reinhard-desaturates-my-blacks-3/
    // https://www.shadertoy.com/view/lslGzl

    let old_luminance = hdr.luminance();

    let numerator = (1.0 + old_luminance / luminance_white_squared) * old_luminance;

    let denominator = 1.0 + old_luminance;

    let new_luminance = numerator / denominator;

    hdr.with_luminance(new_luminance)

    // hdr / (1.0 + hdr.luminance())
}

fn reinhard_jodie(hdr: Vec3) -> Vec3 {
    // See:
    // https://www.shadertoy.com/view/4dBcD1

    let luminance = hdr.luminance();

    let reinhard = hdr / (vec3::ONES + hdr);

    let (start, end) = (hdr / (1.0 + luminance), reinhard);

    Vec3 {
        x: lerp(start.x, end.x, reinhard.x),
        y: lerp(start.y, end.y, reinhard.y),
        z: lerp(start.z, end.z, reinhard.z),
    }
}

fn hable_filmic(hdr: Vec3) -> Vec3 {
    // See:
    // http://filmicworlds.com/blog/filmic-tonemapping-with-piecewise-power-curves/
    // http://slideshare.net/ozlael/hable-john-uncharted2-hdr-lighting

    fn hable_filmic_partial(hdr: Vec3) -> Vec3 {
        let a = vec3::ONES * 0.15;
        let b = vec3::ONES * 0.50;
        let c = vec3::ONES * 0.10;
        let d = vec3::ONES * 0.20;
        let e = vec3::ONES * 0.02;
        let f = vec3::ONES * 0.30;

        let hdr_a = hdr * a;

        ((hdr * (hdr_a + c * b) + d * e) / (hdr * (hdr_a + b) + d * f)) - e / f
    }

    static EXPOSURE_BIAS: f32 = 2.0;

    let current = hable_filmic_partial(hdr * EXPOSURE_BIAS);

    let white = vec3::ONES * 11.2;

    let white_scale = vec3::ONES / hable_filmic_partial(white);

    current / white_scale
}

fn aces(mut hdr: Vec3) -> Vec3 {
    // See:
    // https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl

    fn rtt_to_odt_fit(hdr: Vec3) -> Vec3 {
        let a = vec3::ONES * 0.0245786;
        let b = vec3::ONES * 0.000090537;
        let c = vec3::ONES * 0.983729;
        let d = vec3::ONES * 0.432951;
        let e = vec3::ONES * 0.238081;

        let f = hdr * (hdr + a) - b;
        let g = hdr * (c * hdr + d) + e;

        f / g
    }

    // sRGB -> XYZ -> D65_2_D60 -> AP1 -> RRT_SAT
    static ACES_RGB_TO_RTT_FIT: Mat4 = Mat4::new_from_elements([
        [0.59719, 0.35458, 0.04823, 0.0],
        [0.07600, 0.90834, 0.01566, 0.0],
        [0.02840, 0.13383, 0.83777, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ]);

    // ODT_SAT -> XYZ -> D60_2_D65 -> sRGB
    static ACES_ODT_TO_RGB_FIT: Mat4 = Mat4::new_from_elements([
        [1.60475, -0.53108, -0.07367, 0.0],
        [-0.10208, 1.10813, -0.00605, 0.0],
        [-0.00327, -0.07276, 1.07602, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ]);

    hdr *= ACES_RGB_TO_RTT_FIT;

    hdr = rtt_to_odt_fit(hdr);

    hdr *= ACES_ODT_TO_RGB_FIT;

    hdr.clamp(0.0, 1.0)
}

fn aces_approximate(mut hdr: Vec3) -> Vec3 {
    // See:
    // https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/

    hdr *= vec3::ONES * 0.6;

    let a = vec3::ONES * 2.51;
    let b = vec3::ONES * 0.03;
    let c = vec3::ONES * 2.43;
    let d = vec3::ONES * 0.59;
    let e = vec3::ONES * 0.14;

    hdr = (hdr * (a * hdr + b)) / (hdr * (c * hdr + d) + e);

    hdr.clamp(0.0, 1.0)
}

fn exposure_exponentiated(hdr: Vec3, exposure: f32) -> Vec3 {
    vec3::ONES
        - Vec3 {
            x: (-hdr.x * exposure).exp(),
            y: (-hdr.y * exposure).exp(),
            z: (-hdr.z * exposure).exp(),
        }
}
