/* @piater, @MaxVerevkin
 * https://github.com/MaxVerevkin/wl-gammarelay-rs
 * Copyright © 2024

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

/// Color parameters
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub temp: u16,
    pub gamma: f64,
    pub brightness: f64,
    pub inverted: bool,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            temp: 6500,
            gamma: 1.0,
            brightness: 1.0,
            inverted: false,
        }
    }
}

impl Color {
    /// Returns a color with update temperature, clamping it to [1K, 10K] range, or none if
    /// temperature could not be updated.
    pub fn with_updated_temp(self, delta: i16) -> Option<Self> {
        let new_temp = self.temp.saturating_add_signed(delta).clamp(1_000, 10_000);
        (new_temp != self.temp).then_some(Self {
            temp: new_temp,
            ..self
        })
    }
}

fn map_intensity(v: f64, white: f64, color: Color, v_max_gamma: f64) -> u16 {
    // A gamma ramp is computed as f(x) = x^γ, for x ∈ [0,1].
    // Multiple gamma adjustments can reasonably be combined as
    // f(x) = x^(γ₁γ₂) = (x^γ₁)^γ₂.
    // Here, x^γ₁ ≡ (v * white) is the color-temperature-adjusted intensity,
    // and γ₂ is the overall gamma correction.
    // The factor v_max_gamma adjusts for the fact that generally v ∉ [0,1]:
    // (v/v_max)^γ * v_max = v^γ * v_max^(1-γ)
    ((v * white).powf(color.gamma) * v_max_gamma) as u16
}

pub fn colorramp_fill(r: &mut [u16], g: &mut [u16], b: &mut [u16], ramp_size: usize, color: Color) {
    let color_i = ((color.temp as usize - 1000) / 100) * 3;
    let [white_r, white_g, white_b] = interpolate_color(
        (color.temp % 100) as f64 / 100.0,
        &BLACKBODY_COLOR[color_i..],
        &BLACKBODY_COLOR[(color_i + 3)..],
    );

    let v_max = u16::MAX as f64 * color.brightness;
    let v_max_gamma = v_max.powf(1.0 - color.gamma);
    let step = v_max / (ramp_size - 1) as f64;
    for i in 0..ramp_size {
        let v = step * i as f64;
        let index = if color.inverted { ramp_size - 1 - i } else { i };
        r[index] = map_intensity(v, white_r, color, v_max_gamma);
        g[index] = map_intensity(v, white_g, color, v_max_gamma);
        b[index] = map_intensity(v, white_b, color, v_max_gamma);
    }
}

fn interpolate_color(a: f64, c1: &[f64], c2: &[f64]) -> [f64; 3] {
    [
        (1.0 - a) * c1[0] + a * c2[0],
        (1.0 - a) * c1[1] + a * c2[1],
        (1.0 - a) * c1[2] + a * c2[2],
    ]
}

/// [Black body radiation color](https://en.wikipedia.org/wiki/Black-body_radiation) mapped by
/// temperature in the range [1_000,10_100].
///
/// Refer to <https://gitlab.com/chinstrap/gammastep/-/blob/master/README-colorramp> for more info.
const BLACKBODY_COLOR: &[f64] = &[
    1.00000000, 0.18172716, 0.00000000, 1.00000000, 0.25503671, 0.00000000, 1.00000000, 0.30942099,
    0.00000000, 1.00000000, 0.35357379, 0.00000000, 1.00000000, 0.39091524, 0.00000000, 1.00000000,
    0.42322816, 0.00000000, 1.00000000, 0.45159884, 0.00000000, 1.00000000, 0.47675916, 0.00000000,
    1.00000000, 0.49923747, 0.00000000, 1.00000000, 0.51943421, 0.00000000, 1.00000000, 0.54360078,
    0.08679949, 1.00000000, 0.56618736, 0.14065513, 1.00000000, 0.58734976, 0.18362641, 1.00000000,
    0.60724493, 0.22137978, 1.00000000, 0.62600248, 0.25591950, 1.00000000, 0.64373109, 0.28819679,
    1.00000000, 0.66052319, 0.31873863, 1.00000000, 0.67645822, 0.34786758, 1.00000000, 0.69160518,
    0.37579588, 1.00000000, 0.70602449, 0.40267128, 1.00000000, 0.71976951, 0.42860152, 1.00000000,
    0.73288760, 0.45366838, 1.00000000, 0.74542112, 0.47793608, 1.00000000, 0.75740814, 0.50145662,
    1.00000000, 0.76888303, 0.52427322, 1.00000000, 0.77987699, 0.54642268, 1.00000000, 0.79041843,
    0.56793692, 1.00000000, 0.80053332, 0.58884417, 1.00000000, 0.81024551, 0.60916971, 1.00000000,
    0.81957693, 0.62893653, 1.00000000, 0.82854786, 0.64816570, 1.00000000, 0.83717703, 0.66687674,
    1.00000000, 0.84548188, 0.68508786, 1.00000000, 0.85347859, 0.70281616, 1.00000000, 0.86118227,
    0.72007777, 1.00000000, 0.86860704, 0.73688797, 1.00000000, 0.87576611, 0.75326132, 1.00000000,
    0.88267187, 0.76921169, 1.00000000, 0.88933596, 0.78475236, 1.00000000, 0.89576933, 0.79989606,
    1.00000000, 0.90198230, 0.81465502, 1.00000000, 0.90963069, 0.82838210, 1.00000000, 0.91710889,
    0.84190889, 1.00000000, 0.92441842, 0.85523742, 1.00000000, 0.93156127, 0.86836903, 1.00000000,
    0.93853986, 0.88130458, 1.00000000, 0.94535695, 0.89404470, 1.00000000, 0.95201559, 0.90658983,
    1.00000000, 0.95851906, 0.91894041, 1.00000000, 0.96487079, 0.93109690, 1.00000000, 0.97107439,
    0.94305985, 1.00000000, 0.97713351, 0.95482993, 1.00000000, 0.98305189, 0.96640795, 1.00000000,
    0.98883326, 0.97779486, 1.00000000, 0.99448139, 0.98899179, 1.00000000, 1.00000000, 1.00000000,
    0.98947904, 0.99348723, 1.00000000, 0.97940448, 0.98722715, 1.00000000, 0.96975025, 0.98120637,
    1.00000000, 0.96049223, 0.97541240, 1.00000000, 0.95160805, 0.96983355, 1.00000000, 0.94303638,
    0.96443333, 1.00000000, 0.93480451, 0.95923080, 1.00000000, 0.92689056, 0.95421394, 1.00000000,
    0.91927697, 0.94937330, 1.00000000, 0.91194747, 0.94470005, 1.00000000, 0.90488690, 0.94018594,
    1.00000000, 0.89808115, 0.93582323, 1.00000000, 0.89151710, 0.93160469, 1.00000000, 0.88518247,
    0.92752354, 1.00000000, 0.87906581, 0.92357340, 1.00000000, 0.87315640, 0.91974827, 1.00000000,
    0.86744421, 0.91604254, 1.00000000, 0.86191983, 0.91245088, 1.00000000, 0.85657444, 0.90896831,
    1.00000000, 0.85139976, 0.90559011, 1.00000000, 0.84638799, 0.90231183, 1.00000000, 0.84153180,
    0.89912926, 1.00000000, 0.83682430, 0.89603843, 1.00000000, 0.83225897, 0.89303558, 1.00000000,
    0.82782969, 0.89011714, 1.00000000, 0.82353066, 0.88727974, 1.00000000, 0.81935641, 0.88452017,
    1.00000000, 0.81530175, 0.88183541, 1.00000000, 0.81136180, 0.87922257, 1.00000000, 0.80753191,
    0.87667891, 1.00000000, 0.80380769, 0.87420182, 1.00000000, 0.80018497, 0.87178882, 1.00000000,
    0.79665980, 0.86943756, 1.00000000, 0.79322843, 0.86714579, 1.00000000, 0.78988728, 0.86491137,
    1.00000000, 0.78663296, 0.86273225, 1.00000000,
];