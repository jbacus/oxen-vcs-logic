import { useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ArrowLeft, GitBranch, Terminal, Download } from 'lucide-react';
import { CommitList } from '@/components/commits/CommitList';
import { MetadataViewer } from '@/components/metadata/MetadataViewer';
import { LockManager } from '@/components/locks/LockManager';
import { CloneInstructions } from '@/components/repos/CloneInstructions';
import { Loading } from '@/components/common/Loading';
import { ErrorMessage } from '@/components/common/ErrorMessage';
import {
  getRepository,
  getCommits,
  getLockStatus,
  acquireLock,
  releaseLock,
  heartbeatLock,
  getMetadata,
} from '@/services/api';

type TabType = 'commits' | 'locks' | 'metadata' | 'clone';

export function RepoPage() {
  const { namespace, name } = useParams<{ namespace: string; name: string }>();
  const [activeTab, setActiveTab] = useState<TabType>('clone');
  const [selectedCommit, setSelectedCommit] = useState<string | null>(null);
  const queryClient = useQueryClient();

  if (!namespace || !name) {
    return <div>Invalid repository</div>;
  }

  const { data: repo, isLoading: repoLoading, error: repoError } = useQuery({
    queryKey: ['repository', namespace, name],
    queryFn: async () => {
      const response = await getRepository(namespace, name);
      return response.data;
    },
  });

  const { data: commits, isLoading: commitsLoading } = useQuery({
    queryKey: ['commits', namespace, name],
    queryFn: async () => {
      const response = await getCommits(namespace, name);
      return response.data;
    },
    enabled: activeTab === 'commits',
  });

  const { data: lockInfo, isLoading: lockLoading } = useQuery({
    queryKey: ['lock', namespace, name],
    queryFn: async () => {
      const response = await getLockStatus(namespace, name);
      return response.data;
    },
    enabled: activeTab === 'locks',
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  const { data: metadata, isLoading: metadataLoading } = useQuery({
    queryKey: ['metadata', namespace, name, selectedCommit],
    queryFn: async () => {
      if (!selectedCommit) return null;
      try {
        const response = await getMetadata(namespace, name, selectedCommit);
        return response.data;
      } catch (err) {
        return null;
      }
    },
    enabled: activeTab === 'metadata' && !!selectedCommit,
  });

  const acquireLockMutation = useMutation({
    mutationFn: (timeoutHours: number) => acquireLock(namespace, name, { timeout_hours: timeoutHours }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['lock', namespace, name] });
    },
  });

  const releaseLockMutation = useMutation({
    mutationFn: () => releaseLock(namespace, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['lock', namespace, name] });
    },
  });

  const heartbeatMutation = useMutation({
    mutationFn: () => heartbeatLock(namespace, name),
  });

  if (repoLoading) {
    return <Loading message="Loading repository..." />;
  }

  if (repoError) {
    return (
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <ErrorMessage message="Repository not found" />
      </div>
    );
  }

  const tabs: { id: TabType; label: string; icon: any }[] = [
    { id: 'clone', label: 'Clone', icon: Download },
    { id: 'commits', label: 'Commits', icon: GitBranch },
    { id: 'locks', label: 'Locks', icon: Terminal },
    { id: 'metadata', label: 'Metadata', icon: Terminal },
  ];

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <Link
        to="/"
        className="inline-flex items-center space-x-2 text-sm text-gray-600 hover:text-primary-600 mb-6 transition-colors duration-200 hover:translate-x-[-2px] transition-transform"
        aria-label="Back to repositories"
      >
        <ArrowLeft className="w-4 h-4" aria-hidden="true" />
        <span>Back to repositories</span>
      </Link>

      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">
          {namespace}/{name}
        </h1>
        {repo?.description && (
          <p className="mt-2 text-gray-600">{repo.description}</p>
        )}
      </div>

      <div className="border-b border-gray-200 mb-6">
        <nav className="flex space-x-8" role="tablist" aria-label="Repository sections">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                role="tab"
                aria-selected={activeTab === tab.id}
                aria-controls={`${tab.id}-panel`}
                className={`flex items-center space-x-2 py-4 px-1 border-b-2 font-medium text-sm transition-all duration-200 ${
                  activeTab === tab.id
                    ? 'border-primary-600 text-primary-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <Icon className="w-4 h-4" aria-hidden="true" />
                <span>{tab.label}</span>
              </button>
            );
          })}
        </nav>
      </div>

      <div>
        {activeTab === 'clone' && (
          <div role="tabpanel" id="clone-panel" aria-labelledby="clone-tab">
            <CloneInstructions
              namespace={namespace}
              name={name}
              serverUrl={window.location.origin}
            />
          </div>
        )}

        {activeTab === 'commits' && (
          <div role="tabpanel" id="commits-panel" aria-labelledby="commits-tab">
            {commitsLoading && <Loading message="Loading commits..." />}
            {commits && (
              <CommitList
                commits={commits}
                onCommitClick={(commit) => {
                  setSelectedCommit(commit.id);
                  setActiveTab('metadata');
                }}
              />
            )}
          </div>
        )}

        {activeTab === 'locks' && (
          <div role="tabpanel" id="locks-panel" aria-labelledby="locks-tab">
            <LockManager
              lockInfo={lockInfo || null}
              isLoading={lockLoading}
              onAcquire={async (hours) => { await acquireLockMutation.mutateAsync(hours); }}
              onRelease={async () => { await releaseLockMutation.mutateAsync(); }}
              onHeartbeat={async () => { await heartbeatMutation.mutateAsync(); }}
            />
          </div>
        )}

        {activeTab === 'metadata' && (
          <div role="tabpanel" id="metadata-panel" aria-labelledby="metadata-tab">
            {!selectedCommit ? (
              <div className="card">
                <p className="text-sm text-gray-500">
                  Select a commit from the Commits tab to view its metadata
                </p>
              </div>
            ) : (
              <>
                <div className="mb-4">
                  <p className="text-sm text-gray-600">
                    Viewing metadata for commit:{' '}
                    <code className="font-mono bg-gray-100 px-2 py-1 rounded">
                      {selectedCommit.substring(0, 8)}
                    </code>
                  </p>
                </div>
                <MetadataViewer metadata={metadata || null} isLoading={metadataLoading} />
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
