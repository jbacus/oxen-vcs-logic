import { formatDistanceToNow } from 'date-fns';
import { GitCommit, Clock, User } from 'lucide-react';
import { BouncePreview } from '../bounces';
import type { Commit } from '@/types';

interface CommitListProps {
  commits: Commit[];
  namespace?: string;
  repoName?: string;
  showBounces?: boolean;
  onCommitClick?: (commit: Commit) => void;
}

export function CommitList({ commits, namespace, repoName, showBounces = true, onCommitClick }: CommitListProps) {
  if (commits.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500 animate-in fade-in duration-300">
        No commits yet
      </div>
    );
  }

  return (
    <div className="space-y-3">
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
          <div className="flex items-start space-x-3">
            <div className="bg-gray-100 p-2 rounded-lg flex-shrink-0">
              <GitCommit className="w-5 h-5 text-gray-600" aria-hidden="true" />
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-gray-900 mb-1 leading-relaxed">
                {commit.message}
              </p>
              <div className="flex items-center space-x-4 text-xs text-gray-500 flex-wrap gap-y-1">
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
          </div>
        </div>
      ))}
    </div>
  );
}
