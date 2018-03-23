# version 150 core

uniform sampler2D img;
in vec2  v_uv_pos;
out vec4 Target0;


void main()
{
  vec4 color_out = texture(img, v_uv_pos);
  Target0 = color_out;
}
