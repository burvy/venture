# Venture

This is a small 2D battle simulator type game built for the web! 
You can play it at [burvy.dev/venture](https://burvy.dev/venture).

# Architecture
This uses the same core as my `life-v2` project, using `winit` and `pixels` 
for rendering, but also a few other crates for audio and sprite loading.

The troops themselves are powered by an Entity Component System, 
inspired by `bevy`.

# Sprite Sizes
This is for ME only, because I'm only publishing the core of the game, 
but the buttons at the top left are `512x128`. The little guys are 
`64x64`
