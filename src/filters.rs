
use color::Color;

use serde;
use serde_json;
use serde_derive;

#[derive(Clone, Serialize, Deserialize)]
pub enum Control
{
    Point(Color),
    // Line(Color, Color), TODO: Implement these times
    // Circle { center : Color, radius : f32, normal : (f32, f32, f32) },
    // Disc { center : Color, radius : f32, normal : (f32, f32, f32) },
    // Arc(Color, Color),
}

impl Control
{
    fn shader_fn_name(&self) -> String
    {
        use filters::Control::Point;

        match self
        {
            &Point(_) => "point".to_string(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Merger
{
    ValueLine(Control, Control),
    ValueLineSegment
    {
        start_value : f32, start_control : Control,
        end_value : f32,   end_control : Control
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Rotation
{
    SingleSingle(Control, Control),
    // DoubleSingle(Merger,  Control), TODO: Implement these items
    // SingleDouble(Control, Merger),
    // DoubleDouble(Merger,  Merger),
}

impl Rotation
{
    fn shader_function_name(&self) -> String
    {
        use filters::Rotation::*;
        match self
        {
            &SingleSingle(ref start, ref end) =>
            {
                format!("{}_{}", start.shader_fn_name(), end.shader_fn_name())
            },
        }
    }

    fn shader_args(&self) -> Vec<String>
    {
        use filters::Rotation::*;
        use filters::Control::*;

        match self
        {
            &SingleSingle(Point(ref start), Point(ref end)) =>
            {
                let (start_x, start_y, start_z) = start.to_hsv_vec();
                let (x, y, z, w) = start.get_rotation(end);
                vec![format!("vec3({}, {}, {})", start_x, start_y, start_z),
                     format!("vec4({}, {}, {}, {})", x, y, z, w)]
            },
        }
    }

    pub fn shader_line(&self, in_name : &String) -> String
    {
        let mut line = format!("{}({}", self.shader_function_name(), in_name);

        for arg in self.shader_args()
        {
            line = format!("{}, {}", line, arg);
        }

        format!("{})", line)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scale
{
    RatioClamp,
    BezierLoose,
    // BezierStrict, TODO: Implement a stricter version of bezier scaling
}

impl Scale
{
    pub fn shader_line(&self, rotation : &Rotation, in_name : &String) -> String
    {
        use filters::Rotation::*;
        use filters::Control::*;
        use filters::Scale::*;

        match self
        {
            &RatioClamp =>
            {
                let rot = rotation.shader_line(in_name);

                match rotation
                {
                    &SingleSingle(Point(ref start), Point(ref end)) =>
                    {
                        format!("RatioClamp({}, {})", rot, end.vec_len() / start.vec_len())
                    }
                }
            },
            &BezierLoose =>
            {
                let rot = rotation.shader_line(in_name);

                match rotation
                {
                    &SingleSingle(Point(ref start), Point(ref end)) =>
                    {
                        format!("BezierLoose({}, {}, {})", rot, start.vec_len(), end.vec_len())
                    }
                }
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Palette
{
    pub filters : Vec<(Rotation, Scale)>,
}

impl Palette
{
    pub fn new() -> Palette
    {
        Palette
        {
            filters : Vec::new()
        }
    }

    pub fn shader(&self, in_name : String, out_name : String,
                         total_name : String, zeros_name : String) -> String
    {
        let mut shader = format!(
"{} = 0;
{} = 0;
{}_vec = vec3(0);
{}_vec = vec3(0);

",
                                 total_name, zeros_name, total_name, zeros_name);

        for (i, &(ref rotation, ref scaler)) in self.filters.iter().enumerate()
        {
            let filter_string = scaler.shader_line(rotation, &in_name);

            shader += &format!(
"vec4 {out}_{i} = {filter};
if ({out}_{i}.w < Epsilon)
{{
  {num_zeros} += 1;
  {num_zeros}_vec += {out}_{i}.xyz;
}}
else
{{
  {total}     += 1 / {out}_{i}.w;
  {total}_vec += 1 / {out}_{i}.w * {out}_{i}.xyz;
}}\n",
                                      out=out_name, i=i,
                                      filter=filter_string,
                                      num_zeros=zeros_name,
                                      total=total_name);
        }

        shader += &format!(
"if ({num_zeros} > 0)
{{
  {out} = {num_zeros}_vec / {num_zeros};
}}
else
{{
  {out} = (1 / {total}) * {total}_vec;
}}",
                                  out=out_name, num_zeros=zeros_name, total=total_name);

        shader
    }
}
