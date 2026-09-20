use crate::{
    graphics::{BLUE_TROOP, RED_TROOP},
    sounds::Sounds,
};
use std::collections::HashMap;

pub type Entity = u32;

static BG_MUSIC: [&[u8]; 4] = [
    include_bytes!("../assets/sounds/music/song1.ogg"),
    include_bytes!("../assets/sounds/music/song2.ogg"),
    include_bytes!("../assets/sounds/music/song3.ogg"),
    include_bytes!("../assets/sounds/music/song4.ogg"),
];

pub struct BGMusicPlayer {
    sounds: Sounds,
    current: usize,
}

impl BGMusicPlayer {
    pub fn new() -> Self {
        let mut sounds = Sounds::new();
        sounds.play(BG_MUSIC[0]);
        BGMusicPlayer { sounds, current: 0 }
    }

    pub fn update(&mut self) {
        if self.sounds.finished() {
            self.current = (self.current + 1) % BG_MUSIC.len();
            self.sounds.play(BG_MUSIC[self.current]);
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Team {
    RED,
    BLUE,
}

pub struct Position {
    pub x: u32,
    pub y: u32,
}
pub struct Button {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub on_click: fn(&mut GameState),
}

impl Button {
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

pub struct GameState {
    pub deleting: bool,
    pub paused: bool,
    pub team_mode: Team,
    pub world: World,
}

impl GameState {
    pub fn change_teams(&mut self) {
        if self.deleting {
            self.deleting = false
        }
        self.team_mode = if self.team_mode == Team::RED {
            Team::BLUE
        } else {
            Team::RED
        }
    }
    pub fn toggle_delete(&mut self) {
        self.deleting = !self.deleting
    }
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused
    }
}

#[derive(Default)]
pub struct World {
    next_entity: Entity,
    pub positions: HashMap<Entity, Position>,
    pub teams: HashMap<Entity, Team>,
}

impl World {
    pub fn new_entity(&mut self) -> Entity {
        let id = self.next_entity;
        self.next_entity += 1;
        id
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.positions.remove(&entity);
        self.teams.remove(&entity);
    }
}

pub fn spawn_troop(world: &mut World, pos: Position, team: Team) -> Entity {
    let red_troop = RED_TROOP.get();
    let blue_troop = BLUE_TROOP.get();

    let sprite = match team {
        Team::RED => red_troop,
        Team::BLUE => blue_troop,
    };

    let entity = world.new_entity();
    world.positions.insert(
        entity,
        Position {
            x: pos.x - sprite.expect("couldn't unwrap sprite x").width / 2,
            y: pos.y - sprite.expect("couldn't unwrap sprite y").width / 2,
        },
    );
    world.teams.insert(entity, team);
    entity
}
