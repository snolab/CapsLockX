pub mod acc_model;
#[cfg(test)]
mod acc_model_test;
pub mod agent;
pub mod audio_capture;
pub mod cloud_stt;
pub mod engine;
pub mod key_code;
pub mod llm_client;
#[cfg(feature = "stt")]
pub mod local_sherpa;
#[cfg(feature = "stt")]
pub mod local_whisper;
pub mod modules;
pub mod platform;
pub mod state;
pub mod stt_corrector;
pub mod task_manager;
pub mod tts;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_platform;

pub use engine::{ClxEngine, CoreResponse};
pub use key_code::{KeyCode, Modifiers};
pub use platform::Platform;
pub use state::{ClxConfig, ClxState, SpeedConfig};
