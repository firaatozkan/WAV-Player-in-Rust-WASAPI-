use ctrlc;

mod wav_file;
mod wav_player;

use std::sync::Arc;

use wav_file::WavFile;
use wav_player::WavPlayer;

fn main() -> Result<(), ()> {
    let arguments: Vec<String> = std::env::args().collect();

    if arguments.len() < 2 {
        println!(
            r"This program works with file path of the requested wav file and an optional flag of playback. Usage: .\wav_player.exe your_wav_file.wav <-pb>"
        );
        return Err(());
    }

    let mut player = WavPlayer::new();
    let mut input_file = WavFile::new(arguments[1].as_str());

    let mut signal_flag = Arc::new(std::sync::atomic::AtomicBool::new(true));

    let signal_flag_clone = Arc::clone(&mut signal_flag);

    ctrlc::set_handler(move || {
        println!("Received SIGINT. Closing the player safely!");
        signal_flag_clone.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    if arguments.len() == 3 {
        if arguments[2] == "-pb" {
            player.play_continously(&mut input_file, signal_flag);
        } else {
            println!(
                r"This program works with file path of the requested wav file and an optional flag of playback. Usage: .\wav_player.exe your_wav_file.wav <-pb>"
            );
            return Err(());
        }
    } else {
        player.play_file(&mut input_file, signal_flag);
    }

    return Ok(());
}
