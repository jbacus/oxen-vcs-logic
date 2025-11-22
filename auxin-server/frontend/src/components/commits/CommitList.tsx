import { useState } from 'react';
import { formatDistanceToNow } from 'date-fns';
import { GitCommit, Clock, User, RotateCcw } from 'lucide-react';
import { BouncePreview } from '../bounces';
import { restoreCommit } from '@/services/api';
import type { Commit } from '@/types';

interface CommitListProps {
  commits: Commit[];
  namespace?: string;
  repoName?: string;
  showBounces?: boolean;
  onCommitClick?: (commit: Commit) => void;
  onRestore?: () => void;
}

export function CommitList({ commits, namespace, repoName, showBounces = true, onCommitClick, onRestore }: CommitListProps) {
  const [restoreTarget, setRestoreTarget] = useState<Commit | null>(null);
  const [isRestoring, setIsRestoring] = useState(false);
  const [error, setError] = useState('');

  const handleRestoreClick = (commit: Commit, e: React.MouseEvent) => {
    e.stopPropagation();
    setRestoreTarget(commit);
    setError('');
  };

  const confirmRestore = async () => {
    if (!restoreTarget || !namespace || !repoName) return;

    setIsRestoring(true);
    setError('');

    try {
      await restoreCommit(namespace, repoName, restoreTarget.id);
      setRestoreTarget(null);
      onRestore?.();
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to restore commit');
    } finally {
      setIsRestoring(false);
    }
  };

  const cancelRestore = () => {
    setRestoreTarget(null);
    setError('');
  };

  if (commits.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500 animate-in fade-in duration-300">
        No commits yet
      </div>
    );
  }

  return (
    <>
      <div className="space-y-2">
        {commits.map((commit, index) => (
          <div
            key={commit.id}
            onClick={() => onCommitClick?.(commit)}
            className={`card ${onCommitClick ? 'cursor-pointer hover:shadow-md hover:border-primary-200' : ''} transition-all duration-200 animate-in fade-in slide-in-from-bottom-2`}
            style={{ animationDelay: `${index * 50}ms` }}
            role={onCommitClick ? 'button' : undefined}
            tabIndex={onCommitClick ? 0 : undefined}
            onKeyDown={(e) => {
              if (onCommitClick && (e.key === 'Enter' || e.key === ' ')) {
                e.preventDefault();
                onCommitClick(commit);
              }
            }}
            aria-label={`Commit: ${commit.message}`}
          >
            <div className="flex items-start space-x-2">
              <div className="bg-gray-100 p-2 rounded-lg flex-shrink-0">
                <GitCommit className="w-5 h-5 text-gray-600" aria-hidden="true" />
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-gray-900 mb-1 leading-relaxed">
                  {commit.message}
                </p>
                <div className="flex items-center space-x-3 text-xs text-gray-500 flex-wrap gap-y-1">
                  <span className="font-mono bg-gray-50 px-2 py-0.5 rounded">{commit.id.substring(0, 8)}</span>
                  {commit.author && (
                    <span className="flex items-center space-x-1">
                      <User className="w-3 h-3" aria-hidden="true" />
                      <span>{commit.author}</span>
                    </span>
                  )}
                  <span className="flex items-center space-x-1">
                    <Clock className="w-3 h-3" aria-hidden="true" />
                    <span>{formatDistanceToNow(new Date(commit.timestamp), { addSuffix: true })}</span>
                  </span>
                </div>
                {showBounces && namespace && repoName && (
                  <BouncePreview
                    namespace={namespace}
                    repoName={repoName}
                    commitId={commit.id}
                    compact
                  />
                )}
              </div>
              {namespace && repoName && (
                <button
                  onClick={(e) => handleRestoreClick(commit, e)}
                  className="btn-secondary text-xs px-3 py-1.5 flex items-center space-x-1 flex-shrink-0"
                  title="Restore to this version"
                  aria-label={`Restore to commit ${commit.id.substring(0, 8)}`}
                >
                  <RotateCcw className="w-3.5 h-3.5" aria-hidden="true" />
                  <span>Restore</span>
                </button>
              )}
            </div>
          </div>
        ))}
      </div>

      {/* Confirmation Dialog */}
      {restoreTarget && (
        <div
          className="fixed inset-0 bg-black bg-opacity-50 backdrop-blur-sm flex items-center justify-center z-50 animate-in fade-in duration-200"
          onClick={cancelRestore}
          role="dialog"
          aria-modal="true"
          aria-labelledby="restore-dialog-title"
        >
          <div
            className="bg-white rounded-lg shadow-xl max-w-md w-full mx-4 animate-in zoom-in-95 duration-200"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="p-6">
              <h3 id="restore-dialog-title" className="text-lg font-semibold text-gray-900 mb-2">
                Restore to Previous Version?
              </h3>
              <p className="text-sm text-gray-600 mb-4">
                This will restore your project to the state of commit:
              </p>
              <div className="bg-gray-50 rounded-lg p-3 mb-4">
                <p className="text-sm font-medium text-gray-900 mb-1">{restoreTarget.message}</p>
                <p className="text-xs text-gray-500 font-mono">{restoreTarget.id.substring(0, 8)}</p>
              </div>
              <p className="text-sm text-amber-600 mb-6">
                <strong>Warning:</strong> Your current working directory will be updated to match this commit. Make sure you have saved any important changes.
              </p>

              {error && (
                <div className="bg-red-50 border border-red-200 rounded-lg p-3 mb-4 text-sm text-red-700 animate-in fade-in slide-in-from-top-1 duration-200" role="alert">
                  {error}
                </div>
              )}

              <div className="flex justify-end space-x-3">
                <button
                  onClick={cancelRestore}
                  className="btn-secondary"
                  disabled={isRestoring}
                >
                  Cancel
                </button>
                <button
                  onClick={confirmRestore}
                  className="btn-primary bg-amber-600 hover:bg-amber-700 min-w-[120px]"
                  disabled={isRestoring}
                >
                  {isRestoring ? (
                    <span className="flex items-center justify-center">
                      <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      Restoring...
                    </span>
                  ) : (
                    'Restore'
                  )}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
