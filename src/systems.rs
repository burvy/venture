use crate::graphics;
use crate::sounds::Sounds;
use std::{collections::HashMap, f64::consts::FRAC_PI_2};

pub type Entity = u32;

/// pixels a troop can move per tick
const TROOP_SPEED: f64 = 2.0;
/// speed multiplier of a troop trying to not merge with their teammates
const TROOP_SPREAD_SPEED_MULTIPLIER: f64 = 3.0;
/// pixels to maintain from teammates
const TROOP_TEAM_RANGE: f64 = 64.0;
/// pixels to maintain from enemies
const TROOP_ENEM_RANGE: f64 = 512.0;
/// pixels of margin around a range so troops don't jitter
/// right at the border of being too close or too far
const RANGE_MARGIN: f64 = 4.0;
/// random wobble per tick to make things interesting
const TROOP_WANDER: f64 = 0.5;

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
    let sprites = graphics::sprites();
    let sprite = match team {
        Team::RED => &sprites.red_troop,
        Team::BLUE => &sprites.blue_troop,
    };

    let entity = world.new_entity();
    world.positions.insert(
        entity,
        Position {
            // reminder that saturating sub doesn't sub
            // past the data type's limits
            x: pos.x.saturating_sub(sprite.width / 2),
            y: pos.y.saturating_sub(sprite.height / 2),
        },
    );
    world.teams.insert(entity, team);
    // my troops face upwards
    world.rotations.insert(entity, 0.0);
    entity
}

fn get_speed_for(axis: f64) -> f64 {
    if axis > 0.0 {
        TROOP_SPEED
    } else if axis < 0.0 {
        -TROOP_SPEED
    } else {
        0.0
    }
}

struct TroopUpdate {
    entity: Entity,
    position: Position,
    rotation: Option<f64>,
}

#[derive(PartialEq)]
enum OnMyTeam {
    Yes,
    No,
}
/// Returns the position and distance of the nearest entity
/// its Option<(dx, dy, distance squared)>
///
/// TODO: add a more efficient check that DOESN'T
/// grow by O(n^2) because we check every single
/// entity
fn nearest(
    world: &World,
    entity: Entity,
    own_team: Team,
    x: f64,
    y: f64,
    same_team: OnMyTeam,
) -> Option<(f64, f64, f64)> {
    world
        .positions
        .iter()
        .filter(|&(&other, _)| {
            other != entity
                && (world.teams.get(&other) == Some(&own_team)) == (same_team == OnMyTeam::Yes)
        })
        .map(|(_, other_pos)| {
            let dx = other_pos.x as f64 - x;
            let dy = other_pos.y as f64 - y;
            (dx, dy, dx * dx + dy * dy)
        })
        // `min_by` works like current champion vs next challenger
        // the smallest one wins. it's also secretly a `fold`
        .min_by(|a, b| a.2.partial_cmp(&b.2).expect("NaN value encountered"))
}

/// Give the next step for one singular troop
/// Maintains best distance between enemies and also between friends
/// TODO: Add a bit of randomness into their movement
fn troop_update(world: &World, entity: Entity) -> Option<TroopUpdate> {
    let &own_team = world.teams.get(&entity)?;
    let pos = world.positions.get(&entity)?;
    let (x, y) = (pos.x as f64, pos.y as f64);

    let nearest_enem = nearest(world, entity, own_team, x, y, OnMyTeam::No);
    let nearest_ally = nearest(world, entity, own_team, x, y, OnMyTeam::Yes);

    let mut move_x = 0.0;
    let mut move_y = 0.0;
    let mut rotation = None;

    if let Some((dx, dy, _)) = nearest_enem {
        // + pi / 2 because sprite originally faces up
        rotation = Some(dy.atan2(dx) + FRAC_PI_2);
    }

    // ally that is too close to me
    let crowded_by_ally =
        nearest_ally.filter(|&(_, _, dist_sq)| dist_sq < TROOP_TEAM_RANGE.powi(2));

    if let Some((dx, dy, _)) = crowded_by_ally {
        // move away from crowding allies first!
        move_x -= get_speed_for(dx) * TROOP_SPREAD_SPEED_MULTIPLIER;
        move_y -= get_speed_for(dy) * TROOP_SPREAD_SPEED_MULTIPLIER;
    } else if let Some((dx, dy, dist_sq)) = nearest_enem {
        // maintain range with enemies afterwards
        let x_dir = get_speed_for(dx);
        let y_dir = get_speed_for(dy);
        if dist_sq > (TROOP_ENEM_RANGE + RANGE_MARGIN).powi(2) {
            move_x += x_dir;
            move_y += y_dir;
        } else if dist_sq < (TROOP_ENEM_RANGE - RANGE_MARGIN).powi(2) {
            move_x -= x_dir;
            move_y -= y_dir;
        }
    }

    if move_x != 0.0 || move_y != 0.0 {
        // random is [0, 1), we want [-TROOP_WANDER, TROOP_WANDER)
        move_x += rand::random::<f64>() * (TROOP_WANDER * 2.0) - TROOP_WANDER;
        move_y += rand::random::<f64>() * (TROOP_WANDER * 2.0) - TROOP_WANDER;
    }
    if move_x == 0.0 && move_y == 0.0 && rotation.is_none() {
        return None;
    }

    Some(TroopUpdate {
        entity,
        position: Position {
            // .floor() is faster than .round()
            x: (x + move_x).floor() as u32,
            y: (y + move_y).floor() as u32,
        },
        rotation,
    })
}

/// aggregates all the troop updates and runs them all at once
/// like the game of life, but 2d! Check out my other repo: life-v2
pub fn update_troops(world: &mut World) {
    // takes the world and all entities
    // and updates them one step
    //
    // note that we can look at the keys of
    // world.teams or world.positions, or
    // world.rotations. They're the same
    // entity ids. The values associated
    // with the keys are different though.
    let updates: Vec<TroopUpdate> = world
        .positions
        .keys()
        .copied()
        .filter_map(|entity| troop_update(world, entity))
        .collect();

    for update in updates {
        world.positions.insert(update.entity, update.position);
        if let Some(rotation) = update.rotation {
            world.rotations.insert(update.entity, rotation);
        }
    }
}
