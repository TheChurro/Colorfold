
void main()
{
  // Calculate Texel Index to remove filtering, get color of the image and
  // then translate it into hsv spherical coordinates
  ivec2 texel_index = ivec2(gl_FragCoord);
  vec4 orig_color = texelFetch(img, texel_index, 0);
  vec3 color_vec = hsv2half_spherical(rgb2hsv(orig_color.xyz));

  int num_zeros = 0;
  vec3 num_zeros_vec = vec3(0);
  float total = 0;
  vec3 total_vec = vec3(0);

  vec3 out_vec;
