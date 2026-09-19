use crate::sounds::Sounds;

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

pub enum Team {
    RED,
    BLUE,
}

pub struct GameState {
    pub edit_mode: Team,
}
