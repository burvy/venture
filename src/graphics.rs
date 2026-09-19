use crate::{
    systems,
    window::{App, Graphics},
};
use image::RgbaImage;

use std::sync::OnceLock;

static RED_TROOP: OnceLock<Sprite> = OnceLock::new();
static RED_EDIT_MODE: OnceLock<Sprite> = OnceLock::new();
static BLUE_EDIT_MODE: OnceLock<Sprite> = OnceLock::new();

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
                let pix = ((j * sprite.width + i) << 2) as usize;
                let alpha = sprite.pixels[pix + 3];
                if alpha == 0 {
                    continue;
                }
                let color = [
                    sprite.pixels[pix],
                    sprite.pixels[pix + 1],
                    sprite.pixels[pix + 2],
                    sprite.pixels[pix + 3],
                ];
                self.draw_pixel(x + i, y + j, color);
            }
        }
    }
}

struct Sprite {
    width: u32,
    height: u32,
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
}

pub fn draw_fn(app: &mut App) {
    let Some(graphics) = app.graphics.as_mut() else {
        return;
    };
    let Some(game_state) = app.game_state.as_mut() else {
        return;
    };
    let red_troop = RED_TROOP
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/red-troop.png")));
    let red_edit_mode = RED_EDIT_MODE
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/red-edit-mode.png")));
    let blue_edit_mode = BLUE_EDIT_MODE
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/blue-edit-mode.png")));

    match game_state.edit_mode {
        systems::Team::RED => graphics.draw_sprite(0, 0, red_edit_mode),
        systems::Team::BLUE => graphics.draw_sprite(0, 0, blue_edit_mode),
    }

    let size = graphics.pixels.texture().size();
    let x = (size.width - red_troop.width) / 2;
    let y = (size.height - red_troop.height) / 2;
    graphics.draw_sprite(x, y, red_troop);
}
