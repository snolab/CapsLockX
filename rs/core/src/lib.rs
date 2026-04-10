pub mod acc_model;
#[cfg(test)]
mod acc_model_test;
pub mod audio_capture;
pub mod engine;
pub mod key_code;
pub mod agent;
pub mod cloud_stt;
pub mod llm_client;
pub mod local_sherpa;
pub mod local_whisper;
pub mod stt_corrector;
pub mod task_manager;
pub mod tts;
pub mod modules;
pub mod platform;
pub mod state;

pub use engine::{ClxEngine, CoreResponse};
pub use key_code::{KeyCode, Modifiers};
pub use platform::Platform;
pub use state::{ClxConfig, ClxState, SpeedConfig};
