use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::tts::tts::TextToSpeech;
use crate::util::sleep::sleep;
use tracing::debug;

pub const PLAYING_SOUND: AtomicBool = AtomicBool::new(false);

pub async fn play_sound_u8v(sound_u8vec: Vec<u8>) -> Option<()> {
    #[cfg(feature = "web")]
    {
        use js_sys::Uint8Array;
        use js_sys::wasm_bindgen::JsCast;
        use js_sys::wasm_bindgen::prelude::Closure;
        use web_sys::AudioBuffer;
        let sound_u8a = Uint8Array::from(sound_u8vec.as_slice());
        let audio_context = web_sys::AudioContext::new().ok()?;
        let v = audio_context.decode_audio_data(&sound_u8a.buffer()).ok()?;
        let b = wasm_bindgen_futures::JsFuture::from(v).await.ok()?;
        let source = audio_context.create_buffer_source().ok()?;
        source.set_buffer(Some(&AudioBuffer::from(b)));
        source
            .connect_with_audio_node(&audio_context.destination())
            .ok()?;

        let onended = Arc::new(AtomicBool::new(false));
        let onended_in_closure = onended.to_owned();
        let closure: Closure<dyn Fn()> = Closure::new(move || {
            let start = onended_in_closure.load(Ordering::Relaxed);
            debug!("onended_in_closure_start: {start}");
            let _ = (&onended_in_closure).compare_exchange(
                false,
                true,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
            let end = onended_in_closure.load(Ordering::Relaxed);
            debug!("onended_in_closure_end: {end}");
        });
        source.set_onended(Some(closure.as_ref().unchecked_ref()));

        return if let Ok(_) = source.start() {
            while !onended.load(Ordering::Relaxed) {
                sleep(10).await;
            }
            Some(())
        } else {
            None
        };
    }

    None
}
