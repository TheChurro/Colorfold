extern crate glsl_to_spirv;

#[test]
fn test1() {
    let shader = r#"
#version 330
layout(location = 0) out vec4 f_color;
void main() {
    f_color = vec4(1.0);
}
"#;

    println!("{}", glsl_to_spirv::compile(shader, glsl_to_spirv::ShaderType::Fragment).unwrap());
}
