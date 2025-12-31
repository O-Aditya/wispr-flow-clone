mod audio;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Initialize the state with the Mutex
        .manage(audio::AudioState { 
            stream: std::sync::Mutex::new(None),
            
        })

        .invoke_handler(tauri::generate_handler![
            audio::start_recording,
            audio::stop_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


