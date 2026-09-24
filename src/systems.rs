use crate::graphics;
use crate::sounds::Sounds;
use std::{
    collections::{HashMap, HashSet},
    f64::consts::{FRAC_PI_2, FRAC_PI_6, PI, TAU},
};

pub type Entity = u32;

/// pixels a troop can move per tick
const TROOP_SPEED: f64 = 2.0;
/// speed multiplier of a troop trying to not merge with their teammates
const TROOP_SPREAD_SPEED_MULTIPLIER: f64 = 3.0;
/// how much a troop's velocity can change per tick
const TROOP_ACCELERATION: f64 = 0.3;
/// how many radians a troop can turn per tick
const TROOP_TURN_RATE: f64 = 0.1;
/// pixels to maintain from teammates
const TROOP_CROWDING_RANGE: f64 = 64.0;
/// pixels to maintain from enemies
const TROOP_ENEM_RANGE: f64 = 512.0;
/// pixels of margin around a range so troops don't jitter
/// right at the border of being too close or too far
const RANGE_MARGIN: f64 = 4.0;
/// random wobble per tick to make things interesting
const TROOP_WANDER: f64 = 0.5;

/// pixels the camera moves while movement is held
const CAMERA_SPEED: i32 = 8;

/// radius of the circle for painting/erasing
pub const BRUSH_RADIUS: i32 = 32;

/// pixels to look ahead to avoid things
const WHISKER_LENGTH: f64 = 256.0;
/// two side pixels
const WHISKER_ANGLE: f64 = FRAC_PI_6;
/// how many steps to take within length
const WHISKER_STEPS: f64 = 64.0;

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

impl Default for BGMusicPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Team {
    RED,
    BLUE,
}

#[derive(PartialEq)]
pub enum Mode {
    DEPLOY,
    PAINT,
    ERASE,
}

pub struct Position {
    pub x: i32,
    pub y: i32,
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

pub enum PanDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub struct DPadButton {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub direction: PanDirection,
}

impl DPadButton {
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

pub struct GameState {
    pub mode: Mode,
    pub paused: bool,
    pub team_mode: Team,
    pub world: World,
    pub camera: Position,
}

impl GameState {
    pub fn change_teams(&mut self) {
        self.toggle_deploy();
        self.team_mode = if self.team_mode == Team::RED {
            Team::BLUE
        } else {
            Team::RED
        }
    }
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused
    }
    pub fn toggle_deploy(&mut self) {
        self.mode = Mode::DEPLOY
    }
    pub fn toggle_paint(&mut self) {
        self.mode = Mode::PAINT
    }
    pub fn toggle_erase(&mut self) {
        self.mode = Mode::ERASE
    }
}

#[derive(Default)]
pub struct World {
    next_entity: Entity,
    pub positions: HashMap<Entity, Position>,
    pub teams: HashMap<Entity, Team>,
    pub rotations: HashMap<Entity, f64>,
    pub velocities: HashMap<Entity, (f64, f64)>,
    pub obstacles: Obstacles,
}

impl World {
    pub fn new_entity(&mut self) -> Entity {
        let id = self.next_entity;
        self.next_entity += 1;
        id
    }

    /// despawns the entity and removes
    /// all those extra copies of the entity's
    /// info used for acceleration
    pub fn despawn(&mut self, entity: Entity) {
        self.positions.remove(&entity);
        self.teams.remove(&entity);
        self.rotations.remove(&entity);
        self.velocities.remove(&entity);
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
            x: pos.x - sprite.width as i32 / 2,
            y: pos.y - sprite.height as i32 / 2,
        },
    );
    world.teams.insert(entity, team);
    // my troops face upwards
    world.rotations.insert(entity, 0.0);

    world.velocities.insert(entity, (0.0, 0.0));
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
    velocity: (f64, f64),
}
#[derive(PartialEq)]
enum OnMyTeam {
    Yes,
    No,
    DoesNotMatter,
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
        // filters for ally, not ally, or doesnt matter
        .filter(|&(&other, _)| {
            if other == entity {
                return false;
            }
            let is_ally = world.teams.get(&other) == Some(&own_team);
            match same_team {
                OnMyTeam::Yes => is_ally,
                OnMyTeam::No => !is_ally,
                OnMyTeam::DoesNotMatter => true,
            }
        })
        // extracts positions only
        .map(|(_, other_pos)| {
            let dx = other_pos.x as f64 - x;
            let dy = other_pos.y as f64 - y;
            (dx, dy, dx * dx + dy * dy)
        })
        // sorts for the smallest
        // `min_by` works like current champion vs next challenger
        // the smallest one wins. it's also secretly a `fold`
        .min_by(|a, b| a.2.partial_cmp(&b.2).expect("NaN value encountered"))
}

fn accelerate_towards(current: f64, target: f64, accel: f64) -> f64 {
    if current < target {
        (current + accel).min(target)
    } else if current > target {
        (current - accel).max(target)
    } else {
        current
    }
}

fn turn_towards(current: f64, target: f64, max_turn: f64) -> f64 {
    let mut diff = (target - current) % TAU;
    if diff > PI {
        diff -= TAU;
    } else if diff < -PI {
        diff += TAU;
    }
    current + diff.clamp(-max_turn, max_turn)
}

/// checks if a point is blocked by world obstacles
fn pos_blocked(world: &World, x: i32, y: i32) -> bool {
    // specific function to check if a point is blocked
    world.obstacles.is_blocked(x, y)
}

/// checks if a ray is blocked by world obstacles
fn ray_blocked(world: &World, x: f64, y: f64, angle: f64, length: f64, steps: f64) -> bool {
    let dir_x = angle.cos();
    let dir_y = angle.sin();
    let step_size = length / steps;
    (1..=steps.floor() as i32).any(|i| {
        let dist = step_size * i as f64;
        let pix_i = (x + dir_x * dist).round() as i32;
        let pix_j = (y + dir_y * dist).round() as i32;
        world.obstacles.is_blocked(pix_i, pix_j)
    })
}

fn avoidance_force(world: &World, entity: Entity, mot_x: f64, mot_y: f64) -> (f64, f64) {
    // not moving
    if mot_x == 0.0 && mot_y == 0.0 {
        return (0.0, 0.0);
    }

    let Some((center_x, center_y)) = graphics::center_of_troop(world, entity) else {
        // cant find the center
        return (0.0, 0.0);
    };

    let (center_x, center_y) = (center_x as f64, center_y as f64);

    // infers angle from motion
    let angle = mot_y.atan2(mot_x);

    let left_blocked = ray_blocked(
        world,
        center_x,
        center_y,
        angle - WHISKER_ANGLE,
        WHISKER_LENGTH,
        WHISKER_STEPS,
    );
    let right_blocked = ray_blocked(
        world,
        center_x,
        center_y,
        angle + WHISKER_ANGLE,
        WHISKER_LENGTH,
        WHISKER_STEPS,
    );

    // deconstructs components from angle
    let (dir_x, dir_y) = (angle.cos(), angle.sin());

    // decision table
    let (steer_x, steer_y) = match (left_blocked, right_blocked) {
        (false, false) => return (0.0, 0.0), // clear
        (false, true) => (-dir_y, dir_x),    // steer left
        (true, false) => (dir_y, -dir_x),    // steer right
        (true, true) => (dir_y, -dir_x),     // steer right
    };

    // suggested action
    (steer_x * TROOP_SPEED, steer_y * TROOP_SPEED)
}

/// Push myself away from nearest troops
fn separation_force(world: &World, entity: Entity, own_team: Team, x: f64, y: f64) -> (f64, f64) {
    // the one nearest other entity
    let other = nearest(world, entity, own_team, x, y, OnMyTeam::DoesNotMatter);
    // if that entity is in range
    let in_range = other.filter(|&(_, _, dist_sq)| dist_sq < TROOP_CROWDING_RANGE.powi(2));

    match in_range {
        Some((dx, dy, _)) => (
            -get_speed_for(dx) * TROOP_SPREAD_SPEED_MULTIPLIER,
            -get_speed_for(dy) * TROOP_SPREAD_SPEED_MULTIPLIER,
        ),
        None => (0.0, 0.0),
    }
}

/// Direction to move and rotate into to get closer to enemies
fn pathfind_force(
    world: &World,
    entity: Entity,
    own_team: Team,
    x: f64,
    y: f64,
    current_rotation: f64,
) -> (f64, f64, Option<f64>) {
    let Some((dx, dy, dist_sq)) = nearest(world, entity, own_team, x, y, OnMyTeam::No) else {
        return (0.0, 0.0, None);
    };

    let target_facing = dy.atan2(dx) + FRAC_PI_2;
    let rotation = Some(turn_towards(
        current_rotation,
        target_facing,
        TROOP_TURN_RATE,
    ));

    let x_dir = get_speed_for(dx);
    let y_dir = get_speed_for(dy);
    let (move_x, move_y) = if dist_sq > (TROOP_ENEM_RANGE + RANGE_MARGIN).powi(2) {
        (x_dir, y_dir)
    } else if dist_sq < (TROOP_ENEM_RANGE - RANGE_MARGIN).powi(2) {
        (-x_dir, -y_dir)
    } else {
        (0.0, 0.0)
    };

    (move_x, move_y, rotation)
}

fn randomness(randomness: f64) -> f64 {
    rand::random::<f64>() * (randomness * 2.0) - randomness
}

/// Give the next step for one singular troop
/// Maintains best distance between enemies and also between friends
fn troop_update(world: &World, troop: Entity) -> Option<TroopUpdate> {
    // own information
    let &own_team = world.teams.get(&troop)?;
    let pos = world.positions.get(&troop)?;
    let (x, y) = (pos.x as f64, pos.y as f64);
    let (vel_x, vel_y) = world.velocities.get(&troop).copied().unwrap_or((0.0, 0.0));
    let my_rot = world.rotations.get(&troop).copied().unwrap_or(0.0);

    // forces
    let (sep_x, sep_y) = separation_force(world, troop, own_team, x, y);
    let (seek_x, seek_y, rotation) = pathfind_force(world, troop, own_team, x, y, my_rot);

    // sums up primitive motivating force components into a desired direction
    let primitive_x = [sep_x, seek_x].iter().sum();
    let primitive_y = [sep_y, seek_y].iter().sum();

    // advanced forces
    let (avoid_x, avoid_y) = avoidance_force(world, troop, primitive_x, primitive_y);

    // sums up advanced motivating force components into a desired direction
    let desire_x = [primitive_x, avoid_x, randomness(TROOP_WANDER)]
        .iter()
        .sum();
    let desire_y = [primitive_y, avoid_y, randomness(TROOP_WANDER)]
        .iter()
        .sum();

    // accelerate troop towards net desire vector
    let new_vel_x = accelerate_towards(vel_x, desire_x, TROOP_ACCELERATION);
    let new_vel_y = accelerate_towards(vel_y, desire_y, TROOP_ACCELERATION);

    // precalculates the new x and y position
    // using `.round()` fixes random wobble getting too big of an effect
    let new_x = (x + new_vel_x).round() as i32;
    let new_y = (y + new_vel_y).round() as i32;

    let sprite = graphics::troop_sprite(world, troop)?;

    if new_vel_x == 0.0 && new_vel_y == 0.0
        || pos_blocked(
            world,
            new_x + sprite.width as i32 / 2,
            new_y + sprite.height as i32 / 2,
        )
    {
        // don't update pos and vel with bad conditions
        Some(TroopUpdate {
            entity: troop,
            position: Position {
                x: x as i32,
                y: y as i32,
            },
            rotation,
            velocity: (0.0, 0.0),
        })
    } else {
        // update all
        Some(TroopUpdate {
            entity: troop,
            position: Position { x: new_x, y: new_y },
            rotation,
            velocity: (new_vel_x, new_vel_y),
        })
    }
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
        world.velocities.insert(update.entity, update.velocity);
        if let Some(rotation) = update.rotation {
            world.rotations.insert(update.entity, rotation);
        }
    }
}

#[derive(Default)]
pub struct Obstacles {
    pixels: HashSet<(i32, i32)>,
}

/// returns every point in the circle with radius `radius` around `x`, `y`
fn circle_pixels(x: i32, y: i32, radius: i32) -> impl Iterator<Item = (i32, i32)> {
    // arbritrary square with sides radius * radius
    (-radius..=radius).flat_map(move |dx| {
        (-radius..=radius)
            // filter out non-circle pixels in our square
            .filter(move |&dy| dx.pow(2) + dy.pow(2) <= radius.pow(2))
            // moves the circle to our coordinates x and y
            .map(move |dy| (x + dx, y + dy))
    })
}

/// draws a line from (x1, y1) to (x2, y2) for the circles so it looks smoother
pub fn lerp_points(x1: i32, y1: i32, x2: i32, y2: i32) -> impl Iterator<Item = (i32, i32)> {
    let dx = (x2 - x1) as f64;
    let dy = (y2 - y1) as f64;
    let distance = (dx * dx + dy * dy).sqrt();
    let step = BRUSH_RADIUS as f64 / 2.0;
    let steps = (distance / step).ceil().max(1.0) as i32;

    (0..=steps).map(move |i| {
        let t = i as f64 / steps as f64;
        (
            (x1 as f64 + dx * t).round() as i32,
            (y1 as f64 + dy * t).round() as i32,
        )
    })
}

impl Obstacles {
    /// adds the pixel for every pixel in the circle
    pub fn paint(&mut self, x: i32, y: i32, radius: i32) {
        for p in circle_pixels(x, y, radius) {
            self.pixels.insert(p);
        }
    }

    /// removes the pixel for every pixel in the circle
    pub fn erase(&mut self, x: i32, y: i32, radius: i32) {
        for p in circle_pixels(x, y, radius) {
            self.pixels.remove(&p);
        }
    }

    /// the pixel at x and y is within our set of pixels
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.pixels.contains(&(x, y))
    }

    /// returns our set of pixels
    pub fn painted_pixels(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        self.pixels.iter().copied()
    }
}

fn troop_in_brush(
    troop_pos: &Position,
    sprite: &graphics::Sprite,
    brush_x: i32,
    brush_y: i32,
    brush_radius: i32,
) -> bool {
    let troop_radius = sprite.width.max(sprite.height) as i32 / 2;
    let center_x = troop_pos.x + sprite.width as i32 / 2;
    let center_y = troop_pos.y + sprite.height as i32 / 2;
    let dx = center_x - brush_x;
    let dy = center_y - brush_y;
    let max_dist = brush_radius + troop_radius;
    dx * dx + dy * dy <= max_dist * max_dist
}

pub fn erase_at(world: &mut World, x: i32, y: i32, radius: i32) {
    world.obstacles.erase(x, y, radius);
    let doomed: Vec<Entity> = world
        .positions
        .iter()
        // formats all valid troops
        .filter_map(|(&e, pos)| graphics::troop_sprite(world, e).map(|sprite| (e, pos, sprite)))
        // filters troops that are doomed
        .filter(|(_, pos, sprite)| troop_in_brush(pos, sprite, x, y, radius))
        // gets their ids
        .map(|(e, _, _)| e)
        .collect();
    // and despawns them based on id
    for entity in doomed {
        world.despawn(entity);
    }
}

pub fn paint_at(world: &mut World, x: i32, y: i32, radius: i32) {
    world.obstacles.paint(x, y, radius);
}

pub fn pan_camera(camera: &mut Position, up: bool, down: bool, left: bool, right: bool) {
    if up {
        camera.y = camera.y.saturating_sub(CAMERA_SPEED);
    }
    if down {
        camera.y = camera.y.saturating_add(CAMERA_SPEED);
    }
    if left {
        camera.x = camera.x.saturating_sub(CAMERA_SPEED);
    }
    if right {
        camera.x = camera.x.saturating_add(CAMERA_SPEED);
    }
}
