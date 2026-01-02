use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use futures_util::{SinkExt, StreamExt};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use http::Request;
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use dotenv_codegen::dotenv;



pub struct DeepgramStream(pub cpal::Stream);
unsafe impl Send for DeepgramStream {}
unsafe impl Sync for DeepgramStream {}

pub struct AudioState {
    pub stream: Mutex<Option<DeepgramStream>>,
}

#[tauri::command]

pub async fn start_recording(
    app: AppHandle, 
    state: State<'_, AudioState>
) -> Result<String, String> {
    
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);

    // 1. Setup Microphone
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or("No input device")?;
    

    let supported_configs = device.supported_input_configs().map_err(|e| e.to_string())?;
    let  config_range = supported_configs
        .filter(|c| c.channels() == 1) 
        .next();
    
    // Fallback: If no mono config found, pick the default
    let default_config = device.default_input_config().map_err(|e| e.to_string())?;
    if config_range.is_none() {
        println!("⚠️ No hardware mono found. Using default (might be stereo).");
    }
    
    // Use the detected sample rate (e.g., 48000)
    let sample_rate = config_range.as_ref().map(|c| c.max_sample_rate().0).unwrap_or(default_config.sample_rate().0);
    let sample_format = config_range.as_ref().map(|c| c.sample_format()).unwrap_or(default_config.sample_format());
    
    // Construct the actual StreamConfig
    let config = cpal::StreamConfig {
        channels: 1, 
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    println!("🎤 Mic Selected: {} Hz, {:?}, Channels: 1", sample_rate, sample_format);

    let api_key = dotenv!("DEEPGRAM_API_KEY").to_string();

    // 2. Start Network Thread
    tokio::spawn(async move {
       
        let url = format!(
            "wss://api.deepgram.com/v1/listen?encoding=linear16&sample_rate={}&channels=1&smart_format=true&model=nova-3&language=en-IN&interim_results=true",
            sample_rate
        );

        let request = Request::builder()
            .method("GET")
            .uri(url)
            .header("Authorization", format!("Token {}", api_key))
            .header("Sec-WebSocket-Key", generate_key())
            .header("Host", "api.deepgram.com")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .body(())
            .unwrap();

        match connect_async(request).await {
            Ok((ws_stream, _)) => {
                println!("✅ Connected to Deepgram!");
                let (mut write, mut read) = ws_stream.split();

                let read_handle = tokio::spawn(async move {
                    while let Some(msg) = read.next().await {
                        if let Ok(Message::Text(text)) = msg {
                            // Send EVERYTHING to frontend
                            let _ = app.emit("transcription", text);
                        }
                    }
                });

                while let Some(data) = rx.recv().await {
                    if let Err(_) = write.send(Message::Binary(data)).await { break; }
                }
                read_handle.abort();
            }
            Err(e) => eprintln!("❌ Failed to connect: {}", e),
        }
    });

    // 3. Audio Stream
    let err_fn = move |err| eprintln!("Stream error: {}", err);
    let tx_clone = tx.clone(); 

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut bytes = Vec::with_capacity(data.len() * 2);
                for &sample in data {
                    // Gain boost (optional): * 1.5 to make mic louder
                    let sample_i16 = (sample * 32767.0) as i16;
                    bytes.extend_from_slice(&sample_i16.to_le_bytes());
                }
                let _ = tx_clone.blocking_send(bytes);
            },
            err_fn,
            None
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                let mut bytes = Vec::with_capacity(data.len() * 2);
                for &sample in data {
                    bytes.extend_from_slice(&sample.to_le_bytes());
                }
                let _ = tx_clone.blocking_send(bytes);
            },
            err_fn,
            None
        ),
        _ => return Err("Unsupported format".to_string()),
    }.map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    *state.stream.lock().unwrap() = Some(DeepgramStream(stream));

    Ok("Recording Started".to_string())
}

#[tauri::command]
pub fn stop_recording(state: State<AudioState>) -> Result<String, String> {
    let mut stream_guard = state.stream.lock().unwrap();
    *stream_guard = None;
    println!("🛑 Recording Stopped");
    Ok("Recording Stopped".to_string())
}