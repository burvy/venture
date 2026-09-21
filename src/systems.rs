use crate::{
    graphics::{BLUE_TROOP, RED_TROOP},
    sounds::Sounds,
};
use std::{collections::HashMap, f64::consts::FRAC_PI_2};

pub type Entity = u32;

/// pixels a troop can move per tick
const TROOP_SPEED: f64 = 2.0;
/// "range" to maintain from teammates (64^2)
const TROOP_TEAM_RANGE: f64 = 4096.0;
/// "range" to maintain from teammates (512^2)
const TROOP_ENEM_RANGE: f64 = 262144.0;
/// margin of error to so troops stay fixed on the
/// border of being too close or too far, preventing jittering
const RANGE_MARGIN: f64 = 4.0;

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
    pub rotations: HashMap<Entity, f64>,
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
    // my troop sprite faces upwards (for both troops)
    world.rotations.insert(entity, 0.0);
    entity
}

/// Maintains best distance between enemies and also between friends
/// TODO: Add a bit of randomness into their movement
pub fn update_troops(world: &mut World) {
    // you may use either `teams` or `positions`, they're the
    // same entities
    let entities: Vec<Entity> = world.teams.keys().copied().collect();

    for entity in entities {
        let Some(&own_team) = world.teams.get(&entity) else {
            continue;
        };
        let Some(pos) = world.positions.get(&entity) else {
            continue;
        };

        let (x, y) = (pos.x as f64, pos.y as f64);

        // dx, dy, distance
        let mut nearest_enem: Option<(f64, f64, f64)> = None;
        let mut nearest_ally: Option<(f64, f64, f64)> = None;

        // TODO: add a more efficient check that DOESN'T
        // grow by O(n^2) because we check every single
        // entity
        for (&other, other_pos) in world.positions.iter() {
            if other == entity {
                continue;
            }
            let dx = other_pos.x as f64 - x;
            let dy = other_pos.y as f64 - y;

            let dist_sq = dx * dx + dy * dy;

            // if the last best entity's "distance" was further than
            // the "distance" to this entity or there is none,
            // set the best entity to this one because this one
            // is a better target (it's closer)
            //
            // Note that the distance is not the true
            // distance, but it should be fine.
            //
            // TODO: tweak how entities are selected as the AI
            // gets more advanced
            let update_closest_fn = |pos: &mut Option<(f64, f64, f64)>| {
                if let Some(closest) = pos {
                    if closest.2 > dist_sq {
                        *pos = Some((dx, dy, dist_sq));
                    }
                } else {
                    *pos = Some((dx, dy, dist_sq));
                }
            };

            if world.teams.get(&other) != Some(&own_team) {
                update_closest_fn(&mut nearest_enem);
            } else {
                update_closest_fn(&mut nearest_ally);
            }
        }

        let mut move_x = 0.0;
        let mut move_y = 0.0;

        let determine_troop_speed = |axis: f64| {
            if axis > 0.0 {
                TROOP_SPEED
            } else if axis < 0.0 {
                -TROOP_SPEED
            } else {
                0.0
            }
        };
        if let Some((dx, dy, crude_dist)) = nearest_enem {
            let x_dir = determine_troop_speed(dx);
            let y_dir = determine_troop_speed(dy);
            if crude_dist > TROOP_ENEM_RANGE + RANGE_MARGIN {
                move_x += x_dir;
                move_y += y_dir;
            } else if crude_dist < TROOP_ENEM_RANGE - RANGE_MARGIN {
                move_x -= x_dir;
                move_y -= y_dir;
            }
        }

        if let Some((dx, dy, _)) = nearest_enem {
            // + pi / 2 because sprite originally faces up
            let facing = dy.atan2(dx) + FRAC_PI_2;
            world.rotations.insert(entity, facing);
        }

        if let Some((dx, dy, crude_dist)) = nearest_ally {
            if crude_dist < TROOP_TEAM_RANGE {
                // colliding with teammates is more important to resolve
                let away_x = -determine_troop_speed(dx) * 2.0;
                let away_y = -determine_troop_speed(dy) * 2.0;
                move_x += away_x;
                move_y += away_y;
            }
        }

        if move_x != 0.0 || move_y != 0.0 {
            let new_pos = Position {
                x: (x + move_x).floor() as u32,
                y: (y + move_y).floor() as u32,
            };
            world.positions.insert(entity, new_pos);
        }
    }
}
