use crate::{GameError, GameResult};


// TODO: implement useful shader

/// Default vertex shader that will be used for rendering textured meshes in 2D.
pub const DEFAULT_VERTEX_SHADER: &str = r"#version 330 core
layout (location = 0) in vec2 pos;

void main()
{
    gl_Position = vec4(pos.x, pos.y,0.0, 1.0);
}";

/// Default fragment shader that will be used for rendering textured meshes in 2D.
pub const DEFAULT_FRAGMENT_SHADER: &str = r"#version 330 core
out vec4 FragColor;

void main()
{
    FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
}";
