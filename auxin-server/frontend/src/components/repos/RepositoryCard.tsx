import { useState } from 'react';
import { Link } from 'react-router-dom';
import { FolderGit2, Download, Check } from 'lucide-react';
import type { Repository } from '@/types';

interface RepositoryCardProps {
  repo: Repository;
}

export function RepositoryCard({ repo }: RepositoryCardProps) {
  const [showClone, setShowClone] = useState(false);
  const [copied, setCopied] = useState(false);

  const cloneCommand = `auxin clone ${window.location.origin}/${repo.namespace}/${repo.name} ${repo.name}`;

  const handleCopyClone = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();

    try {
      await navigator.clipboard.writeText(cloneCommand);
      setCopied(true);
      setTimeout(() => {
        setCopied(false);
        setShowClone(false);
      }, 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const handleCloneClick = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setShowClone(!showClone);
  };

  return (
    <div className="relative">
      <Link
        to={`/${repo.namespace}/${repo.name}`}
        className="card hover:shadow-md transition-all duration-200 cursor-pointer group hover:border-primary-200 block"
        aria-label={`View repository ${repo.namespace}/${repo.name}`}
      >
        <div className="flex items-start space-x-4">
          <div className="bg-primary-100 p-3 rounded-lg group-hover:bg-primary-200 transition-colors duration-200">
            <FolderGit2 className="w-6 h-6 text-primary-600 group-hover:scale-110 transition-transform duration-200" aria-hidden="true" />
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-start justify-between">
              <h3 className="text-lg font-semibold text-gray-900 truncate group-hover:text-primary-600 transition-colors duration-200">
                {repo.namespace}/{repo.name}
              </h3>
              <button
                onClick={handleCloneClick}
                className="ml-2 p-1.5 rounded-lg hover:bg-primary-100 text-primary-600 transition-all duration-200 flex-shrink-0"
                title="Clone repository"
                aria-label="Show clone command"
              >
                <Download className="w-4 h-4" aria-hidden="true" />
              </button>
            </div>
            {repo.description && (
              <p className="mt-1 text-sm text-gray-600 line-clamp-2 leading-relaxed">
                {repo.description}
              </p>
            )}
            <div className="mt-2 text-xs text-gray-500 font-mono bg-gray-50 px-2 py-1 rounded max-w-full" title={repo.path}>
              <p className="truncate">{repo.path}</p>
            </div>
          </div>
        </div>
      </Link>

      {/* Clone Tooltip */}
      {showClone && (
        <div className="absolute top-full left-0 right-0 mt-2 z-10 bg-white border border-primary-200 rounded-lg shadow-lg p-3 animate-in fade-in slide-in-from-top-2 duration-200">
          <div className="flex items-center justify-between mb-2">
            <p className="text-xs font-semibold text-gray-700">Clone command:</p>
            <button
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                setShowClone(false);
              }}
              className="text-gray-400 hover:text-gray-600 text-xs"
              aria-label="Close clone tooltip"
            >
              ✕
            </button>
          </div>
          <div className="flex items-center space-x-2 bg-gray-50 border border-gray-200 rounded p-2">
            <code className="text-xs font-mono text-gray-800 flex-1 break-all select-all">
              {cloneCommand}
            </code>
            <button
              onClick={handleCopyClone}
              className="p-1.5 rounded hover:bg-gray-200 flex-shrink-0 transition-colors duration-200"
              title={copied ? 'Copied!' : 'Copy to clipboard'}
              aria-label={copied ? 'Copied to clipboard' : 'Copy to clipboard'}
            >
              {copied ? (
                <Check className="w-3.5 h-3.5 text-green-600" aria-hidden="true" />
              ) : (
                <Download className="w-3.5 h-3.5 text-gray-600" aria-hidden="true" />
              )}
            </button>
          </div>
          <p className="mt-2 text-xs text-gray-500">
            Click the repository for more clone options
          </p>
        </div>
      )}
    </div>
  );
}
