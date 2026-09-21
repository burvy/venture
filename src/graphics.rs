use crate::{
    systems,
    window::{App, Graphics},
};
use image::RgbaImage;

use std::sync::OnceLock;

static SPRITES: OnceLock<Sprites> = OnceLock::new();

/// struct to hold the predefined sprites, not
/// general sprites
pub struct Sprites {
    pub red_troop: Sprite,
    pub blue_troop: Sprite,
    pub red_edit_mode: Sprite,
    pub blue_edit_mode: Sprite,
    pub obstacle_mode: Sprite,
    pub delete_mode: Sprite,
    pub change_mode: Sprite,
    pub playing: Sprite,
    pub paused: Sprite,
    pub delete: Sprite,
    pub obstacle_mode_button: Sprite,
    pub dpad_right: Sprite,
}

impl Sprites {
    fn load() -> Self {
        Sprites {
            red_troop: Sprite::from_bytes(include_bytes!("../assets/images/red-troop.png")),
            blue_troop: Sprite::from_bytes(include_bytes!("../assets/images/blue-troop.png")),
            red_edit_mode: Sprite::from_bytes(include_bytes!("../assets/images/red-edit-mode.png")),
            blue_edit_mode: Sprite::from_bytes(include_bytes!(
                "../assets/images/blue-edit-mode.png"
            )),
            obstacle_mode: Sprite::from_bytes(include_bytes!("../assets/images/obstacle-mode.png")),
            delete_mode: Sprite::from_bytes(include_bytes!("../assets/images/delete-mode.png")),
            change_mode: Sprite::from_bytes(include_bytes!("../assets/images/change-mode.png")),
            playing: Sprite::from_bytes(include_bytes!("../assets/images/playing.png")),
            paused: Sprite::from_bytes(include_bytes!("../assets/images/paused.png")),
            delete: Sprite::from_bytes(include_bytes!("../assets/images/delete.png")),
            obstacle_mode_button: Sprite::from_bytes(include_bytes!(
                "../assets/images/obstacle-mode-button.png"
            )),
            dpad_right: Sprite::from_bytes(include_bytes!("../assets/images/dpad-right.png")),
        }
    }
}

pub fn sprites() -> &'static Sprites {
    SPRITES.get_or_init(Sprites::load)
}

fn troop_sprite(world: &systems::World, entity: systems::Entity) -> Option<&'static Sprite> {
    match world.teams.get(&entity)? {
        systems::Team::RED => Some(&sprites().red_troop),
        systems::Team::BLUE => Some(&sprites().blue_troop),
    }
}
impl Graphics {
    fn draw_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let size = self.pixels.texture().size();

        if x >= size.width || y >= size.height {
            return;
        }

        let index = ((y * size.width + x) << 2) as usize;

        self.pixels.frame_mut()[index..index + 4].copy_from_slice(&color)
    }

    fn draw_sprite(&mut self, x: u32, y: u32, sprite: &Sprite) {
        for i in 0..sprite.width {
            for j in 0..sprite.height {
                if let Some(color) = sprite.get_pixel_color(i, j) {
                    self.draw_pixel(x + i, y + j, color);
                }
            }
        }
    }
    fn draw_sprite_rotated(&mut self, x: u32, y: u32, sprite: &Sprite, angle: f64) {
        let (center_x, center_y) = (sprite.width as f64 / 2.0, sprite.height as f64 / 2.0);
        let (cos_a, sin_a) = (angle.cos(), angle.sin());

        for i in 0..sprite.width {
            for j in 0..sprite.height {
                // the offset of the pixel from the center
                let offset_x = i as f64 - center_x;
                let offset_y = j as f64 - center_y;

                // rotate backwards to find which source pixel belongs here
                // this is the inverse 2D rotation matrix, best to look it
                // up if you want to prove it.
                let og_x = offset_x * cos_a + offset_y * sin_a + center_x;
                let og_y = -offset_x * sin_a + offset_y * cos_a + center_y;

                // don't draw out of bounds
                let rounded_x = og_x.round();
                let rounded_y = og_y.round();
                if rounded_x < 0.0
                    || rounded_y < 0.0
                    || rounded_x >= sprite.width as f64
                    || rounded_y >= sprite.height as f64
                {
                    continue;
                }

                if let Some(color) = sprite.get_pixel_color(rounded_x as u32, rounded_y as u32) {
                    self.draw_pixel(x + i, y + j, color);
                }
            }
        }
    }
}

pub struct Sprite {
    pub width: u32,
    pub height: u32,
    pixels: Vec<u8>,
}

impl Sprite {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let img: RgbaImage = image::load_from_memory(bytes)
            .expect("invalid sprite data")
            .to_rgba8();
        Sprite {
            width: img.width(),
            height: img.height(),
            pixels: img.into_raw(),
        }
    }

    /// Gets the color of a pixel from the sprite
    fn get_pixel_color(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        let pix = ((y * self.width + x) << 2) as usize;
        let alpha = self.pixels[pix + 3];
        if alpha == 0 {
            return None;
        }
        Some([
            self.pixels[pix],
            self.pixels[pix + 1],
            self.pixels[pix + 2],
            self.pixels[pix + 3],
        ])
    }
}

pub fn draw_fn(app: &mut App) {
    let Some(graphics) = app.graphics.as_mut() else {
        return;
    };
    let Some(game_state) = app.game_state.as_mut() else {
        return;
    };

    // LOADING SPRITES
    let sprites = sprites();

    // DRAWING SPRITES
    graphics.draw_sprite(0, 0, &sprites.change_mode);
    match game_state.paused {
        true => graphics.draw_sprite(512, 0, &sprites.paused),
        false => graphics.draw_sprite(512, 0, &sprites.playing),
    }
    graphics.draw_sprite(1024, 0, &sprites.delete);
    if game_state.mode == systems::Mode::ERASE {
        graphics.draw_sprite(0, 128, &sprites.delete_mode);
    } else {
        match game_state.team_mode {
            systems::Team::RED => graphics.draw_sprite(0, 128, &sprites.red_edit_mode),
            systems::Team::BLUE => graphics.draw_sprite(0, 128, &sprites.blue_edit_mode),
        }
    }
    graphics.draw_sprite(1536, 0, &sprites.obstacle_mode_button);
    graphics.draw_sprite(1536, 1280, &sprites.dpad_right);

    // DRAWING TROOP SPRITES
    for (&entity, pos) in game_state.world.positions.iter() {
        let Some(screen_x) = pos.x.checked_sub(game_state.camera.x) else {
            continue;
        };
        let Some(screen_y) = pos.y.checked_sub(game_state.camera.y) else {
            continue;
        };
        let Some(sprite) = troop_sprite(&game_state.world, entity) else {
            continue;
        };
        // troop sprites can be rotated
        let rotation = game_state
            .world
            .rotations
            .get(&entity)
            .copied()
            .unwrap_or(0.0);
        graphics.draw_sprite_rotated(screen_x, screen_y, sprite, rotation);
    }
}

pub fn troop_at(world: &systems::World, x: u32, y: u32) -> Option<systems::Entity> {
    for (&entity, pos) in world.positions.iter() {
        let Some(sprite) = troop_sprite(world, entity) else {
            continue;
        };
        if x >= pos.x && x < pos.x + sprite.width && y >= pos.y && y < pos.y + sprite.height {
            return Some(entity);
        }
    }
    None
}

/// definition of some buttons with logic
pub fn buttons(game_state: &systems::GameState) -> [systems::Button; 4] {
    let sprites = sprites();
    let play_pause = if game_state.paused {
        &sprites.paused
    } else {
        &sprites.playing
    };

    [
        systems::Button {
            x: 0,
            y: 0,
            width: sprites.change_mode.width,
            height: sprites.change_mode.height,
            on_click: systems::GameState::change_teams,
        },
        systems::Button {
            x: 512,
            y: 0,
            width: play_pause.width,
            height: play_pause.height,
            on_click: systems::GameState::toggle_pause,
        },
        systems::Button {
            x: 1024,
            y: 0,
            width: sprites.delete.width,
            height: sprites.delete.height,
            on_click: systems::GameState::toggle_delete,
        },
        systems::Button {
            x: 1536,
            y: 0,
            width: sprites.obstacle_mode_button.width,
            height: sprites.obstacle_mode_button.height,
            on_click: systems::GameState::toggle_paint,
        },
    ]
}
