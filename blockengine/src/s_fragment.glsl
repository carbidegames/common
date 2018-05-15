#version 150 core

uniform Locals {
    mat4 u_transform;
};

in vec4 a_pos;
in vec2 a_tex_coord;
out vec2 v_tex_coord;

void main() {
    v_tex_coord = a_tex_coord;
    gl_Position = u_transform * a_pos;
    gl_ClipDistance[0] = 1.0;
}
