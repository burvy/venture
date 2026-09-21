use crate::{
    systems,
    window::{App, Graphics},
};
use image::RgbaImage;

use std::{
    f64::consts::{FRAC_PI_2, PI},
    sync::OnceLock,
};

static SPRITES: OnceLock<Sprites> = OnceLock::new();

/// struct to hold the predefined sprites, not
/// general sprites
pub struct Sprites {
    pub red_troop: Sprite,
    pub blue_troop: Sprite,
    pub red_edit_mode: Sprite,
    pub blue_edit_mode: Sprite,
    pub obstacle_mode: Sprite,
    pub erase_mode: Sprite,
    pub change_mode: Sprite,
    pub playing: Sprite,
    pub paused: Sprite,
    pub erase: Sprite,
    pub obstacle_mode_button: Sprite,
    pub dpad_arrow: Sprite,
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
            erase_mode: Sprite::from_bytes(include_bytes!("../assets/images/erase-mode.png")),
            change_mode: Sprite::from_bytes(include_bytes!("../assets/images/change-mode.png")),
            playing: Sprite::from_bytes(include_bytes!("../assets/images/playing.png")),
            paused: Sprite::from_bytes(include_bytes!("../assets/images/paused.png")),
            erase: Sprite::from_bytes(include_bytes!("../assets/images/erase.png")),
            obstacle_mode_button: Sprite::from_bytes(include_bytes!(
                "../assets/images/obstacle-mode-button.png"
            )),
            dpad_arrow: Sprite::from_bytes(include_bytes!("../assets/images/dpad-arrow.png")),
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
        let out = &mut self.pixels.frame_mut()[index..index + 4];

        let alpha = color[3] as f64 / 255.0;
        for channel in 0..3 {
            out[channel] =
                (color[channel] as f64 * alpha + out[channel] as f64 * (1.0 - alpha)) as u8;
        }
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
    fn draw_sprite_rotated(&mut self, x: u32, y: u32, sprite: &Sprite, angle: f64, scale: f64) {
        let (center_x, center_y) = (sprite.width as f64 / 2.0, sprite.height as f64 / 2.0);
        let (cos_a, sin_a) = (angle.cos(), angle.sin());
        let scaled_width = (sprite.width as f64 * scale).round() as u32;
        let scaled_height = (sprite.height as f64 * scale).round() as u32;

        for i in 0..scaled_width {
            for j in 0..scaled_height {
                // the offset of the pixel from the center
                let offset_x = i as f64 - center_x * scale;
                let offset_y = j as f64 - center_y * scale;

                // rotate backwards to find which source pixel belongs here
                // this is the inverse 2D rotation matrix, best to look it
                // up if you want to prove it. Plus there is a scale factor
                let og_x = (offset_x * cos_a + offset_y * sin_a) / scale + center_x;
                let og_y = (-offset_x * sin_a + offset_y * cos_a) / scale + center_y;

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
        // pixel buffer calculation
        let pix = ((y * self.width + x) << 2) as usize;
        let alpha = self.pixels[pix + 3];
        if alpha == 0 {
            return None;
        }
        Some([
            self.pixels[pix],
            self.pixels[pix + 1],
            self.pixels[pix + 2],
            alpha,
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
    graphics.draw_sprite(1024, 0, &sprites.erase);
    match game_state.mode {
        systems::Mode::DEPLOY => match game_state.team_mode {
            systems::Team::RED => graphics.draw_sprite(0, 128, &sprites.red_edit_mode),
            systems::Team::BLUE => graphics.draw_sprite(0, 128, &sprites.blue_edit_mode),
        },
        systems::Mode::PAINT => graphics.draw_sprite(0, 128, &sprites.obstacle_mode),
        systems::Mode::ERASE => graphics.draw_sprite(0, 128, &sprites.erase_mode),
    }
    graphics.draw_sprite(1536, 0, &sprites.obstacle_mode_button);
    // Draw DPad buttons
    let size = graphics.pixels.texture().size();
    for button in dpad_buttons(size.width, size.height) {
        let rotation = match button.direction {
            systems::PanDirection::RIGHT => 0.0,
            systems::PanDirection::DOWN => FRAC_PI_2,
            systems::PanDirection::LEFT => PI,
            systems::PanDirection::UP => -FRAC_PI_2,
        };
        let scale = button.width as f64 / sprites.dpad_arrow.width as f64;
        graphics.draw_sprite_rotated(button.x, button.y, &sprites.dpad_arrow, rotation, scale);
    }

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
        graphics.draw_sprite_rotated(screen_x, screen_y, sprite, rotation, 1.0);
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
            width: sprites.erase.width,
            height: sprites.erase.height,
            on_click: systems::GameState::toggle_erase,
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

/// Interactable DPad buttons for mobile
pub fn dpad_buttons(screen_width: u32, screen_height: u32) -> [systems::DPadButton; 4] {
    let sprites = sprites();
    let (sprite_w, sprite_h) = (sprites.dpad_arrow.width, sprites.dpad_arrow.height);

    let quarter_width = screen_width / 2;
    let quarter_height = screen_height / 2;
    // scale to the smaller axis
    let scale = (quarter_width as f64 / (sprite_w as f64 * 2.0))
        .min(quarter_height as f64 / (sprite_h as f64 * 2.0));

    let w = (sprite_w as f64 * scale).round() as u32;
    let h = (sprite_h as f64 * scale).round() as u32;

    let quadrant_center_x = screen_width / 2 + screen_width / 4;
    let quadrant_center_y = screen_height / 2 + screen_height / 4;

    let center_x = quadrant_center_x.saturating_sub(w / 2);
    let center_y = quadrant_center_y.saturating_sub(h / 2);

    [
        systems::DPadButton {
            x: center_x,
            y: center_y.saturating_sub(h),
            width: w,
            height: h,
            direction: systems::PanDirection::UP,
        },
        systems::DPadButton {
            x: center_x,
            y: center_y + h,
            width: w,
            height: h,
            direction: systems::PanDirection::DOWN,
        },
        systems::DPadButton {
            x: center_x.saturating_sub(w),
            y: center_y,
            width: w,
            height: h,
            direction: systems::PanDirection::LEFT,
        },
        systems::DPadButton {
            x: center_x + w,
            y: center_y,
            width: w,
            height: h,
            direction: systems::PanDirection::RIGHT,
        },
    ]
}
