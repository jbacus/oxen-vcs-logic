import { useState, useEffect } from 'react';
import { Music, Download, Info } from 'lucide-react';
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
  description?: string;
}

interface BouncePreviewProps {
  namespace: string;
  repoName: string;
  commitId: string;
  compact?: boolean;
}

export function BouncePreview({ namespace, repoName, commitId, compact = false }: BouncePreviewProps) {
  const [bounce, setBounce] = useState<BounceMetadata | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showDetails, setShowDetails] = useState(false);

  useEffect(() => {
    const fetchBounce = async () => {
      setLoading(true);
      setError(null);
      try {
        const response = await fetch(
          `/api/repos/${namespace}/${repoName}/bounces/${commitId}`
        );
        if (response.status === 404) {
          // No bounce for this commit
          setBounce(null);
        } else if (!response.ok) {
          throw new Error('Failed to fetch bounce');
        } else {
          const data = await response.json();
          setBounce(data);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };

    fetchBounce();
  }, [namespace, repoName, commitId]);

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

  if (loading) {
    return compact ? null : (
      <div className="text-xs text-gray-400">Loading bounce...</div>
    );
  }

  if (error || !bounce) {
    return null;
  }

  const audioUrl = `/api/repos/${namespace}/${repoName}/bounces/${commitId}/audio`;

  if (compact) {
    return (
      <div className="mt-2">
        <AudioPlayer
          src={audioUrl}
          title={bounce.original_filename}
          duration={bounce.duration_secs}
        />
      </div>
    );
  }

  return (
    <div className="mt-3 border-t pt-3">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center space-x-2 text-sm text-gray-600">
          <Music className="w-4 h-4" />
          <span className="font-medium">Audio Bounce</span>
        </div>
        <div className="flex items-center space-x-1">
          <button
            onClick={() => setShowDetails(!showDetails)}
            className="p-1 rounded hover:bg-gray-100 transition-colors"
            title="Toggle details"
          >
            <Info className="w-4 h-4 text-gray-400" />
          </button>
          <a
            href={audioUrl}
            download={bounce.original_filename}
            className="p-1 rounded hover:bg-gray-100 transition-colors"
            title="Download"
          >
            <Download className="w-4 h-4 text-gray-400" />
          </a>
        </div>
      </div>

      <AudioPlayer
        src={audioUrl}
        title={bounce.original_filename}
        duration={bounce.duration_secs}
      />

      {showDetails && (
        <div className="mt-2 text-xs text-gray-500 space-y-1">
          <div className="grid grid-cols-2 gap-x-4">
            <span>Format:</span>
            <span className="font-mono">{bounce.format.toUpperCase()}</span>
            <span>Size:</span>
            <span>{formatSize(bounce.size_bytes)}</span>
            <span>Duration:</span>
            <span>{formatDuration(bounce.duration_secs)}</span>
            {bounce.sample_rate && (
              <>
                <span>Sample Rate:</span>
                <span>{bounce.sample_rate} Hz</span>
              </>
            )}
            {bounce.bit_depth && (
              <>
                <span>Bit Depth:</span>
                <span>{bounce.bit_depth}-bit</span>
              </>
            )}
            {bounce.channels && (
              <>
                <span>Channels:</span>
                <span>{bounce.channels === 1 ? 'Mono' : bounce.channels === 2 ? 'Stereo' : bounce.channels}</span>
              </>
            )}
          </div>
          {bounce.description && (
            <p className="mt-2 italic">{bounce.description}</p>
          )}
        </div>
      )}
    </div>
  );
}
