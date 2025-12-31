import "./App.css";
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { Mic, Square, Copy, Trash2, Sparkles } from "lucide-react";

function App() {
  const [fullTranscript, setFullTranscript] = useState("");
  const [interimTranscript, setInterimTranscript] = useState("");
  const [isRecording, setIsRecording] = useState(false);
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let unlisten: Promise<any>;
    const setupListener = async () => {
      unlisten = listen("transcription", (event: any) => {
        try {
          const data = JSON.parse(event.payload);
          const transcript = data.channel?.alternatives?.[0]?.transcript;
          const isFinal = data.is_final;

          if (transcript) {
            if (isFinal) {
              setFullTranscript((prev) => (prev ? prev + " " : "") + transcript);
              setInterimTranscript("");
            } else {
              setInterimTranscript(transcript);
            }
          }
        } catch (e) {
          console.error(e);
        }
      });
    };
    setupListener();
    return () => { if (unlisten) unlisten.then((f) => f()); };
  }, []);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [fullTranscript, interimTranscript]);

  const toggleRecording = async () => {
    if (isRecording) {
      await invoke("stop_recording");
      setIsRecording(false);
    } else {
      setInterimTranscript("");
      await invoke("start_recording");
      setIsRecording(true);
    }
  };

  const copyToClipboard = () => {
    if (!fullTranscript && !interimTranscript) return;
    navigator.clipboard.writeText(fullTranscript + " " + interimTranscript);
  };

  const clearText = () => {
    setFullTranscript("");
    setInterimTranscript("");
  };

  return (
    <div className="app-container">
      
    
      <div className="content-area">
        {fullTranscript === "" && interimTranscript === "" ? (
          <div className="placeholder">
            <Sparkles size={48} strokeWidth={1} />
            <span>Ready to capture thoughts...</span>
          </div>
        ) : (
          <>
            <span>{fullTranscript}</span>
            <span className="interim">{" " + interimTranscript}</span>
          </>
        )}
        <div ref={endRef} />
      </div>

      {/* FOOTER */}
      <div className="controls">
        <button className="icon-btn" onClick={clearText} title="Clear All">
          <Trash2 size={20} />
        </button>

        <button 
          className={`record-btn ${isRecording ? "recording" : "idle"}`} 
          onClick={toggleRecording}
        >
          {isRecording ? <Square size={24} fill="currentColor" /> : <Mic size={28} />}
        </button>

        <button className="icon-btn" onClick={copyToClipboard} title="Copy">
          <Copy size={20} />
        </button>
      </div>
    </div>
  );
}

export default App;