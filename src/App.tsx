import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sidebar } from "./components/Sidebar";
import { ChatTerminal } from "./components/ChatTerminal";
import { CommandBar } from "./components/CommandBar";
import { DragonIcon } from "./components/DragonIcon";
import { LayoutGrid, Cloud, Database } from "lucide-react";

export default function App() {
  const MAX_HISTORY_MESSAGES = 30;
  const [sessions, setSessions] = useState([
    { id: "SESS_001", title: "Core Optimization", messages: [{ role: "system", content: "Connection established. Awaiting architect input." }] }
  ]);
  const [activeId, setActiveId] = useState("SESS_001");
  const [isProcessing, setIsProcessing] = useState(false);
  const [isCapturing, setIsCapturing] = useState(false);

  const activeSession = sessions.find(s => s.id === activeId) || sessions[0];

  const handleSend = async (text: string) => {
    if (!text || isProcessing) return;
    setIsProcessing(true);

    const newMessages = [...activeSession.messages, { role: "user", content: text }];
    setSessions(prev => prev.map(s => s.id === activeId ? { ...s, messages: newMessages } : s));

    // Limita el historial para reducir uso de memoria/tokens por request.
    const historyForPython = newMessages
      .slice(-MAX_HISTORY_MESSAGES)
      .filter(m => m.role === "user" || m.role === "system")
      .map(m => ({ 
        role: m.role === "user" ? "user" : "model", 
        content: m.content 
      }));

    try {
      const res: string = await invoke("invocar_agente_multimodal", { 
        prompt: text, 
        agente: "Architect",
        history: historyForPython
      });
      
      setSessions(prev => prev.map(s => s.id === activeId ? { ...s, messages: [...newMessages, { role: "system", content: res }] } : s));
    } catch (e) {
      setSessions(prev => prev.map(s => s.id === activeId ? { ...s, messages: [...newMessages, { role: "system", content: `ERR: ${e}` }] } : s));
    } finally {
      setIsProcessing(false);
    }
  };

  const handleCaptureScreenshot = async () => {
    if (isProcessing || isCapturing) return;
    setIsCapturing(true);

    try {
      const screenshot = await invoke<{
        path: string;
        capture_method: string;
        ocr_text?: string | null;
      }>("capturar_screenshot_seleccion");

      const ocrText = screenshot.ocr_text?.trim();
      const screenshotPrompt = [
        "<FODORIAN_SCREENSHOT>",
        `path: ${screenshot.path}`,
        `capture_method: ${screenshot.capture_method}`,
        ocrText ? "ocr_text:" : "ocr_text: unavailable",
        ocrText || "",
        "</FODORIAN_SCREENSHOT>",
        "",
        "Analiza esta captura en tiempo real como copiloto tecnico. Prioriza errores, stack traces, alertas y acciones concretas."
      ]
        .filter(Boolean)
        .join("\n");

      await handleSend(screenshotPrompt);
    } catch (e) {
      await handleSend(`No se pudo capturar screenshot: ${e}`);
    } finally {
      setIsCapturing(false);
    }
  };

  return (
    <div className="h-screen flex overflow-hidden bg-[#050505] text-[#ff6a00] font-mono relative">
      <div className="scanline"></div>
      
      <nav className="w-16 border-r border-[#8a2be2]/20 flex flex-col items-center py-6 gap-6 bg-black">
        <div className="p-2 text-[#8a2be2] shadow-[0_0_10px_rgba(138,43,226,0.5)]"><LayoutGrid size={24}/></div>
        <Cloud className="opacity-20 hover:opacity-100 cursor-pointer transition-all" size={20}/>
        <Database className="opacity-20 hover:opacity-100 cursor-pointer transition-all" size={20}/>
      </nav>

      <Sidebar 
        sessions={sessions} 
        activeId={activeId} 
        onSelect={setActiveId} 
        onNew={() => {
          const id = "SESS_" + Date.now();
          setSessions(prev => [{ id, title: "New Node", messages: [] }, ...prev]);
          setActiveId(id);
        }}
        onDelete={(id: string) => {
          setSessions(prev => {
            const newSessions = prev.filter(s => s.id !== id);
            if (newSessions.length === 0) {
              const newId = "SESS_" + Date.now();
              setActiveId(newId);
              return [{ id: newId, title: "New Node", messages: [] }];
            }

            if (activeId === id) setActiveId(newSessions[0].id);
            return newSessions;
          });
        }}
      />

      <main className="flex-1 flex flex-col relative">
        <header className="p-4 border-b border-[#8a2be2]/20 flex justify-between items-center bg-black/40">
          <span className="text-[10px] tracking-[0.3em] text-[#8a2be2] uppercase">~/SESSION/{activeSession.title}</span>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2 text-[9px] text-green-500">
              <div className={`w-1.5 h-1.5 rounded-full bg-green-500 ${isProcessing ? 'animate-ping' : ''}`}></div>
              {(isProcessing || isCapturing) ? 'PROCESSING' : 'OPTIMAL'}
            </div>
            <DragonIcon />
          </div>
        </header>

        <ChatTerminal 
          sessionId={activeSession.id}
          messages={activeSession.messages} 
          onApprove={async (cmd: string) => {
            try {
              const res: string = await invoke("ejecutar_comando_sandbox", { comando: cmd });
              await handleSend(`[STDOUT]:\n${res}`);
            } catch (e) {
              await handleSend(`Error ejecutando comando sandbox: ${e}`);
            }
          }} 
          onReject={(cmd: string) => {
            handleSend(`He rechazado la ejecución del comando: ${cmd}. Propón otra alternativa.`);
          }}
        />
        
        <CommandBar onSend={handleSend} onCaptureScreenshot={handleCaptureScreenshot} disabled={isProcessing || isCapturing} />
      </main>
    </div>
  );
}
