#[cfg(test)]
mod engine_test;

pub mod acc_model;
pub mod engine;
pub mod key_code;
pub mod modules;
pub mod platform;
pub mod state;

#[cfg(not(target_arch = "wasm32"))]
pub mod audio_capture;
#[cfg(not(target_arch = "wasm32"))]
pub mod agent;
#[cfg(not(target_arch = "wasm32"))]
pub mod cloud_stt;
#[cfg(not(target_arch = "wasm32"))]
pub mod llm_client;
#[cfg(not(target_arch = "wasm32"))]
pub mod local_sherpa;
#[cfg(not(target_arch = "wasm32"))]
pub mod local_whisper;
#[cfg(not(target_arch = "wasm32"))]
pub mod stt_corrector;
#[cfg(not(target_arch = "wasm32"))]
pub mod task_manager;
#[cfg(not(target_arch = "wasm32"))]
pub mod tts;

pub use engine::{ClxEngine, CoreResponse};
pub use key_code::{KeyCode, Modifiers};
pub use platform::Platform;
pub use state::{ClxConfig, ClxState, SpeedConfig};
