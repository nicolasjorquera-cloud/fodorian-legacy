import ReactMarkdown from 'react-markdown';
import { ShieldAlert, CheckCircle, XCircle, TerminalSquare } from "lucide-react";
import { useEffect, useState } from 'react';

interface Message {
  role: string;
  content: string;
}

interface ChatTerminalProps {
  sessionId: string;
  messages: Message[];
  onApprove: (cmd: string) => void;
  onReject: (cmd: string) => void;
}

export const ChatTerminal = ({ sessionId, messages, onApprove, onReject }: ChatTerminalProps) => {
  // Estado para recordar qué comandos ya fueron procesados (aprobados o rechazados)
  const [processedCommands, setProcessedCommands] = useState<Set<number>>(new Set());

  useEffect(() => {
    setProcessedCommands(new Set());
  }, [sessionId]);

  const handleAction = (index: number, cmd: string, action: 'approve' | 'reject') => {
    // Marcar este comando como procesado para ocultar los botones
    setProcessedCommands(prev => new Set(prev).add(index));
    
    if (action === 'approve') {
      onApprove(cmd);
    } else {
      onReject(cmd);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto p-12 space-y-8">
      {messages.map((m: Message, i: number) => {
        // Buscar la etiqueta <FODORIAN_EXEC>
        const execMatch = m.content.match(/<FODORIAN_EXEC>(.*?)<\/FODORIAN_EXEC>/s);
        const cleanText = m.content.replace(/<FODORIAN_EXEC>.*?<\/FODORIAN_EXEC>/s, "").trim();
        const isProcessed = processedCommands.has(i);

        return (
          <div key={i} className={`flex flex-col ${m.role === 'user' ? 'items-end' : 'items-start'}`}>
            <span className="text-[8px] mb-2 opacity-20 tracking-[0.3em] uppercase">
              {m.role === 'user' ? 'Architect' : 'System'}
            </span>
            
            <div className={`max-w-[80%] p-6 rounded-sm border ${m.role === 'user' ? 'border-[#ff6a00]/20 bg-[#ff6a00]/5' : 'border-[#ff6a00]/40 bg-[#0d0700]'}`}>
              
              {/* TEXTO DEL MENSAJE */}
              <div className="text-sm leading-relaxed prose prose-invert prose-orange">
                <ReactMarkdown>{cleanText}</ReactMarkdown>
              </div>
              
              {/* UI DE APROBACIÓN HIL (Solo si hay comando y no ha sido procesado) */}
              {execMatch && !isProcessed && (
                <div className="mt-6 border border-red-500/50 bg-red-500/10 p-4 rounded shadow-[0_0_15px_rgba(255,0,0,0.1)]">
                  <p className="text-[10px] text-red-400 mb-3 tracking-widest flex items-center gap-2">
                    <ShieldAlert size={12}/> RCE REQUESTED
                  </p>
                  <code className="block bg-black p-3 text-xs text-red-300 mb-4 border border-red-500/20 font-mono">
                    {execMatch[1]}
                  </code>
                  <div className="flex gap-3">
                    <button 
                      onClick={() => handleAction(i, execMatch[1], 'approve')} 
                      className="bg-green-600/20 text-green-500 border border-green-500/50 px-4 py-2 text-[10px] hover:bg-green-600/40 transition-all flex items-center gap-2 rounded"
                    >
                      <CheckCircle size={12}/> APPROVE
                    </button>
                    <button 
                      onClick={() => handleAction(i, execMatch[1], 'reject')} 
                      className="bg-red-600/20 text-red-500 border border-red-500/50 px-4 py-2 text-[10px] hover:bg-red-600/40 transition-all flex items-center gap-2 rounded"
                    >
                      <XCircle size={12}/> REJECT
                    </button>
                  </div>
                </div>
              )}

              {/* INDICADOR DE COMANDO PROCESADO */}
              {execMatch && isProcessed && (
                <div className="mt-4 flex items-center gap-2 text-[10px] text-[#ff6a00] opacity-50">
                  <TerminalSquare size={12} /> Comando procesado por el Soberano.
                </div>
              )}

            </div>
          </div>
        );
      })}
    </div>
  );
};
