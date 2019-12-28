attribute vec2 position;
attribute float color;

varying vec4 vertexColor;

uniform vec3 colors[100];

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vertexColor = vec4(colors[int(color)], 1.0);
}