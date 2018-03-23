
use std::f32::EPSILON;
use std::f32::consts::PI;
use std::f32::consts::FRAC_PI_2;

pub struct Color(pub u8, pub u8, pub u8);

fn max(a:f32, b:f32) -> f32
{
    if a < b {b} else {a}
}

fn min(a:f32, b:f32) -> f32
{
    if a > b {b} else {a}
}

fn div_all(a : (f32, f32, f32), b : f32) -> (f32, f32, f32)
{
    (a.0 / b, a.1 / b, a.2 / b)
}

fn len(a : (f32, f32, f32)) -> f32
{
    (a.0 * a.0 + a.1 * a.1 + a.2 * a.2).sqrt()
}

fn cross(a : (f32, f32, f32), b : (f32, f32, f32)) -> (f32, f32, f32)
{
    (a.1 * b.2 - a.2 * b.1, a.2 * b.0 - a.0 * b.2, a.0 * b.1 - a.1 * b.0)
}

fn dot(a : (f32, f32, f32), b : (f32, f32, f32)) -> f32
{
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

impl Color
{
    pub fn to_hsv_vec(&self) -> (f32, f32, f32)
    {
        let &Color(r, g, b) = self;
        let r = (r as f32) / 255.0;
        let g = (g as f32) / 255.0;
        let b = (b as f32) / 255.0;

        let c_max = max(r, max(g, b));
        let c_min = min(r, min(g, b));

        let delta = c_max - c_min;

        let mut hue =
            if delta < 10.0 * EPSILON
            {
                0.0
            }
            else if c_max == r
            {
                (((g - b) / delta) % 6.0) / 6.0
            }
            else if c_max == g
            {
                (((b - r) / delta) + 2.0) / 6.0
            }
            else
            {
                (((r - g) / delta) + 4.0) / 6.0
            };
        if hue < 0.0
        {
            hue += 1.0;
        }

        let sat = if c_max < 10.0 * EPSILON { 0.0 } else { delta / c_max };

        let val = c_max;

        let hue_angle = 2.0 * PI * hue;
        let sat_angle = sat * FRAC_PI_2;
        let sat_sin   = sat_angle.sin();

        (val * sat_sin * hue_angle.cos(), val * sat_sin * hue_angle.sin(), val * sat_angle.cos())
    }

    pub fn vec_len(&self) -> f32
    {
        len(self.to_hsv_vec())
    }

    pub fn get_rotation(&self, other : &Color) -> (f32, f32, f32, f32)
    {
        let start = self.to_hsv_vec();
        let end   = other.to_hsv_vec();

        if len(start) <= 10.0 * EPSILON || len(end) <= 10.0 * EPSILON
        {
            (0.0, 0.0, 0.0, 1.0)
        }
        else
        {
            let mut axis = cross(start, end);
            if dot(axis, axis) <= 10.0 * EPSILON
            {
                let axis1 = cross(start, (0.0, 1.0, 0.0));
                let axis2 = cross(start, (1.0, 0.0, 0.0));
                if dot(axis1, axis1) <= dot(axis2, axis2)
                {
                    axis = axis2;
                }
                else
                {
                    axis = axis1;
                }
            }

            axis = div_all(axis, len(axis));
            let angle_half = (dot(start, end) / (len(start) * len(end))).acos() / 2.0;
            let sin_half = angle_half.sin();

            (axis.0 * sin_half, axis.1 * sin_half, axis.2 * sin_half, angle_half.cos())
        }
    }
}
