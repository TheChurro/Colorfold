
use geometry::Geom0D;

use std::collections::HashSet;
use linked_hash_set::LinkedHashSet;

#[derive(Clone, Serialize, Deserialize)]
pub enum Summation
{
    InvWeighted
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Scale
{
    Clamp,
    RatioClamp,
    BezierLoose,
    // BezierStrict, TODO: Implement a stricter version of bezier scaling
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Compute
{
    Compute  { name : String, operations : Vec<Compute>, sum_type : Summation },
    Rotation { start_point : Geom0D, end_point : Geom0D, source : String, rescale : Scale },
}

impl Compute
{
    pub fn is_compute(&self) -> bool
    {
        use filters::Compute::Compute;
        if let &Compute {..} = self { true } else { false }
    }

    pub fn get_file(&self) -> String
    {
        use filters::Compute::*;
        match self
        {
            &Compute { ref name , .. } => name.clone(),
            &Rotation { ref source, .. } => source.clone()
        }
    }

    pub fn get_required_sources(&self) -> HashSet<String>
    {
        use filters::Compute::*;
        let mut sources = HashSet::new();
        match self
        {
            &Compute  { name:ref _name, ref operations, sum_type:ref _sum_type } =>
            {
                for op in operations
                {
                    sources = sources.union(&op.get_required_sources()).cloned().collect();
                }
            },
            &Rotation { ref start_point, ref end_point, ref source, rescale:ref _rescale } =>
            {
                sources.insert(source.clone());
                sources = sources.union(&start_point.get_required_sources()).cloned().collect();
                sources = sources.union(&end_point.get_required_sources()).cloned().collect();
            }
        }
        sources
    }

    pub fn get_params(&self) -> LinkedHashSet<String>
    {
        use filters::Compute::*;
        let mut sources = LinkedHashSet::new();
        match self
        {
            &Compute  { name:ref _name, ref operations, sum_type:ref _sum_type } =>
            {
                for op in operations
                {
                    sources = sources.union(&op.get_params()).cloned().collect();
                }
            },
            &Rotation { ref start_point, ref end_point, ref source, rescale:ref _rescale } =>
            {
                sources.insert(source.clone());
                sources = sources.union(&start_point.get_params()).cloned().collect();
                sources = sources.union(&end_point.get_params()).cloned().collect();
            }
        }
        sources
    }

    // Return a list of strings.
    // The first string is the line to call this compute shader
    // The rest of the items are fully written out definitions for required compute shaders.
    // Print these in inverse order.
    pub fn get_shader(&self) -> Vec<String>
    {
        use filters::Compute::*;
        match self
        {
            &Compute  { ref name, ref operations, sum_type:ref _sum_type } =>
            {
                let params = self.get_params();
                let mut call_line = format!("{name}(", name=name);
                let mut function_def = format!("vec4 {name}(", name=name);
                for (i, param) in params.iter().enumerate()
                {
                    if i == 0
                    {
                        function_def = format!("{function_def}vec4 {param}", function_def=function_def, param=param);
                        call_line = format!("{call_line}{param}", call_line=call_line, param=param);
                    }
                    else
                    {
                        function_def = format!("{function_def},vec4 {param}", function_def=function_def, param=param);
                        call_line = format!("{call_line},{param}", call_line=call_line, param=param);
                    }

                }
                function_def += ")
{
  float total_inv_weight = 0;
  vec3 total_inv_weight_vecs = vec3(0);
  int num_zeros = 0;
  vec3 total_zeros = vec3(0);
  vec4 _rot_start_ = vec4(0);
  vec4 _rot_end_ = vec4(0);";
                call_line += ")";
                let mut inner_compute_functions = Vec::new();

                for op in operations
                {
                    // Get the returned shaders
                    let mut returned_shaders : Vec<String> = op.get_shader().iter().cloned().collect();
                    let first_line = returned_shaders.remove(0);

                    match op
                    {
                        &Compute { name: ref inner_name, .. } =>
                        {
                            function_def += &format!(
"
vec4 {name}_rot = {function_call};
if ({name}_rot.w > -0.5)
{{
    if ({name}_rot.w < Epsilon)
    {{
        num_zeros += 1;
        total_zeros += {name}_rot.xyz;
    }}
    else
    {{
        total_inv_weight += 1 / {name}_rot.w;
        total_inv_weight_vecs += 1 / {name}_rot.w * {name}_rot.xyz;
    }}
}}",
                                name=inner_name, function_call=first_line);
                            inner_compute_functions.append(&mut returned_shaders);
                        },
                        &Rotation { .. } =>
                        {
                            function_def += &first_line;
                        }
                    }
                }

                function_def +=
"
  if (num_zeros > 0)
  {
    return vec4(total_zeros / num_zeros, 0);
  }
  else if (total_inv_weight > Epsilon)
  {
      return vec4((1 / total_inv_weight) * total_inv_weight_vecs, 0);
  }
  else
  {
      return vec4(0, 0, 0, -1);
  }
}";

                inner_compute_functions.insert(0, function_def);
                inner_compute_functions.insert(0, call_line);
                inner_compute_functions

            },
            &Rotation { ref start_point, ref end_point, ref source, ref rescale } =>
            {
                vec![format!(
"
_rot_start_ = {start};
_rot_end_ = {end};
if (_rot_start_.w > -0.5 && _rot_end_.w > -0.5)
{{
    vec4 {source}_rot = {rescale:?}(point_point({source}, _rot_start_, _rot_end_), length(_rot_start_.xyz), length(_rot_end_.xyz));
    if ({source}_rot.w > -0.5)
    {{
        if ({source}_rot.w < Epsilon)
        {{
            num_zeros += 1;
            total_zeros += {source}_rot.xyz;
        }}
        else
        {{
            total_inv_weight += 1 / {source}_rot.w;
            total_inv_weight_vecs += 1 / {source}_rot.w * {source}_rot.xyz;
        }}
    }}
}}",
                start=start_point.get_shader(), end=end_point.get_shader(), rescale=rescale, source=source)]
            }
        }
    }
}
