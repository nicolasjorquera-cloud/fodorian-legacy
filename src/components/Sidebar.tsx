import { MessageSquare } from "lucide-react";

interface Session {
  id: string;
  title: string;
  messages: { role: string; content: string }[];
  date?: string;
}

interface SidebarProps {
  sessions: Session[];
  activeId: string;
  onNew: () => void;
  onSelect: (id: string) => void;
  onDelete: (id: string) => void;
}

export const Sidebar = ({ sessions, activeId, onNew, onSelect, onDelete }: SidebarProps) => (
  <aside className="w-72 border-r border-white/5 p-6 flex flex-col bg-[#050505]">
    <div className="mb-8">
      <h2 className="text-xl font-bold tracking-tighter text-fodorian-orange">FODORIAN OS <span className="text-[10px] bg-fodorian-orange text-black px-1 align-top ml-1">SYS</span></h2>
      <p className="text-[9px] opacity-30 tracking-[0.2em] mt-1">SOVEREIGN ARCHITECT NODE</p>
    </div>

    <button
      onClick={onNew}
      className="w-full border border-fodorian-orange/40 py-2 mb-8 hover:bg-fodorian-orange/10 transition-all text-[11px] tracking-widest"
    >
      + NEW SESSION
    </button>

    <div className="flex-1 overflow-y-auto">
      <p className="text-[9px] opacity-20 mb-4 tracking-widest uppercase">Active Sessions</p>
      <div className="space-y-3">
        {sessions.map((s: Session) => (
          <div
            key={s.id}
            onClick={() => onSelect(s.id)}
            className={`p-4 border transition-all cursor-pointer relative ${s.id === activeId ? 'border-fodorian-orange/60 bg-fodorian-orange/5' : 'border-white/5 opacity-40'}`}
          >
            {s.id === activeId && <div className="absolute left-0 top-0 w-1 h-full bg-fodorian-orange"></div>}
            <div className="flex items-center gap-3">
              <MessageSquare size={14} className={s.id === activeId ? 'text-fodorian-orange' : ''} />
              <div>
                <p className="text-xs">{s.title}</p>
                <p className="text-[8px] opacity-30 mt-1">{s.date}</p>
              </div>
            </div>
            <button
              onClick={(e) => {
                e.stopPropagation();
                onDelete(s.id);
              }}
              className="absolute top-2 right-2 text-[10px] px-1 opacity-40 hover:opacity-100"
              aria-label={`Delete ${s.title}`}
            >
              x
            </button>
          </div>
        ))}
      </div>
    </div>
  </aside>
);
