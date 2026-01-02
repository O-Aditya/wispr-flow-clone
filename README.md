# 🎙️ My AI Flow (Wispr Flow Clone)

> A high-performance, cross-platform voice-to-text desktop application built with **Rust (Tauri)** and **React**.

## 📖 Overview

This project is a technical implementation of a real-time voice dictation app inspired by **Wispr Flow**. It allows users to press a button, speak into their microphone, and see their speech converted to text instantly using AI.

Unlike web-based recorders, this app runs as a native desktop binary, offering superior performance, global system integration, and raw audio control.

## ✨ Key Features

* **⚡ Real-Time Streaming:** Audio is streamed via WebSockets to Deepgram for near-instant transcription.
* **🎙️ Native Audio Capture:** Uses Rust's `cpal` library to capture raw PCM audio directly from hardware (bypassing browser throttling).
* **🧠 AI-Powered:** Powered by Deepgram's `Nova-3` model for high-accuracy speech recognition.
* **🔄 Instant Feedback:** Displays "Interim" (gray) results while speaking and "Final" (white) results upon completion.
* **🛡️ Secure Architecture:** API keys are protected using compile-time environment variables (`dotenv-codegen`).
* **🎨 Modern UI:** A clean, dark-themed interface built with React and Lucide icons.

---

## 🛠️ Technical Stack

| Component | Technology | Reasoning |
| --- | --- | --- |
| **Backend** | **Rust** (Tauri) | For native performance, thread safety, and direct hardware access. |
| **Frontend** | **React** (TypeScript) | For a reactive, component-based UI that handles rapid state updates. |
| **Audio** | **Cpal** | The standard Rust library for low-level cross-platform audio input. |
| **AI Service** | **Deepgram API** | Chosen for its low-latency WebSocket API and high accuracy. |
| **Network** | **Tokio + Tungstenite** | Async runtime and WebSocket client for non-blocking streaming. |
| **Styling** | **CSS Modules** | Custom dark mode design with a focus on minimalism. |

---

## 🚀 How It Works (Architecture)

The app follows a **Client-Server-Service** architecture packed into a single desktop binary:

1. **Capture:** The Rust backend spawns a dedicated audio thread using `cpal`. It requests **Mono (1 Channel)** audio to match the AI model's requirements.
2. **Process:** Raw audio frames (F32) are mathematically converted to Integers (I16) to ensure compatibility with 16-bit PCM standards.
3. **Stream:** A `Tokio` async task pipes the audio data through a secure WebSocket connection to Deepgram.
4. **Transcribe:** Deepgram returns JSON events. We parse `is_final` flags to distinguish between "ghost text" (interim) and committed sentences.
5. **Render:** The React frontend listens for Rust events (`transcription`) and updates the UI state in real-time.

---

## 🚧 Challenges & Solutions

During development, we encountered and solved several complex engineering hurdles:

### 1. The "Chipmunk" Audio Effect 🐿️

* **Problem:** The audio transcription was garbled and fast-forwarded.
* **Cause:** Modern microphones default to **Stereo (2 Channels)**, but Deepgram expects **Mono**. Sending stereo data as mono doubled the playback speed.
* **Solution:** We implemented logic in Rust to explicitly query the hardware for a Mono configuration (`channels: 1`). If unavailable, we manually process the buffer to mix down channels.

### 2. Thread Safety in Rust 🔒

* **Problem:** Compiler errors like `*mut () cannot be sent between threads`.
* **Cause:** Raw audio stream pointers from Windows (WASAPI) are not thread-safe by default.
* **Solution:** We created a `DeepgramStream` wrapper struct and implemented `unsafe impl Send` to guarantee safe transport across thread boundaries within a `Mutex`.

### 3. Connection Security 🔐

* **Problem:** Hardcoding API keys in `audio.rs` is a security risk.
* **Solution:** Integrated `dotenv-codegen`. The API key is now read from a local `.env` file at **compile time**, ensuring it never appears in the source code or git history.

### 4. WebSocket Handshakes 🤝

* **Problem:** Initial `400 Bad Request` and `401 Unauthorized` errors.
* **Cause:** Manual WebSocket header construction is fragile.
* **Solution:** We switched to passing the API Key via the URL query parameter (`access_token=...`) and utilized `tokio-tungstenite` to handle the complex handshake headers automatically.

---

## 📥 Installation & Setup

### Prerequisites

* **Node.js** (v18+)
* **Rust** (Latest Stable)
* **Deepgram API Key**

### Steps

1. **Clone the repo**
```bash
git clone https://github.com/O-Aditya/wispr-flow-clone.git
cd wispr-flow-clone

```


2. **Install Frontend Dependencies**
```bash
npm install

```


3. **Setup Environment Variables**
Create a file named `.env` in the `src-tauri` folder:
```env
DEEPGRAM_API_KEY=your_actual_key_here

```


4. **Run Development Mode**
```bash
npm run tauri dev

```



---

## 🔮 Future Improvements

* **Global Hotkeys:** Implement system-wide shortcuts (e.g., `Ctrl+Space`) to toggle recording even when minimized.
* **Text Injection:** Use `enigo` crate to type the transcribed text directly into other apps (Slack, Notepad).
* **Audio Waveform:** Add a visualizer in the UI to show microphone input levels.

---

## 📄 License

This project is open-source and available under the **MIT License**.