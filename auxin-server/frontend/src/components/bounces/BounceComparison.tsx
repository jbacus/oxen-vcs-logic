import { useState, useEffect } from 'react';
import { ArrowLeftRight, Music, Clock, HardDrive } from 'lucide-react';
import { AudioPlayer } from './AudioPlayer';

interface BounceMetadata {
  commit_id: string;
  original_filename: string;
  format: string;
  size_bytes: number;
  duration_secs?: number;
  sample_rate?: number;
  bit_depth?: number;
  channels?: number;
  added_at: string;
  added_by: string;
}

interface BounceComparisonProps {
  namespace: string;
  repoName: string;
  commitA: string;
  commitB: string;
  onClose?: () => void;
}

export function BounceComparison({ namespace, repoName, commitA, commitB, onClose }: BounceComparisonProps) {
  const [bounceA, setBounceA] = useState<BounceMetadata | null>(null);
  const [bounceB, setBounceB] = useState<BounceMetadata | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchBounces = async () => {
      setLoading(true);
      setError(null);
      try {
        const [responseA, responseB] = await Promise.all([
          fetch(`/api/repos/${namespace}/${repoName}/bounces/${commitA}`),
          fetch(`/api/repos/${namespace}/${repoName}/bounces/${commitB}`),
        ]);

        if (!responseA.ok || !responseB.ok) {
          throw new Error('Failed to fetch one or both bounces');
        }

        const [dataA, dataB] = await Promise.all([
          responseA.json(),
          responseB.json(),
        ]);

        setBounceA(dataA);
        setBounceB(dataB);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };

    fetchBounces();
  }, [namespace, repoName, commitA, commitB]);

  const formatSize = (bytes: number): string => {
    if (bytes >= 1_000_000) {
      return `${(bytes / 1_000_000).toFixed(2)} MB`;
    } else if (bytes >= 1_000) {
      return `${(bytes / 1_000).toFixed(1)} KB`;
    }
    return `${bytes} bytes`;
  };

  const formatDuration = (secs?: number): string => {
    if (!secs) return '--:--';
    const mins = Math.floor(secs / 60);
    const remaining = Math.floor(secs % 60);
    return `${mins}:${remaining.toString().padStart(2, '0')}`;
  };

  const getDurationDiff = (): string | null => {
    if (!bounceA?.duration_secs || !bounceB?.duration_secs) return null;
    const diff = bounceB.duration_secs - bounceA.duration_secs;
    const sign = diff >= 0 ? '+' : '';
    return `${sign}${diff.toFixed(2)}s`;
  };

  const getSizeDiff = (): string => {
    if (!bounceA || !bounceB) return '';
    const diff = bounceB.size_bytes - bounceA.size_bytes;
    const sign = diff >= 0 ? '+' : '';
    if (Math.abs(diff) >= 1_000_000) {
      return `${sign}${(diff / 1_000_000).toFixed(2)} MB`;
    } else if (Math.abs(diff) >= 1_000) {
      return `${sign}${(diff / 1_000).toFixed(1)} KB`;
    }
    return `${sign}${diff} bytes`;
  };

  if (loading) {
    return (
      <div className="p-6 text-center text-gray-500">
        Loading comparison...
      </div>
    );
  }

  if (error || !bounceA || !bounceB) {
    return (
      <div className="p-6 text-center text-red-500">
        {error || 'Could not load bounces for comparison'}
      </div>
    );
  }

  const audioUrlA = `/api/repos/${namespace}/${repoName}/bounces/${commitA}/audio`;
  const audioUrlB = `/api/repos/${namespace}/${repoName}/bounces/${commitB}/audio`;

  return (
    <div className="bg-white rounded-lg shadow-lg p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold flex items-center space-x-2">
          <ArrowLeftRight className="w-5 h-5 text-blue-500" />
          <span>Bounce Comparison</span>
        </h3>
        {onClose && (
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            &times;
          </button>
        )}
      </div>

      <div className="grid grid-cols-2 gap-6">
        {/* Bounce A */}
        <div className="space-y-3">
          <div className="text-sm font-medium text-gray-500">Version A</div>
          <div className="text-sm font-mono bg-gray-100 px-2 py-1 rounded">
            {commitA.substring(0, 8)}
          </div>

          <div className="flex items-center space-x-2 text-sm">
            <Music className="w-4 h-4 text-gray-400" />
            <span>{bounceA.original_filename}</span>
          </div>

          <AudioPlayer
            src={audioUrlA}
            title="Version A"
            duration={bounceA.duration_secs}
          />

          <div className="text-xs text-gray-500 space-y-1">
            <div className="flex items-center space-x-2">
              <Clock className="w-3 h-3" />
              <span>{formatDuration(bounceA.duration_secs)}</span>
            </div>
            <div className="flex items-center space-x-2">
              <HardDrive className="w-3 h-3" />
              <span>{formatSize(bounceA.size_bytes)}</span>
            </div>
            {bounceA.sample_rate && (
              <div>{bounceA.sample_rate} Hz</div>
            )}
          </div>
        </div>

        {/* Bounce B */}
        <div className="space-y-3">
          <div className="text-sm font-medium text-gray-500">Version B</div>
          <div className="text-sm font-mono bg-gray-100 px-2 py-1 rounded">
            {commitB.substring(0, 8)}
          </div>

          <div className="flex items-center space-x-2 text-sm">
            <Music className="w-4 h-4 text-gray-400" />
            <span>{bounceB.original_filename}</span>
          </div>

          <AudioPlayer
            src={audioUrlB}
            title="Version B"
            duration={bounceB.duration_secs}
          />

          <div className="text-xs text-gray-500 space-y-1">
            <div className="flex items-center space-x-2">
              <Clock className="w-3 h-3" />
              <span>{formatDuration(bounceB.duration_secs)}</span>
            </div>
            <div className="flex items-center space-x-2">
              <HardDrive className="w-3 h-3" />
              <span>{formatSize(bounceB.size_bytes)}</span>
            </div>
            {bounceB.sample_rate && (
              <div>{bounceB.sample_rate} Hz</div>
            )}
          </div>
        </div>
      </div>

      {/* Difference summary */}
      <div className="mt-4 pt-4 border-t">
        <div className="text-sm font-medium text-gray-700 mb-2">Differences</div>
        <div className="grid grid-cols-2 gap-4 text-xs">
          <div className="flex justify-between">
            <span className="text-gray-500">Duration:</span>
            <span className={getDurationDiff()?.startsWith('+') ? 'text-green-600' : getDurationDiff()?.startsWith('-') ? 'text-red-600' : ''}>
              {getDurationDiff() || 'N/A'}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-500">Size:</span>
            <span className={getSizeDiff().startsWith('+') ? 'text-green-600' : getSizeDiff().startsWith('-') ? 'text-red-600' : ''}>
              {getSizeDiff()}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
