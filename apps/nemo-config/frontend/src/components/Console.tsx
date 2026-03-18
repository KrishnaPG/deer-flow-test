import { useEffect, useRef } from 'react';
import { Terminal, Activity } from 'lucide-react';

interface ConsoleProps {
  output: string[];
  mode?: 'deployment' | 'container';
}

export function Console({ output, mode = 'deployment' }: ConsoleProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [output]);

  const title = mode === 'container' ? 'Container Logs' : 'Deployment Console';
  const Icon = mode === 'container' ? Activity : Terminal;
  const placeholder = mode === 'container' 
    ? 'Waiting for container logs...' 
    : 'No output yet. Start a deployment to see logs...';

  return (
    <div className="h-full flex flex-col bg-gray-900">
      <div className="flex items-center gap-2 px-4 py-2 bg-gray-800 border-b border-gray-700">
        <Icon size={16} className="text-gray-400" />
        <span className="text-sm font-medium text-gray-300">{title}</span>
        {mode === 'container' && (
          <span className="ml-2 text-xs text-gray-500">(auto-refresh)</span>
        )}
      </div>
      <div 
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-4 font-mono text-sm"
      >
        {output.length === 0 ? (
          <p className="text-gray-500 italic">{placeholder}</p>
        ) : (
          output.map((line, i) => (
            <div key={i} className="text-gray-300 whitespace-pre-wrap break-all">
              {line}
            </div>
          ))
        )}
      </div>
    </div>
  );
}