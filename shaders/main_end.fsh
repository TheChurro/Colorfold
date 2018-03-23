  // Convert the out_color back into rgb. Maintain alpha.
  vec4 color_out = vec4(hsv2rgb(half_spherical2hsv(out_vec)),
                      orig_color.w);
  Target0 = color_out;
}
