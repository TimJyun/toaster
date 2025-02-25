#[cfg(feature = "web")]
use web_sys::SpeechSynthesisVoice;

//SpeechSynthesisVoice
#[derive(Clone)]
pub struct Voice(
    // JsValue
    #[cfg(feature = "web")] pub(super) SpeechSynthesisVoice,
);

impl Voice {
    pub fn name(&self) -> String {
        #[cfg(feature = "web")]
        {
            return self.0.name();
        }
        todo!()
    }
}
