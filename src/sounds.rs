#[cfg(not(target_arch = "wasm32"))]
mod native {
    use kira::{
        AudioManager, AudioManagerSettings, DefaultBackend, sound::static_sound::StaticSoundData,
    };

    pub struct Sounds {
        manager: AudioManager,
    }

    impl Sounds {
        pub fn new() -> Self {
            Sounds {
                manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                    .expect("failed to create audio manager"),
            }
        }

        pub fn play(&mut self, bytes: &'static [u8]) {
            let sound = StaticSoundData::from_cursor(std::io::Cursor::new(bytes))
                .expect("invalid sound data");
            self.manager.play(sound).expect("failed to play sound");
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use js_sys::{Array, Uint8Array};
    use wasm_bindgen::JsValue;
    use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, Url};

    pub struct Sounds;

    impl Sounds {
        pub fn new() -> Self {
            Sounds
        }

        pub fn play(&mut self, bytes: &'static [u8]) {
            let array = Uint8Array::from(bytes);
            let parts = Array::new();
            parts.push(&array);
            let parts: JsValue = parts.into();

            let options = BlobPropertyBag::new();
            options.set_type("audio/ogg");

            let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &options)
                .expect("failed to build blob");
            let url = Url::create_object_url_with_blob(&blob).expect("failed to create url");
            let audio =
                HtmlAudioElement::new_with_src(&url).expect("failed to create audio element");
            let _ = audio.play();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::Sounds;
#[cfg(target_arch = "wasm32")]
pub use web::Sounds;
