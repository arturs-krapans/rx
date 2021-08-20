uniform sampler2D framebuffer;

in  vec2 f_uv;
out vec4 fragColor;

void main() {
	fragColor = texture(
		framebuffer,
		vec2(f_uv.s, 1 - f_uv.t)
	);
}
