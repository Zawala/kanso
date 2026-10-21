//! A real dictation from the default microphone. Needs the model and a
//! microphone this process may use:
//!
//! ```sh
//! KANSO_VOICE_MODEL_DIR=/path/to/model cargo test -p kanso-voice --test dictation -- --ignored --nocapture
//! ```

use std::path::PathBuf;
use std::time::{Duration, Instant};

use kanso_voice::{Dictation, Permission, VoiceError, permission};

#[test]
#[ignore = "needs a microphone and the downloaded model in KANSO_VOICE_MODEL_DIR"]
fn a_short_dictation_records_and_finishes() {
    let dir = PathBuf::from(std::env::var("KANSO_VOICE_MODEL_DIR").unwrap());
    println!("microphone permission: {:?}", permission::status());
    let dictation = match Dictation::start(&dir) {
        Ok(dictation) => dictation,
        Err(e @ (VoiceError::NoMicrophone | VoiceError::MicrophoneDenied)) => {
            println!("skipped: {e}");
            return;
        }
        Err(e) => panic!("{e}"),
    };
    let mut loudest = 0.0f32;
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(3) {
        assert!(dictation.is_recording());
        loudest = loudest.max(dictation.level());
        std::thread::sleep(Duration::from_millis(50));
    }
    let elapsed = dictation.elapsed();
    println!("recorded {elapsed:?}, loudest level {loudest:.2}");
    assert!(
        elapsed >= Duration::from_millis(2_500) && elapsed <= Duration::from_millis(3_500),
        "{elapsed:?}"
    );
    assert!((0.0..=1.0).contains(&loudest));

    let stopping = Instant::now();
    let result = dictation.stop().recv_blocking().unwrap();
    println!("result after {:?}: {result:?}", stopping.elapsed());
    assert!(
        matches!(result, Ok(_) | Err(VoiceError::NothingHeard)),
        "{result:?}"
    );
}

#[test]
#[ignore = "needs a microphone"]
fn a_cancelled_dictation_lets_go_of_the_microphone() {
    if permission::status() == Permission::Denied {
        return;
    }
    let Ok(dictation) = Dictation::start(std::path::Path::new("/nonexistent/model")) else {
        return;
    };
    dictation.cancel();
    // A second one can open the device straight away.
    let Ok(again) = Dictation::start(std::path::Path::new("/nonexistent/model")) else {
        panic!("the microphone stayed busy");
    };
    let result = again.stop().recv_blocking().unwrap();
    assert!(matches!(result, Err(VoiceError::Model(_))), "{result:?}");
}
