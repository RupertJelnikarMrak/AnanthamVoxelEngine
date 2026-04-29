#version 460
#extension GL_EXT_mesh_shader : require

layout(location = 0) in vec3 v_color;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec2 v_size;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 final_color = v_color;

    // Adjust this to make the lines thicker or thinner
    float line_thickness = 0.03;

    // 1. Is the pixel on the absolute border of the greedy quad?
    bool quad_border = (v_uv.x < line_thickness * 2.0 || v_uv.x > v_size.x - line_thickness * 2.0 ||
            v_uv.y < line_thickness * 2.0 || v_uv.y > v_size.y - line_thickness * 2.0);

    // 2. Is the pixel on the border of a 1x1 block?
    // fract() turns 2.99 into 0.99, and 3.01 into 0.01.
    vec2 fract_uv = fract(v_uv);
    bool block_border = (fract_uv.x < line_thickness || fract_uv.x > 1.0 - line_thickness ||
            fract_uv.y < line_thickness || fract_uv.y > 1.0 - line_thickness);

    // Apply colors (Quad borders overwrite block borders)
    if (quad_border) {
        final_color = vec3(1.0, 1.0, 0.0); // Bright Yellow for Quad Bounds
    } else if (block_border) {
        final_color = vec3(0.0, 0.0, 0.0); // Black for 1x1 Block Bounds
    }

    out_color = vec4(final_color, 1.0);
}
