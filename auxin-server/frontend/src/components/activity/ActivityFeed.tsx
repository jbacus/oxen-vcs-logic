import { useQuery } from '@tanstack/react-query';
import { getActivity } from '@/services/api';
import { GitCommit, Lock, Unlock, GitBranch, UserPlus, RefreshCw } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

interface ActivityFeedProps {
  namespace: string;
  name: string;
  limit?: number;
}

interface Activity {
  id: string;
  type: 'commit' | 'lock_acquired' | 'lock_released' | 'branch_created' | 'user_joined';
  user: string;
  description: string;
  timestamp: string;
  metadata?: Record<string, unknown>;
}

function getActivityIcon(type: Activity['type']) {
  switch (type) {
    case 'commit':
      return <GitCommit className="w-4 h-4 text-green-500" />;
    case 'lock_acquired':
      return <Lock className="w-4 h-4 text-orange-500" />;
    case 'lock_released':
      return <Unlock className="w-4 h-4 text-blue-500" />;
    case 'branch_created':
      return <GitBranch className="w-4 h-4 text-purple-500" />;
    case 'user_joined':
      return <UserPlus className="w-4 h-4 text-teal-500" />;
    default:
      return <RefreshCw className="w-4 h-4 text-gray-500" />;
  }
}

function formatTimestamp(timestamp: string): string {
  try {
    return formatDistanceToNow(new Date(timestamp), { addSuffix: true });
  } catch {
    return timestamp;
  }
}

export function ActivityFeed({ namespace, name, limit = 20 }: ActivityFeedProps) {
  const { data: activities = [], isLoading, error, refetch } = useQuery({
    queryKey: ['activity', namespace, name, limit],
    queryFn: () => getActivity(namespace, name, limit).then((res) => res.data),
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  if (isLoading) {
    return (
      <div className="animate-pulse space-y-3">
        {[1, 2, 3, 4, 5].map((i) => (
          <div key={i} className="flex items-start space-x-3">
            <div className="w-8 h-8 bg-gray-200 dark:bg-gray-700 rounded-full" />
            <div className="flex-1 space-y-2">
              <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-3/4" />
              <div className="h-3 bg-gray-200 dark:bg-gray-700 rounded w-1/4" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-8 text-gray-500 dark:text-gray-400">
        Failed to load activity feed
      </div>
    );
  }

  if (activities.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500 dark:text-gray-400">
        No recent activity
      </div>
    );
  }

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-medium text-gray-900 dark:text-white">Recent Activity</h3>
        <button
          onClick={() => refetch()}
          className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
          title="Refresh"
        >
          <RefreshCw className="w-4 h-4 text-gray-500" />
        </button>
      </div>

      <div className="space-y-4">
        {activities.map((activity: Activity) => (
          <div key={activity.id} className="flex items-start space-x-3">
            <div className="flex-shrink-0 mt-0.5">
              <div className="w-8 h-8 rounded-full bg-gray-100 dark:bg-gray-700 flex items-center justify-center">
                {getActivityIcon(activity.type)}
              </div>
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm text-gray-900 dark:text-white">
                <span className="font-medium">{activity.user}</span>{' '}
                <span className="text-gray-600 dark:text-gray-300">{activity.description}</span>
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                {formatTimestamp(activity.timestamp)}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
