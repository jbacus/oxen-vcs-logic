import { useState } from 'react';
import { Terminal, Download, Copy, Check, Info } from 'lucide-react';
import { CopyButton } from '@/components/common/CopyButton';

interface CloneInstructionsProps {
  namespace: string;
  name: string;
  serverUrl?: string;
}

export function CloneInstructions({ namespace, name, serverUrl = 'http://localhost:3000' }: CloneInstructionsProps) {
  const [cloneMethod, setCloneMethod] = useState<'auxin' | 'oxen'>('auxin');

  // Generate clone URLs
  const httpUrl = `${serverUrl}/${namespace}/${name}`;
  const auxinCloneCommand = `auxin clone ${httpUrl} ${name}`;
  const oxenCloneCommand = `oxen clone ${httpUrl} ${name}`;

  // Detect project type from name
  const projectType = name.endsWith('.logicx') ? 'Logic Pro'
    : name.endsWith('.skp') ? 'SketchUp'
    : name.endsWith('.blend') ? 'Blender'
    : 'project';

  return (
    <div className="space-y-4">
      {/* Quick Clone Section */}
      <div className="bg-gradient-to-br from-primary-50 to-primary-100 border border-primary-200 rounded-lg p-4 shadow-sm">
        <div className="flex items-start space-x-3">
          <Download className="w-5 h-5 text-primary-600 mt-0.5 flex-shrink-0" aria-hidden="true" />
          <div className="flex-1">
            <h3 className="text-sm font-semibold text-primary-900 mb-2">
              Clone this {projectType} project
            </h3>

            {/* Method Selector */}
            <div className="flex space-x-2 mb-3">
              <button
                onClick={() => setCloneMethod('auxin')}
                className={`px-3 py-1.5 text-xs font-medium rounded transition-all duration-200 ${
                  cloneMethod === 'auxin'
                    ? 'bg-primary-600 text-white shadow-sm'
                    : 'bg-white text-primary-700 hover:bg-primary-50 border border-primary-200'
                }`}
                aria-label="Use Auxin CLI to clone"
              >
                Auxin CLI (Recommended)
              </button>
              <button
                onClick={() => setCloneMethod('oxen')}
                className={`px-3 py-1.5 text-xs font-medium rounded transition-all duration-200 ${
                  cloneMethod === 'oxen'
                    ? 'bg-primary-600 text-white shadow-sm'
                    : 'bg-white text-primary-700 hover:bg-primary-50 border border-primary-200'
                }`}
                aria-label="Use Oxen CLI to clone"
              >
                Oxen CLI
              </button>
            </div>

            {/* Clone Command */}
            <div className="flex items-center space-x-2 bg-white border border-primary-200 rounded-lg p-3 shadow-sm">
              <Terminal className="w-4 h-4 text-primary-600 flex-shrink-0" aria-hidden="true" />
              <code className="text-sm font-mono text-gray-800 flex-1 select-all break-all" aria-label="Clone command">
                {cloneMethod === 'auxin' ? auxinCloneCommand : oxenCloneCommand}
              </code>
              <CopyButton text={cloneMethod === 'auxin' ? auxinCloneCommand : oxenCloneCommand} />
            </div>

            {/* Info Message */}
            {cloneMethod === 'auxin' && (
              <div className="mt-3 flex items-start space-x-2 text-xs text-primary-800 bg-primary-50 p-2 rounded border border-primary-200">
                <Info className="w-3.5 h-3.5 mt-0.5 flex-shrink-0" aria-hidden="true" />
                <p className="leading-relaxed">
                  Auxin CLI provides enhanced features for {projectType} projects including
                  automatic metadata tracking and lock management.
                </p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Detailed Instructions */}
      <details className="group">
        <summary className="cursor-pointer select-none text-sm font-medium text-gray-700 hover:text-primary-600 transition-colors duration-200 flex items-center space-x-2">
          <span className="group-open:rotate-90 transition-transform duration-200">▶</span>
          <span>Show detailed clone instructions</span>
        </summary>

        <div className="mt-4 space-y-4 pl-6 border-l-2 border-gray-200">
          {/* Prerequisites */}
          <div>
            <h4 className="text-sm font-semibold text-gray-900 mb-2">Prerequisites</h4>
            <ul className="text-sm text-gray-600 space-y-1 list-disc list-inside">
              {cloneMethod === 'auxin' ? (
                <>
                  <li>Install Auxin CLI: Follow the installation guide</li>
                  <li>Oxen CLI will be automatically used by Auxin</li>
                </>
              ) : (
                <li>Install Oxen CLI: <code className="bg-gray-100 px-1.5 py-0.5 rounded font-mono text-xs">pip install oxen-ai</code></li>
              )}
            </ul>
          </div>

          {/* Step by Step */}
          <div>
            <h4 className="text-sm font-semibold text-gray-900 mb-2">Step by Step</h4>
            <ol className="text-sm text-gray-600 space-y-2 list-decimal list-inside">
              <li>
                Open your terminal or command prompt
              </li>
              <li>
                Navigate to where you want to clone the project:
                <div className="mt-1 ml-5 bg-gray-50 border border-gray-200 rounded p-2 font-mono text-xs">
                  <code>cd ~/Projects</code>
                </div>
              </li>
              <li>
                Run the clone command:
                <div className="mt-1 ml-5 bg-gray-50 border border-gray-200 rounded p-2">
                  <div className="flex items-center justify-between">
                    <code className="font-mono text-xs break-all flex-1">
                      {cloneMethod === 'auxin' ? auxinCloneCommand : oxenCloneCommand}
                    </code>
                    <CopyButton text={cloneMethod === 'auxin' ? auxinCloneCommand : oxenCloneCommand} className="ml-2" />
                  </div>
                </div>
              </li>
              <li>
                Wait for the download to complete
              </li>
              <li>
                Open the project in {projectType === 'project' ? 'your application' : projectType}
              </li>
            </ol>
          </div>

          {/* Alternative URLs */}
          <div>
            <h4 className="text-sm font-semibold text-gray-900 mb-2">Alternative Clone URLs</h4>
            <div className="space-y-2">
              <div className="text-xs">
                <p className="text-gray-600 mb-1">HTTP URL (current):</p>
                <div className="flex items-center space-x-2 bg-gray-50 border border-gray-200 rounded p-2">
                  <code className="font-mono text-gray-800 flex-1 break-all">{httpUrl}</code>
                  <CopyButton text={httpUrl} />
                </div>
              </div>

              {/* Show file:// URL hint */}
              <div className="text-xs">
                <p className="text-gray-600 mb-1">File URL (for local network shares):</p>
                <div className="bg-gray-50 border border-gray-200 rounded p-2">
                  <code className="font-mono text-gray-800">file:///path/to/server/repos/{namespace}/{name}</code>
                </div>
              </div>
            </div>
          </div>

          {/* Next Steps */}
          {cloneMethod === 'auxin' && (
            <div>
              <h4 className="text-sm font-semibold text-gray-900 mb-2">After Cloning</h4>
              <ul className="text-sm text-gray-600 space-y-1 list-disc list-inside">
                <li>View commit history: <code className="bg-gray-100 px-1.5 py-0.5 rounded font-mono text-xs">auxin log</code></li>
                <li>Check repository status: <code className="bg-gray-100 px-1.5 py-0.5 rounded font-mono text-xs">auxin status</code></li>
                <li>Create commits: <code className="bg-gray-100 px-1.5 py-0.5 rounded font-mono text-xs">auxin commit -m "message"</code></li>
                <li>Acquire a lock before editing: <code className="bg-gray-100 px-1.5 py-0.5 rounded font-mono text-xs">auxin lock acquire</code></li>
              </ul>
            </div>
          )}
        </div>
      </details>
    </div>
  );
}
