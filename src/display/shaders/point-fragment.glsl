precision highp float;

varying vec2 fragmentUV;
varying vec3 centerColor;

void main() {
    float alpha = 1.0;
    if (distance(vec2(0.5, 0.5), fragmentUV) >= 0.5) {
        alpha = 0.0;
    }
    vec3 color = centerColor;
    if (distance(vec2(0.5, 0.5), fragmentUV) >= 0.4) {
        color = vec3(0.0);
    }
    gl_FragColor = vec4(color, alpha);
} 