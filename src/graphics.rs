use crate::{
    systems,
    window::{App, Graphics},
};
use image::RgbaImage;

use std::sync::OnceLock;

pub static RED_TROOP: OnceLock<Sprite> = OnceLock::new();
pub static BLUE_TROOP: OnceLock<Sprite> = OnceLock::new();
static RED_EDIT_MODE: OnceLock<Sprite> = OnceLock::new();
static BLUE_EDIT_MODE: OnceLock<Sprite> = OnceLock::new();
static DELETE_MODE: OnceLock<Sprite> = OnceLock::new();
static CHANGE_MODE: OnceLock<Sprite> = OnceLock::new();
static PLAYING: OnceLock<Sprite> = OnceLock::new();
static PAUSED: OnceLock<Sprite> = OnceLock::new();
static DELETE: OnceLock<Sprite> = OnceLock::new();

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
    let blue_troop = BLUE_TROOP
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/blue-troop.png")));
    let red_edit_mode = RED_EDIT_MODE
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/red-edit-mode.png")));
    let blue_edit_mode = BLUE_EDIT_MODE
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/blue-edit-mode.png")));
    let delete_mode = DELETE_MODE
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/delete-mode.png")));
    let change_mode = CHANGE_MODE
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/change-mode.png")));
    let playing =
        PLAYING.get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/playing.png")));
    let paused =
        PAUSED.get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/paused.png")));
    let delete =
        DELETE.get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/delete.png")));

    graphics.draw_sprite(0, 0, change_mode);
    match game_state.paused {
        true => graphics.draw_sprite(512, 0, paused),
        false => graphics.draw_sprite(512, 0, playing),
    }
    graphics.draw_sprite(1024, 0, delete);
    if game_state.deleting {
        graphics.draw_sprite(0, 128, delete_mode);
    } else {
        match game_state.team_mode {
            systems::Team::RED => graphics.draw_sprite(0, 128, red_edit_mode),
            systems::Team::BLUE => graphics.draw_sprite(0, 128, blue_edit_mode),
        }
    }

    for (entity, pos) in game_state.world.positions.iter() {
        let sprite = match game_state.world.teams.get(entity) {
            Some(systems::Team::RED) => red_troop,
            Some(systems::Team::BLUE) => blue_troop,
            None => continue,
        };
        graphics.draw_sprite(pos.x, pos.y, sprite);
    }
}
pub fn troop_at(world: &systems::World, x: u32, y: u32) -> Option<systems::Entity> {
    let red_troop = RED_TROOP
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/red-troop.png")));
    let blue_troop = BLUE_TROOP
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/blue-troop.png")));

    for (&entity, pos) in world.positions.iter() {
        let sprite = match world.teams.get(&entity) {
            Some(systems::Team::RED) => red_troop,
            Some(systems::Team::BLUE) => blue_troop,
            None => continue,
        };
        // origin is at top left corner
        if x >= pos.x && x < pos.x + sprite.width && y >= pos.y && y < pos.y + sprite.height {
            return Some(entity);
        }
    }
    None
}
pub fn buttons(game_state: &systems::GameState) -> [systems::Button; 3] {
    let change_mode = CHANGE_MODE
        .get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/change-mode.png")));
    let play_pause = if game_state.paused {
        PAUSED.get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/paused.png")))
    } else {
        PLAYING.get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/playing.png")))
    };
    let delete =
        DELETE.get_or_init(|| Sprite::from_bytes(include_bytes!("../assets/images/delete.png")));

    [
        systems::Button {
            x: 0,
            y: 0,
            width: change_mode.width,
            height: change_mode.height,
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
            width: delete.width,
            height: delete.height,
            on_click: systems::GameState::toggle_delete,
        },
    ]
}
