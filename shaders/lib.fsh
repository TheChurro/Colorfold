#version 150 core

uniform sampler2D img;
uniform img_dims
{
  ivec2 size;
};

in vec2  v_uv_pos;
out vec4 Target0;

// Color conversion code from
// http://lolengine.net/blog/2013/07/27/rgb-to-hsv-in-glsl

vec3 rgb2hsv(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
    vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

// Now for my code

// ====================================================================
// == Conversions between the hsv cube and hsv in spherical notation ==
// ====================================================================

vec3 hsv2half_spherical(vec3 color)
{
  vec3 hue_sat = vec3(sin(color.y) * vec2(cos(color.x), sin(color.x)), cos(color.y));
  return vec3(color.z * hue_sat);
}

vec3 half_spherical2hsv(vec3 color)
{
  return vec3(atan(color.y, color.x), atan(length(color.xy), abs(color.z)), length(color));
}

// ====================================================================
// == Rotation Calculation Routines                                  ==
// ====================================================================

const float Epsilon = 0.0000001;

/**
 * Get quaternion which rotates a given angle about a given axis
 */
vec4 get_axis_angle_quat(vec3 axis, float angle)
{
  return vec4(sin(angle / 2) * axis, cos(angle / 2));
}

/**
 * Calculate the quaternion which rotates start to end
 */
vec4 get_rotation_quat(vec3 start, vec3 end)
{
  if (length(start) <= Epsilon || length(end) <= Epsilon)
    return vec4(0, 0, 0, 1);

  vec3 axis = cross(start, end);
  if (dot(axis, axis) < Epsilon)
  {
    axis = cross(start, vec3(0, 1, 0));
    if (dot(axis, axis) < Epsilon)
    {
      axis = cross(start, vec3(1, 0, 0));
    }
  }
  axis /= length(axis);

  float angle = acos(dot(start, end) / (length(start) * length(end)));
  return vec4(sin(angle / 2) * axis, cos(angle / 2));
}

/**
 * Apply a quaternion rotation to a position vector.
 */
vec3 rotate_by_quat(vec3 position, vec4 quaternion)
{
  vec4 q = quaternion;
  vec3 p = position;
  return p + 2.0 * cross(q.xyz, cross(q.xyz, p) + q.w * p);
}

/**
 * Interpolate using rotations between two vectors
 */
vec3 interp(vec3 start, vec3 end, float percent)
{
  if (length(start) <= Epsilon)
    return end * percent;
  if (length(end) <= Epsilon)
    return start * (1 - percent);

  vec3 axis = cross(start, end);
  if (dot(axis, axis) < Epsilon)
  {
    axis = cross(start, vec3(0, 1, 0));
    if (dot(axis, axis) < Epsilon)
    {
      axis = cross(start, vec3(1, 0, 0));
    }
  }
  axis /= length(axis);
  float angle = acos(dot(start, end) / (length(start) * length(end)));

  vec4 rotation_quat = get_axis_angle_quat(axis, angle * percent);
  float start_length = length(start);
  float new_length    = start_length + (length(end) - start_length) * percent;

  return rotate_by_quat(start * new_length / start_length, rotation_quat);
}

// ====================================================================
// == Palette Transformation Routines                                ==
// ====================================================================

// === Rotations
// ===== All functions return (rotated vector, weight)
// ======= weight = -1 implies that this rotation should not be counted
// ======= as part of the sum.

/**
 * Rotate the input vector by the rotation between start and end
 */
vec4 single_point_rotation(vec3 in_vec, vec3 start, vec3 end)
{
  vec4 rotation = get_rotation_quat(start, end);
  vec3 disp_vec = start - in_vec;
  return vec4(rotate_by_quat(in_vec, rotation), dot(disp_vec, disp_vec));
}

// === Scaling
// ===== All functions return (scaled vector with norm <= 1, weight)
// ======= If weight = -1 then scaling may not occur

/**
 * Scale the input vector by the given ratio.
 */
vec4 clamp_scaling(vec4 in_vec, float ratio)
{
  if (in_vec.w < -Epsilon) return in_vec;

  vec3 scaled = in_vec.xyz * ratio;
  if (dot(scaled, scaled) > 1)
  {
    scaled /= length(scaled);
  }
  return vec4(scaled, in_vec.w);
}

/**
 * Scale the in_vec by the bezier curve given by the controls
 *        (0, 0), (start_mid, end_mid), (1, 1).
 * Note: This scheme will not necessarily map a vector of length
 * start_mid to a vector of length end_mid.
 */
vec4 bezier_scaling(vec4 in_vec, float start_mid, float end_mid)
{
  if (in_vec.w < 0) return in_vec;

  float in_len     = length(in_vec.xyz);
  float percent    = mix(mix(0, start_mid, in_len),
                         mix(start_mid, 1, in_len), in_len);
  float new_length = mix(mix(0, end_mid, percent),
                         mix(end_mid, 1, percent), percent);
  return vec4(new_length / in_len * in_vec.xyz, in_vec.w);
}

void main()
{
  // Calculate Texel Index to remove filtering, get color of the image and
  // then translate it into hsv spherical coordinates
  ivec2 texel_index = ivec2(v_uv_pos * size);
  vec4 orig_color = texelFetch(img, texel_index, 0);
  vec3 color_vec = hsv2half_spherical(rgb2hsv(orig_color.xyz));
