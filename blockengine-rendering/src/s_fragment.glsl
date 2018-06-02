#version 150 core

uniform sampler2D u_texture;

in vec2 v_tex_coord;
out vec4 o_color;

void main() {
    vec4 tex = texture(u_texture, v_tex_coord);
    float blend = dot(v_tex_coord-vec2(0.5,0.5), v_tex_coord-vec2(0.5,0.5));
    o_color = vec4(mix(tex.rgb, vec3(0.0, 0.0, 0.0), blend*1.0), tex.a);
    //o_color = texture(u_texture, v_tex_coord);
}
