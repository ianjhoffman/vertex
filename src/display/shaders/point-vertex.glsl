attribute vec2 position;
attribute float index;

uniform mat4 viewMatrix;
uniform int selected[2];

void main() {
    float point_size = 10.0;
    int index_as_int = int(index);
    if (selected[0] == index_as_int || selected[1] == index_as_int) {
        point_size = 15.0;
    }
    gl_Position = viewMatrix * vec4(position, -1.0, 1.0);
    gl_PointSize = point_size;
}