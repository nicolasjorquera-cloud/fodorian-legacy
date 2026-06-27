import { Send, Mic, Paperclip, Loader2, Camera } from "lucide-react";
import { useRef, useState } from "react";

export const CommandBar = ({ onSend, onCaptureScreenshot, disabled }: any) => {
  const [val, setVal] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);
  const MAX_ATTACHMENT_CHARS = 20000;

  const submit = () => {
    if (!val || disabled) return;
    onSend(val);
    setVal("");
  };

  const openFilePicker = () => {
    if (disabled) return;
    fileInputRef.current?.click();
  };

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || disabled) return;

    const resetInput = () => {
      if (fileInputRef.current) fileInputRef.current.value = "";
    };

    try {
      const text = await file.text();
      const trimmedText = text.length > MAX_ATTACHMENT_CHARS
        ? `${text.slice(0, MAX_ATTACHMENT_CHARS)}\n\n[...truncated ${text.length - MAX_ATTACHMENT_CHARS} chars]`
        : text;

      const attachmentMessage = [
        "<FODORIAN_ATTACHMENT>",
        `name: ${file.name}`,
        `type: ${file.type || "unknown"}`,
        `size_bytes: ${file.size}`,
        "content:",
        trimmedText,
        "</FODORIAN_ATTACHMENT>"
      ].join("\n");

      onSend(attachmentMessage);
    } catch (_err) {
      // If browser/OS refuses text read (binary or protected), still send metadata.
      onSend(
        [
          "<FODORIAN_ATTACHMENT>",
          `name: ${file.name}`,
          `type: ${file.type || "unknown"}`,
          `size_bytes: ${file.size}`,
          "content_unavailable: true",
          "</FODORIAN_ATTACHMENT>"
        ].join("\n")
      );
    } finally {
      resetInput();
    }
  };

  return (
    <footer className="p-12 pt-0">
      <div className="flex items-center gap-4 bg-[#0a0a0a] border border-white/10 p-4 rounded focus-within:border-[#8a2be2]/50 transition-all">
        <button
          onClick={openFilePicker}
          disabled={disabled}
          className="opacity-20 hover:opacity-100 transition-opacity disabled:opacity-10"
          aria-label="Attach file"
          title="Attach file"
        >
          <Paperclip size={20} className="cursor-pointer" />
        </button>
        <input
          ref={fileInputRef}
          type="file"
          className="hidden"
          onChange={handleFileSelect}
          disabled={disabled}
        />
        <input 
          className="flex-1 bg-transparent outline-none text-sm"
          value={val}
          onChange={(e) => setVal(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && submit()}
          placeholder="Enter command..."
          disabled={disabled}
        />
        <button
          onClick={onCaptureScreenshot}
          disabled={disabled}
          className="opacity-20 hover:opacity-100 transition-opacity disabled:opacity-10"
          aria-label="Capture screenshot"
          title="Capture screenshot"
        >
          <Camera size={20} className="cursor-pointer" />
        </button>
        <Mic size={20} className="opacity-20 hover:opacity-100 cursor-pointer" />
        <button onClick={submit} disabled={disabled} className="text-[#ff6a00] p-2 hover:bg-[#ff6a00]/10 rounded disabled:opacity-20">
          {disabled ? <Loader2 className="animate-spin" size={20}/> : <Send size={20} />}
        </button>
      </div>
    </footer>
  );
};
