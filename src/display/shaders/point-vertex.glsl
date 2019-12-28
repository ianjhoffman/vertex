attribute vec2 position;
attribute float index;

void main() {
    // Just so index is used and not optimized away
    float z = 0.0;
    if (int(index) == 2000) {
        z = 0.0;
    }
    gl_Position = vec4(position, z, 1.0);
    gl_PointSize = 5.0;
}