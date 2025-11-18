import { Link } from 'react-router-dom';
import { FolderGit2 } from 'lucide-react';
import type { Repository } from '@/types';

interface RepositoryCardProps {
  repo: Repository;
}

export function RepositoryCard({ repo }: RepositoryCardProps) {
  return (
    <Link
      to={`/${repo.namespace}/${repo.name}`}
      className="card hover:shadow-md transition-shadow cursor-pointer"
    >
      <div className="flex items-start space-x-4">
        <div className="bg-primary-100 p-3 rounded-lg">
          <FolderGit2 className="w-6 h-6 text-primary-600" />
        </div>
        <div className="flex-1 min-w-0">
          <h3 className="text-lg font-semibold text-gray-900 truncate">
            {repo.namespace}/{repo.name}
          </h3>
          {repo.description && (
            <p className="mt-1 text-sm text-gray-600 line-clamp-2">
              {repo.description}
            </p>
          )}
          <p className="mt-2 text-xs text-gray-500 font-mono truncate">
            {repo.path}
          </p>
        </div>
      </div>
    </Link>
  );
}
