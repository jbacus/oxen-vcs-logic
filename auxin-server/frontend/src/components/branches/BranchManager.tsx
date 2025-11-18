import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getBranches, createBranch } from '@/services/api';
import { GitBranch, Plus, Check } from 'lucide-react';
import toast from 'react-hot-toast';
import { handleApiError } from '@/utils/errors';
import type { Branch } from '@/types';

interface BranchManagerProps {
  namespace: string;
  name: string;
  currentBranch: string;
}

export function BranchManager({ namespace, name, currentBranch }: BranchManagerProps) {
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newBranchName, setNewBranchName] = useState('');
  const queryClient = useQueryClient();

  const { data: branches = [], isLoading } = useQuery({
    queryKey: ['branches', namespace, name],
    queryFn: () => getBranches(namespace, name).then((res) => res.data),
  });

  const createBranchMutation = useMutation({
    mutationFn: (branchName: string) => createBranch(namespace, name, branchName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['branches', namespace, name] });
      toast.success(`Branch "${newBranchName}" created`);
      setNewBranchName('');
      setShowCreateForm(false);
    },
    onError: (error) => {
      toast.error(handleApiError(error));
    },
  });

  const handleCreateBranch = (e: React.FormEvent) => {
    e.preventDefault();

    // Validate branch name
    const validNameRegex = /^[a-zA-Z0-9_-]+$/;
    if (!validNameRegex.test(newBranchName)) {
      toast.error('Branch name can only contain letters, numbers, hyphens, and underscores');
      return;
    }

    if (newBranchName.length < 1 || newBranchName.length > 100) {
      toast.error('Branch name must be between 1 and 100 characters');
      return;
    }

    createBranchMutation.mutate(newBranchName);
  };

  if (isLoading) {
    return (
      <div className="animate-pulse space-y-3">
        {[1, 2, 3].map((i) => (
          <div key={i} className="h-12 bg-gray-200 dark:bg-gray-700 rounded" />
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-medium text-gray-900 dark:text-white">Branches</h3>
        <button
          onClick={() => setShowCreateForm(!showCreateForm)}
          className="btn-primary flex items-center space-x-2 text-sm"
        >
          <Plus className="w-4 h-4" />
          <span>New Branch</span>
        </button>
      </div>

      {showCreateForm && (
        <form onSubmit={handleCreateBranch} className="flex items-center space-x-2">
          <input
            type="text"
            value={newBranchName}
            onChange={(e) => setNewBranchName(e.target.value)}
            placeholder="Branch name"
            className="input flex-1"
            autoFocus
          />
          <button
            type="submit"
            disabled={createBranchMutation.isPending || !newBranchName}
            className="btn-primary"
          >
            {createBranchMutation.isPending ? 'Creating...' : 'Create'}
          </button>
          <button
            type="button"
            onClick={() => {
              setShowCreateForm(false);
              setNewBranchName('');
            }}
            className="btn-secondary"
          >
            Cancel
          </button>
        </form>
      )}

      <div className="space-y-2">
        {branches.map((branch: Branch) => (
          <div
            key={branch.name}
            className={`flex items-center justify-between p-3 rounded-lg border ${
              branch.name === currentBranch
                ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800'
            }`}
          >
            <div className="flex items-center space-x-3">
              <GitBranch className={`w-4 h-4 ${
                branch.name === currentBranch
                  ? 'text-primary-600 dark:text-primary-400'
                  : 'text-gray-400'
              }`} />
              <span className={`font-medium ${
                branch.name === currentBranch
                  ? 'text-primary-700 dark:text-primary-300'
                  : 'text-gray-900 dark:text-white'
              }`}>
                {branch.name}
              </span>
              {branch.name === currentBranch && (
                <span className="flex items-center space-x-1 text-xs text-primary-600 dark:text-primary-400">
                  <Check className="w-3 h-3" />
                  <span>Current</span>
                </span>
              )}
            </div>
            <span className="text-xs text-gray-500 dark:text-gray-400 font-mono">
              {branch.commit_id?.substring(0, 8) || 'N/A'}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
