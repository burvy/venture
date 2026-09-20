# Venture

This is a small 2D battle simulator type game built for the web! 
You can play it at [burvy.dev/venture](https://burvy.dev/venture).

# Architecture
This uses the same core as my `life-v2` project, using `winit` and `pixels` 
for rendering, but also a few other crates for audio and sprite loading.

The troops themselves are powered by an Entity Component System, 
inspired by `bevy`.
