float LinearizeDepth(float depth, float near, float far) {
    float z = depth * 2.0 - 1.0; // NDC
    return (2.0 * near * far) / (far + near - z * (far - near));
}

vec3 WorldPosFromDepth(vec2 uv, float depth, mat4 inverseVP) {
    vec4 ndc = vec4(uv * 2.0 - 1.0, depth * 2.0 - 1.0, 1.0);
    vec4 wp = inverseVP * ndc;
    return wp.xyz / wp.w;
}
