import { useState } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ArrowLeft, GitBranch, Terminal, Download, Trash2, AlertTriangle, FolderTree, Lock } from 'lucide-react';
import { CommitList } from '@/components/commits/CommitList';
import { MetadataViewer } from '@/components/metadata/MetadataViewer';
import { LockManager } from '@/components/locks/LockManager';
import { CloneInstructions } from '@/components/repos/CloneInstructions';
import { ActivityFeed } from '@/components/activity/ActivityFeed';
import { FileBrowser } from '@/components/files/FileBrowser';
import { BranchManager } from '@/components/branches/BranchManager';
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
  deleteRepository,
} from '@/services/api';

type TabType = 'commits' | 'locks' | 'metadata' | 'clone' | 'files' | 'branches';

export function RepoPage() {
  const { namespace, name } = useParams<{ namespace: string; name: string }>();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<TabType>('clone');
  const [selectedCommit, setSelectedCommit] = useState<string | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
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

  const deleteMutation = useMutation({
    mutationFn: () => deleteRepository(namespace, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      navigate('/');
    },
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
    { id: 'branches', label: 'Branches', icon: GitBranch },
    { id: 'files', label: 'Files', icon: FolderTree },
    { id: 'locks', label: 'Locks', icon: Lock },
    { id: 'metadata', label: 'Metadata', icon: Terminal },
  ];

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <Link
        to="/"
        className="inline-flex items-center space-x-2 text-sm text-gray-600 hover:text-primary-600 mb-4 transition-colors duration-200 hover:translate-x-[-2px] transition-transform"
        aria-label="Back to repositories"
      >
        <ArrowLeft className="w-4 h-4" aria-hidden="true" />
        <span>Back to repositories</span>
      </Link>

      <div className="mb-6">
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <h1 className="text-3xl font-bold text-gray-900">
              {namespace}/{name}
            </h1>
            {repo?.description && (
              <p className="mt-2 text-gray-600">{repo.description}</p>
            )}
          </div>
          <button
            onClick={() => setShowDeleteConfirm(true)}
            className="ml-4 px-4 py-2 text-sm font-medium text-red-700 bg-red-50 border border-red-200 rounded-lg hover:bg-red-100 hover:border-red-300 transition-colors duration-200 flex items-center space-x-2"
            aria-label="Delete repository"
          >
            <Trash2 className="w-4 h-4" aria-hidden="true" />
            <span>Delete</span>
          </button>
        </div>
      </div>

      <div className="border-b border-gray-200 mb-4">
        <nav className="flex space-x-6" role="tablist" aria-label="Repository sections">
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

        {activeTab === 'branches' && (
          <div role="tabpanel" id="branches-panel" aria-labelledby="branches-tab">
            <BranchManager namespace={namespace} name={name} currentBranch="main" />
          </div>
        )}

        {activeTab === 'files' && (
          <div role="tabpanel" id="files-panel" aria-labelledby="files-tab">
            {selectedCommit ? (
              <FileBrowser
                namespace={namespace}
                name={name}
                commit={selectedCommit}
              />
            ) : (
              <div className="card">
                <p className="text-sm text-gray-500">
                  Select a commit from the Commits tab to browse files
                </p>
              </div>
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

      {/* Delete Confirmation Modal */}
      {showDeleteConfirm && (
        <div
          className="fixed inset-0 bg-black bg-opacity-50 backdrop-blur-sm flex items-center justify-center z-50 animate-in fade-in duration-200"
          onClick={() => setShowDeleteConfirm(false)}
          role="dialog"
          aria-modal="true"
          aria-labelledby="delete-dialog-title"
        >
          <div
            className="bg-white rounded-lg shadow-xl max-w-md w-full mx-4 animate-in zoom-in-95 duration-200"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="p-6">
              <div className="flex items-start space-x-4">
                <div className="flex-shrink-0">
                  <div className="flex items-center justify-center w-12 h-12 rounded-full bg-red-100">
                    <AlertTriangle className="w-6 h-6 text-red-600" aria-hidden="true" />
                  </div>
                </div>
                <div className="flex-1">
                  <h3 id="delete-dialog-title" className="text-lg font-semibold text-gray-900 mb-2">
                    Delete Repository
                  </h3>
                  <p className="text-sm text-gray-600 mb-4">
                    Are you sure you want to delete <strong>{namespace}/{name}</strong>?
                    This action cannot be undone and will permanently delete all commits, branches, and history.
                  </p>
                  <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-3 mb-4">
                    <p className="text-xs text-yellow-800">
                      <strong>Warning:</strong> This will delete the repository from the server. Any local clones will not be affected.
                    </p>
                  </div>
                </div>
              </div>

              <div className="flex justify-end space-x-3 mt-6">
                <button
                  onClick={() => setShowDeleteConfirm(false)}
                  className="btn-secondary"
                  disabled={deleteMutation.isPending}
                >
                  Cancel
                </button>
                <button
                  onClick={async () => {
                    await deleteMutation.mutateAsync();
                  }}
                  className="px-4 py-2 text-sm font-medium text-white bg-red-600 border border-transparent rounded-lg hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 min-w-[100px]"
                  disabled={deleteMutation.isPending}
                >
                  {deleteMutation.isPending ? (
                    <span className="flex items-center justify-center">
                      <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      Deleting...
                    </span>
                  ) : (
                    'Delete Repository'
                  )}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
        </div>

        {/* Activity Feed Sidebar */}
        <div className="lg:col-span-1">
          <div className="sticky top-6">
            <ActivityFeed namespace={namespace} name={name} limit={20} />
          </div>
        </div>
      </div>
    </div>
  );
}
