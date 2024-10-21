use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LightAttenuation {
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

impl LightAttenuation {
    pub fn attenuate_for_distance(&self, distance: f32) -> f32 {
        1.0 / (self.quadratic * distance * distance + self.linear * distance + self.constant)
    }

    pub fn get_approximate_influence_distance(&self) -> f32 {
        let mut distance: f32 = 0.01;

        let mut attenuation = self.attenuate_for_distance(distance);

        while attenuation > 0.1 {
            distance += 0.01;

            attenuation = self.attenuate_for_distance(distance);
        }

        distance -= 0.01;

        distance
    }
}
