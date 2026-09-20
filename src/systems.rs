use crate::sounds::Sounds;
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

pub struct GameState {
    pub deleting: bool,
    pub paused: bool,
    pub team_mode: Team,
    pub world: World,
}

impl GameState {
    pub fn change_teams(&mut self) {
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
}

pub fn spawn_troop(world: &mut World, pos: Position, team: Team) -> Entity {
    let entity = world.new_entity();
    world.positions.insert(entity, pos);
    world.teams.insert(entity, team);
    entity
}
