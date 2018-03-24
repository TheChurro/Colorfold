
use std::f32::EPSILON;
use std::f32::consts::PI;
use std::f32::consts::FRAC_PI_2;

use serde::{Serialize, Serializer, Deserialize, Deserializer};

#[derive(Clone)]
pub struct Color(pub u8, pub u8, pub u8);

#[derive(Clone, Serialize, Deserialize)]
pub enum ColorProperties
{
    Hue,
    Value,
    Saturation,
    Red,
    Green,
    Blue
}

impl ColorProperties
{
    pub fn suffix(&self) -> &'static str
    {
        use color::ColorProperties::*;
        match self
        {
            &Hue        => "hsv.x",
            &Value      => "hsv.y",
            &Saturation => "hsv.z",
            &Red   => "rgb.x",
            &Green => "rgb.y",
            &Blue  => "rgb.z",
        }
    }

    pub fn get_color_space(&self) -> &'static str
    {
        use color::ColorProperties::*;
        match self
        {
            &Hue        => "hsv",
            &Value      => "hsv",
            &Saturation => "hsv",
            &Red   => "rgb",
            &Green => "rgb",
            &Blue  => "rgb",
        }
    }
}

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
                (((g - b) / delta + 6.0) % 6.0) / 6.0
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
                if dot(axis1, axis1) <= 10.0 * EPSILON
                {
                    axis = axis2;
                }
                else if dot(axis2, axis2) <= 10.0 * EPSILON
                {
                    axis = axis1;
                }
                else if dot(axis1, (0.0, 0.0, 1.0)) <= dot(axis2, (0.0, 0.0, 1.0))
                {
                    axis = axis1;
                }
                else
                {
                    axis = axis2;
                }
            }

            axis = div_all(axis, len(axis));
            let angle_half = (dot(start, end) / (len(start) * len(end))).acos() / 2.0;
            let sin_half = angle_half.sin();

            (axis.0 * sin_half, axis.1 * sin_half, axis.2 * sin_half, angle_half.cos())
        }
    }
}

// ================================================================================================
// == Serde Serialization for parsing input files.                                               ==
// ================================================================================================
#[derive(Serialize, Deserialize)]
struct SerializableColor {
    red: u8,
    green: u8,
    blue: u8
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        SerializableColor { red: self.0, green: self.1, blue: self.2 }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        Deserialize::deserialize(deserializer)
            .map(|SerializableColor { red, green, blue }| Color(red, green, blue))
    }
}
