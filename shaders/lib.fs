
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

#define M_PI 3.1415926535897932384626433832795

vec4 hsv2half_spherical(vec3 color)
{
  float sat_angle = color.y * M_PI / 2.0;
  float hue_angle = color.x * 2.0 * M_PI;
  vec3 hue_sat = vec3(sin(sat_angle) * vec2(cos(hue_angle), sin(hue_angle)), cos(sat_angle));
  return vec4(color.z * hue_sat, 0.0);
}

vec3 half_spherical2hsv(vec3 color)
{
  float hue_angle = atan(color.y, color.x);
  float sat_angle = atan(length(color.xy), abs(color.z));

  if (hue_angle < 0.0)
  {
    hue_angle += 2.0 * M_PI;
  }

  return vec3(hue_angle / (2.0 * M_PI), sat_angle * 2.0 / M_PI, length(color));
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
  return vec4(sin(angle / 2.0) * axis, cos(angle / 2.0));
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
  return vec4(sin(angle / 2.0) * axis, cos(angle / 2.0));
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
 * Interpolate using rotations between two vectors checking for null values.
 * Assuming non-null input vectors (w >= -0.5) will always return non-null output
 */
vec4 lin_interp(vec4 start_4, vec4 end_4, float percent)
{
  if (start_4.w < -0.5 || end_4.w < -0.5)
  {
    return vec4(0, 0, 0, -1);
  }
  return mix(start_4, end_4, percent);
}

/**
 * This is the same as above but returns null if percent is not in [0, 1]
 */
vec4 lin_interp_bounded(vec4 start_4, vec4 end_4, float percent)
{
  if (percent < 0.0 || percent > 1.0)
  {
    return vec4(0, 0, 0, -1);
  }
  return mix(start_4, end_4, percent);
}

/**
 * Interpolate using rotations between two vectors checking for null values.
 * Assuming non-null input vectors (w >= -0.5) will always return non-null output
 */
vec4 rot_interp(vec4 start_4, vec4 end_4, float percent)
{
  if (start_4.w < -0.5 || end_4.w < -0.5)
  {
    return vec4(0, 0, 0, -1);
  }
  vec3 start = start_4.xyz;
  vec3 end   = end_4.xyz;

  if (length(start) <= Epsilon)
    return vec4(end * percent, 0);
  if (length(end) <= Epsilon)
    return vec4(start * (1.0 - percent), 0);

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

  return vec4(rotate_by_quat(start * new_length / start_length, rotation_quat), 0);
}

/**
 * This is the same as above but returns null if percent is not in [0, 1]
 */
vec4 rot_interp_bounded(vec4 start, vec4 end, float percent)
{
  if (percent < 0.0 || percent > 1.0)
  {
    return vec4(0, 0, 0, -1);
  }
  return rot_interp(start, end, percent);
}

// ====================================================================
// == Palette Transformation Routines                                ==
// ====================================================================

// === Rotations
// ===== All functions return (rotated vector, weight)
// ======= weight = -1 implies that this rotation should not be counted
// ======= as part of the sum.

// /**
//  * Rotate the input vector by the given rotation
//  */
// vec4 point_point(vec3 in_vec, vec3 start, vec4 rotation)
// {
//   vec3 dist_vec = start - in_vec;
//   return vec4(rotate_by_quat(in_vec, rotation), dot(dist_vec, dist_vec));
// }

/**
 * Rotate the input vector by the rotation between start and end. Returns null
 * if given null.
 */
vec4 point_point(vec4 in_vec, vec4 start, vec4 end)
{
  if (in_vec.w < -0.5 || start.w < -0.5 || end.w < -0.5)
    return vec4(0, 0, 0, -1);

  vec4 rotation = get_rotation_quat(start.xyz, end.xyz);
  vec3 dist_vec = start.xyz - in_vec.xyz;
  return vec4(rotate_by_quat(in_vec.xyz, rotation), dot(dist_vec, dist_vec));
}

// === Scaling
// ===== All functions return (scaled vector with norm <= 1, weight)

/**
 * Clamps vectors with too much length and does nothing else.
 */
vec4 Clamp(vec4 in_vec, float start, float end)
{
  vec3 position = in_vec.xyz;
  if (dot(position, position) > 1.0)
  {
    position /= length(position);
  }
  return vec4(position, in_vec.w);
}

/**
 * Scale the input vector by the given ratio.
 */
vec4 RatioClamp(vec4 in_vec, float start, float end)
{
  if (start < Epsilon)
  {
    return vec4(0, 0, 0, in_vec.w);
  }
  else
  {
    float ratio = end / start;
    vec3 scaled = in_vec.xyz * ratio;
    if (dot(scaled, scaled) > 1.0)
    {
      scaled /= length(scaled);
    }
    return vec4(scaled, in_vec.w);
  }
}

/**
 * Scale the in_vec by the bezier curve given by the controls
 *        (0, 0), (start_mid, end_mid), (1, 1).
 * Note: This scheme will not necessarily map a vector of length
 * start_mid to a vector of length end_mid.
 */
vec4 BezierLoose(vec4 in_vec, float start_mid, float end_mid)
{
  float in_len     = length(in_vec.xyz);
  if (in_len < Epsilon) return vec4(0, 0, 0, in_vec.w);

  float percent    = mix(mix(0.0, start_mid, in_len),
                         mix(start_mid, 1.0, in_len), in_len);
  float new_length = mix(mix(0.0, end_mid, percent),
                         mix(end_mid, 1.0, percent), percent);
  return vec4(new_length / in_len * in_vec.xyz, in_vec.w);
}
