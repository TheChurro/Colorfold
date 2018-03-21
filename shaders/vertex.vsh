#version 150

in vec2  pos;
in vec2  uv_pos;
out vec2 v_uv_pos;

void main()
{
  v_uv_pos = uv_pos;
  gl_Position = vec4(pos, 0.0, 1.0);
}
