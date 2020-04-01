#version 450

layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform Uniforms {
    vec3 color;
    vec2 resolution;
};

void main() {
    outColor.rgb = vec3(gl_FragCoord.x > resolution.x ? 1 : 0);
    outColor.r = (gl_FragCoord.x / resolution.x).x;
    outColor.a = 1;
}
