attribute vec2 position;

uniform mat4 viewMatrix;

void main() {
    gl_Position = viewMatrix * vec4(position, -1.0, 1.0);
}