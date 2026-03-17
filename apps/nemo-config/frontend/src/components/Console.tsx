import { useEffect, useRef } from 'react';
import { Terminal } from 'lucide-react';

interface ConsoleProps {
  output: string[];
}

export function Console({ output }: ConsoleProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [output]);

  return (
    <div className="h-full flex flex-col bg-gray-900">
      <div className="flex items-center gap-2 px-4 py-2 bg-gray-800 border-b border-gray-700">
        <Terminal size={16} className="text-gray-400" />
        <span className="text-sm font-medium text-gray-300">Deployment Console</span>
      </div>
      <div 
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-4 font-mono text-sm"
      >
        {output.length === 0 ? (
          <p className="text-gray-500 italic">No output yet. Start a deployment to see logs...</p>
        ) : (
          output.map((line, i) => (
            <div key={i} className="text-gray-300 mb-1">
              <span className="text-gray-600 mr-2">[{new Date().toLocaleTimeString()}]</span>
              {line}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
