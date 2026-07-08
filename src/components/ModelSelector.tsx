import { useState, useRef, useEffect } from "react";
import { useApp } from "@/contexts";
import { Check, ChevronDown, Sparkles } from "lucide-react";

export function ModelSelector() {
  const {
    selectedAIProvider,
    allAiProviders,
    onSetSelectedAIProvider,
  } = useApp();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const current = allAiProviders.find(
    (p) => p.id === selectedAIProvider.provider
  );
  const label = current?.id ?? selectedAIProvider.provider || "No provider";

  return (
    <div ref={ref} className="relative select-none">
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1 px-2 py-1 text-[11px] rounded-md border border-input/40 bg-muted/30 hover:bg-muted/60 text-muted-foreground transition-colors cursor-pointer whitespace-nowrap"
        title="Switch AI provider"
      >
        <Sparkles className="h-3 w-3" />
        <span className="max-w-[100px] truncate">{label}</span>
        <ChevronDown className="h-3 w-3 opacity-60" />
      </button>

      {open && (
        <div className="absolute left-0 top-full mt-1 z-50 w-56 max-h-72 overflow-y-auto rounded-md border bg-popover p-1 shadow-lg">
          {allAiProviders.map((p) => {
            const active = p.id === selectedAIProvider.provider;
            return (
              <button
                key={p.id}
                onClick={() => {
                  onSetSelectedAIProvider({
                    provider: p.id!,
                    variables: selectedAIProvider.variables,
                  });
                  setOpen(false);
                }}
                className={`flex w-full items-center gap-2 px-3 py-1.5 text-xs rounded-sm transition-colors cursor-pointer ${
                  active
                    ? "bg-accent text-accent-foreground font-medium"
                    : "text-muted-foreground hover:bg-accent/50"
                }`}
              >
                <span className="flex-1 text-left truncate">{p.id}</span>
                {active && <Check className="h-3 w-3 shrink-0" />}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
