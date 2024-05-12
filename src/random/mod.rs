// See: http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html

use crate::vec::vec2::Vec2;

pub fn van_der_corput_sequence_decimal_inverse(mut bits: u32) -> f32 {
    bits = (bits << 16) | (bits >> 16);
    bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
    bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
    bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
    bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);

    bits as f32 * 2.328_306_4e-10
}

pub fn hammersley_2d_sequence(i: u32, one_over_n: f32) -> Vec2 {
    Vec2 {
        x: i as f32 * one_over_n,
        y: van_der_corput_sequence_decimal_inverse(i),
        z: 0.0,
    }
}
