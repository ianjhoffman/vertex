attribute vec2 position;
attribute float index;

uniform mat4 viewMatrix;

void main() {
    // Just so index is used and not optimized away
    float z = -1.0;
    if (int(index) == 2000) {
        z = -1.0;
    }
    gl_Position = viewMatrix * vec4(position, z, 1.0);
    gl_PointSize = 5.0;
}