use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::tts::voice::Voice;
use crate::util::sleep::sleep;

pub struct TextToSpeech;

impl TextToSpeech {
    pub async fn get_speakers() -> Vec<Voice> {
        #[cfg(feature = "web")]
        {
            use web_sys::{SpeechSynthesisUtterance, SpeechSynthesisVoice};
            let mut type_get_speaker_time = 0;
            for _ in 0..100 {
                type_get_speaker_time += 1;
                if let Some(window) = web_sys::window() {
                    if let Ok(ss) = window.speech_synthesis() {
                        let mut voice = ss
                            .get_voices()
                            .into_iter()
                            .map(|it| SpeechSynthesisVoice::from(it))
                            .map(|it| Voice(it))
                            .collect::<Vec<_>>();

                        if voice.len() > 0 {
                            let mut filtered = voice
                                .iter()
                                .filter(|v| {
                                    let n = v.0.name().trim().to_ascii_lowercase();
                                    n.contains("chinese")
                                        || n.contains("china")
                                        || n.contains("taiwan")
                                        || n.contains("zh")
                                        || n.contains("cn")
                                        || n.contains("中文")
                                        || n.contains("普通话")
                                        || n.contains("国语")
                                        || n.contains("汉语")
                                        || n.contains("男")
                                        || n.contains("女")
                                })
                                .cloned()
                                .collect::<Vec<_>>();
                            if filtered.len() > 0 {
                                voice = filtered;
                            }

                            if type_get_speaker_time > 1 {
                                debug!(
                                    "type_get_speaker_failed_time:{}",
                                    type_get_speaker_time - 1
                                );
                            }

                            return voice;
                        }
                    }
                }
                sleep(50).await;
            }
            debug!("type_get_speaker_time:{type_get_speaker_time}");
        }

        Vec::new()
    }
    pub async fn speak_with_speaker(
        text: String,
        speaker: Option<&Voice>,
        volume: f32,
    ) -> Option<()> {
        #[cfg(feature = "web")]
        {
            use js_sys::wasm_bindgen::JsCast;
            use js_sys::wasm_bindgen::prelude::Closure;
            use web_sys::{SpeechSynthesisUtterance, SpeechSynthesisVoice};
            debug!("try to speak,volume:{volume}");

            let window = web_sys::window()?;
            let ss = window.speech_synthesis().ok()?;
            let ssu = SpeechSynthesisUtterance::new_with_text(text.as_str()).ok()?;
            ssu.set_voice(speaker.map(|s| &s.0));
            // ssu.set_lang("zh");
            ssu.set_volume(volume);

            let playing = Rc::new(AtomicBool::new(true));
            let playing_in_closure = playing.to_owned();
            let closure: Closure<dyn Fn()> =
                Closure::new(move || playing_in_closure.store(false, Ordering::Relaxed));
            ssu.set_onend(Some(closure.as_ref().unchecked_ref()));
            ss.speak(&ssu);

            while playing.load(Ordering::Relaxed) {
                sleep(10).await;
            }

            drop(closure);
        }

        Some(())
    }
}
