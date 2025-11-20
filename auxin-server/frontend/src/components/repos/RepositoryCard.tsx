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
      className="card hover:shadow-md transition-all duration-200 cursor-pointer group hover:border-primary-200"
      aria-label={`View repository ${repo.namespace}/${repo.name}`}
    >
      <div className="flex items-start space-x-4">
        <div className="bg-primary-100 p-3 rounded-lg group-hover:bg-primary-200 transition-colors duration-200">
          <FolderGit2 className="w-6 h-6 text-primary-600 group-hover:scale-110 transition-transform duration-200" aria-hidden="true" />
        </div>
        <div className="flex-1 min-w-0">
          <h3 className="text-lg font-semibold text-gray-900 truncate group-hover:text-primary-600 transition-colors duration-200">
            {repo.namespace}/{repo.name}
          </h3>
          {repo.description && (
            <p className="mt-1 text-sm text-gray-600 line-clamp-2 leading-relaxed">
              {repo.description}
            </p>
          )}
          <p className="mt-2 text-xs text-gray-500 font-mono truncate bg-gray-50 px-2 py-1 rounded inline-block">
            {repo.path}
          </p>
        </div>
      </div>
    </Link>
  );
}
