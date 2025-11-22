import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus, Search, FolderGit2 } from 'lucide-react';
import { RepositoryCard } from '@/components/repos/RepositoryCard';
import { CreateRepoModal } from '@/components/repos/CreateRepoModal';
import { Loading } from '@/components/common/Loading';
import { ErrorMessage } from '@/components/common/ErrorMessage';
import { EmptyState } from '@/components/common/EmptyState';
import { listRepositories, createRepository } from '@/services/api';

export function HomePage() {
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const queryClient = useQueryClient();

  const { data: repos, isLoading, error, refetch } = useQuery({
    queryKey: ['repositories'],
    queryFn: async () => {
      const response = await listRepositories();
      return response.data;
    },
  });

  const createMutation = useMutation({
    mutationFn: async ({
      namespace,
      name,
      description,
    }: {
      namespace: string;
      name: string;
      description: string;
    }) => {
      await createRepository(namespace, name, { description });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    },
  });

  const filteredRepos = repos?.filter((repo) => {
    const searchLower = searchQuery.toLowerCase();
    return (
      repo.name.toLowerCase().includes(searchLower) ||
      repo.namespace.toLowerCase().includes(searchLower) ||
      repo.description?.toLowerCase().includes(searchLower)
    );
  });

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Repositories</h1>
          <p className="mt-1 text-sm text-gray-600">
            Version control for Logic Pro projects
          </p>
        </div>
        <button
          onClick={() => setIsCreateModalOpen(true)}
          className="btn-primary flex items-center space-x-2"
        >
          <Plus className="w-4 h-4" />
          <span>New Repository</span>
        </button>
      </div>

      <div className="mb-4">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400 pointer-events-none" aria-hidden="true" />
          <input
            type="search"
            placeholder="Search repositories..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="input w-full pl-10 shadow-sm"
            aria-label="Search repositories"
          />
        </div>
      </div>

      {isLoading && <Loading message="Loading repositories..." />}

      {error && (
        <ErrorMessage
          message="Failed to load repositories"
          onRetry={() => refetch()}
        />
      )}

      {!isLoading && !error && filteredRepos && (
        <>
          {filteredRepos.length === 0 ? (
            searchQuery ? (
              <EmptyState
                icon={Search}
                title="No repositories found"
                description={`No repositories match "${searchQuery}"`}
              />
            ) : (
              <EmptyState
                icon={FolderGit2}
                title="No repositories yet"
                description="Create your first repository to get started with Logic Pro version control"
                action={{
                  label: 'Create Repository',
                  onClick: () => setIsCreateModalOpen(true),
                }}
              />
            )
          ) : (
            <>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {filteredRepos.map((repo) => (
                  <RepositoryCard key={`${repo.namespace}/${repo.name}`} repo={repo} />
                ))}
              </div>
              <p className="mt-4 text-sm text-gray-500 text-center">
                Showing {filteredRepos.length} of {repos?.length ?? 0} repositories
              </p>
            </>
          )}
        </>
      )}

      <CreateRepoModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        onSubmit={async (namespace, name, description) => {
          await createMutation.mutateAsync({ namespace, name, description });
        }}
      />
    </div>
  );
}
