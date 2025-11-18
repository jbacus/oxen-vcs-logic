import { useState, useEffect } from 'react';
import { Lock, Unlock, Heart, Clock, User, Monitor } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import type { LockInfo } from '@/types';

interface LockManagerProps {
  namespace: string;
  name: string;
  lockInfo: LockInfo | null;
  isLoading: boolean;
  onAcquire: (timeoutHours: number) => Promise<void>;
  onRelease: () => Promise<void>;
  onHeartbeat: () => Promise<void>;
}

export function LockManager({
  namespace,
  name,
  lockInfo,
  isLoading,
  onAcquire,
  onRelease,
  onHeartbeat,
}: LockManagerProps) {
  const [timeoutHours, setTimeoutHours] = useState(24);
  const [isProcessing, setIsProcessing] = useState(false);

  // Auto-heartbeat every 5 minutes if locked
  useEffect(() => {
    if (!lockInfo?.locked) return;

    const interval = setInterval(() => {
      onHeartbeat();
    }, 5 * 60 * 1000); // 5 minutes

    return () => clearInterval(interval);
  }, [lockInfo?.locked, onHeartbeat]);

  const handleAcquire = async () => {
    setIsProcessing(true);
    try {
      await onAcquire(timeoutHours);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleRelease = async () => {
    setIsProcessing(true);
    try {
      await onRelease();
    } finally {
      setIsProcessing(false);
    }
  };

  if (isLoading) {
    return (
      <div className="card">
        <p className="text-sm text-gray-500">Loading lock status...</p>
      </div>
    );
  }

  const isLocked = lockInfo?.locked ?? false;

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900 flex items-center space-x-2">
          {isLocked ? (
            <>
              <Lock className="w-5 h-5 text-red-600" />
              <span>Project Locked</span>
            </>
          ) : (
            <>
              <Unlock className="w-5 h-5 text-green-600" />
              <span>Project Available</span>
            </>
          )}
        </h3>
        <div className={`badge ${isLocked ? 'badge-red' : 'badge-green'}`}>
          {isLocked ? 'Locked' : 'Available'}
        </div>
      </div>

      {isLocked && lockInfo?.holder && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-4">
          <div className="space-y-2 text-sm">
            <div className="flex items-center space-x-2 text-gray-700">
              <User className="w-4 h-4 text-red-600" />
              <span className="font-medium">Locked by:</span>
              <span>{lockInfo.holder.user}</span>
            </div>
            <div className="flex items-center space-x-2 text-gray-700">
              <Monitor className="w-4 h-4 text-red-600" />
              <span className="font-medium">Machine:</span>
              <span className="font-mono text-xs">{lockInfo.holder.machine_id}</span>
            </div>
            <div className="flex items-center space-x-2 text-gray-700">
              <Clock className="w-4 h-4 text-red-600" />
              <span className="font-medium">Acquired:</span>
              <span>
                {formatDistanceToNow(new Date(lockInfo.holder.acquired_at), { addSuffix: true })}
              </span>
            </div>
            <div className="flex items-center space-x-2 text-gray-700">
              <Clock className="w-4 h-4 text-red-600" />
              <span className="font-medium">Expires:</span>
              <span>
                {formatDistanceToNow(new Date(lockInfo.holder.expires_at), { addSuffix: true })}
              </span>
            </div>
          </div>
        </div>
      )}

      {!isLocked && (
        <div className="mb-4">
          <label htmlFor="timeout" className="block text-sm font-medium text-gray-700 mb-2">
            Lock Timeout (hours)
          </label>
          <input
            id="timeout"
            type="number"
            min="1"
            max="72"
            value={timeoutHours}
            onChange={(e) => setTimeoutHours(parseInt(e.target.value))}
            className="input w-32"
          />
          <p className="mt-1 text-xs text-gray-500">
            Lock will automatically expire after this duration
          </p>
        </div>
      )}

      <div className="flex items-center space-x-3">
        {isLocked ? (
          <>
            <button
              onClick={handleRelease}
              disabled={isProcessing}
              className="btn-danger flex items-center space-x-2"
            >
              <Unlock className="w-4 h-4" />
              <span>{isProcessing ? 'Releasing...' : 'Release Lock'}</span>
            </button>
            <button
              onClick={onHeartbeat}
              className="btn-secondary flex items-center space-x-2"
              title="Send heartbeat to extend lock"
            >
              <Heart className="w-4 h-4" />
              <span>Heartbeat</span>
            </button>
          </>
        ) : (
          <button
            onClick={handleAcquire}
            disabled={isProcessing}
            className="btn-primary flex items-center space-x-2"
          >
            <Lock className="w-4 h-4" />
            <span>{isProcessing ? 'Acquiring...' : 'Acquire Lock'}</span>
          </button>
        )}
      </div>

      <div className="mt-4 p-3 bg-gray-50 rounded-lg">
        <p className="text-xs text-gray-600">
          <strong>Note:</strong> Acquiring a lock prevents other users from editing this project.
          Release the lock when you're done to allow others to work.
        </p>
      </div>
    </div>
  );
}
