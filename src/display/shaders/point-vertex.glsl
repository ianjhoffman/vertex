attribute vec2 position;
attribute vec2 uv;
attribute float texture_index;

varying vec2 fragmentUV;
varying vec3 centerColor;

uniform mat4 viewMatrix;

void main() {
    fragmentUV = uv;
    if (int(texture_index) == 0) {
        centerColor = vec3(1.0, 0.3, 0.3);
    } else {
        centerColor = vec3(1.0);
    }
    gl_Position = viewMatrix * vec4(position, -1.0, 1.0);
}