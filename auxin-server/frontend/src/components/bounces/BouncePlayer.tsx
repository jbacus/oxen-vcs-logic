import { useQuery } from '@tanstack/react-query';
import { Music, Play, Pause, Download, Clock, User, Calendar } from 'lucide-react';
import { useState, useRef, useEffect } from 'react';
import { listBounces, getBounceAudioUrl } from '@/services/api';
import type { BounceMetadata } from '@/types';
import { Loading } from '@/components/common/Loading';
import { ErrorMessage } from '@/components/common/ErrorMessage';
import { EmptyState } from '@/components/common/EmptyState';

interface BouncePlayerProps {
  namespace: string;
  name: string;
}

export function BouncePlayer({ namespace, name }: BouncePlayerProps) {
  const [currentPlaying, setCurrentPlaying] = useState<string | null>(null);
  const [currentTime, setCurrentTime] = useState<number>(0);
  const [duration, setDuration] = useState<number>(0);
  const audioRef = useRef<HTMLAudioElement>(null);

  const { data: bounces, isLoading, error } = useQuery({
    queryKey: ['bounces', namespace, name],
    queryFn: async () => {
      const response = await listBounces(namespace, name);
      return response.data;
    },
  });

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const handleTimeUpdate = () => setCurrentTime(audio.currentTime);
    const handleDurationChange = () => setDuration(audio.duration);
    const handleEnded = () => setCurrentPlaying(null);

    audio.addEventListener('timeupdate', handleTimeUpdate);
    audio.addEventListener('durationchange', handleDurationChange);
    audio.addEventListener('ended', handleEnded);

    return () => {
      audio.removeEventListener('timeupdate', handleTimeUpdate);
      audio.removeEventListener('durationchange', handleDurationChange);
      audio.removeEventListener('ended', handleEnded);
    };
  }, []);

  const togglePlay = (commitId: string, audioUrl: string) => {
    const audio = audioRef.current;
    if (!audio) return;

    if (currentPlaying === commitId) {
      // Pause current
      audio.pause();
      setCurrentPlaying(null);
    } else {
      // Play new
      audio.src = audioUrl;
      audio.play();
      setCurrentPlaying(commitId);
    }
  };

  const formatTime = (seconds: number) => {
    if (!isFinite(seconds)) return '--:--';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatFileSize = (bytes: number) => {
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(2)} MB`;
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  if (isLoading) {
    return <Loading message="Loading bounces..." />;
  }

  if (error) {
    return <ErrorMessage message="Failed to load bounces" />;
  }

  if (!bounces || bounces.length === 0) {
    return (
      <EmptyState
        icon={Music}
        title="No bounces yet"
        description="Audio bounces will appear here once uploaded by producers."
      />
    );
  }

  return (
    <div className="space-y-4">
      {/* Hidden audio element */}
      <audio ref={audioRef} />

      {/* Bounces list */}
      <div className="space-y-3">
        {bounces.map((bounce: BounceMetadata) => {
          const audioUrl = getBounceAudioUrl(namespace, name, bounce.commit_id);
          const isPlaying = currentPlaying === bounce.commit_id;

          return (
            <div
              key={bounce.commit_id}
              className="card hover:shadow-md transition-shadow duration-200"
            >
              <div className="flex items-start gap-4">
                {/* Play button */}
                <button
                  onClick={() => togglePlay(bounce.commit_id, audioUrl)}
                  className={`flex-shrink-0 w-12 h-12 rounded-full flex items-center justify-center transition-colors duration-200 ${
                    isPlaying
                      ? 'bg-primary-600 text-white'
                      : 'bg-primary-100 text-primary-600 hover:bg-primary-200'
                  }`}
                  aria-label={isPlaying ? 'Pause' : 'Play'}
                >
                  {isPlaying ? (
                    <Pause className="w-5 h-5" aria-hidden="true" />
                  ) : (
                    <Play className="w-5 h-5 ml-0.5" aria-hidden="true" />
                  )}
                </button>

                {/* Bounce info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-start justify-between gap-4 mb-2">
                    <div className="flex-1 min-w-0">
                      <h3 className="text-base font-semibold text-gray-900 truncate">
                        {bounce.original_filename}
                      </h3>
                      {bounce.description && (
                        <p className="text-sm text-gray-600 mt-1">
                          {bounce.description}
                        </p>
                      )}
                    </div>
                    <a
                      href={audioUrl}
                      download={bounce.original_filename}
                      className="flex-shrink-0 p-2 text-gray-400 hover:text-primary-600 rounded-lg hover:bg-gray-100 transition-colors duration-200"
                      aria-label="Download bounce"
                    >
                      <Download className="w-5 h-5" aria-hidden="true" />
                    </a>
                  </div>

                  {/* Progress bar (only show when playing) */}
                  {isPlaying && duration > 0 && (
                    <div className="mb-2">
                      <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
                        <span>{formatTime(currentTime)}</span>
                        <span>{formatTime(duration)}</span>
                      </div>
                      <div className="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-primary-600 transition-all duration-100"
                          style={{ width: `${(currentTime / duration) * 100}%` }}
                        />
                      </div>
                    </div>
                  )}

                  {/* Metadata */}
                  <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-gray-500">
                    <span className="flex items-center gap-1">
                      <Calendar className="w-3.5 h-3.5" aria-hidden="true" />
                      {formatDate(bounce.added_at)}
                    </span>
                    <span className="flex items-center gap-1">
                      <User className="w-3.5 h-3.5" aria-hidden="true" />
                      {bounce.added_by}
                    </span>
                    <span className="flex items-center gap-1 uppercase">
                      <Music className="w-3.5 h-3.5" aria-hidden="true" />
                      {bounce.format}
                    </span>
                    <span className="flex items-center gap-1">
                      {formatFileSize(bounce.size_bytes)}
                    </span>
                    {bounce.sample_rate && (
                      <span>{(bounce.sample_rate / 1000).toFixed(1)}kHz</span>
                    )}
                    {bounce.bit_depth && <span>{bounce.bit_depth}-bit</span>}
                    {bounce.channels && (
                      <span>
                        {bounce.channels === 1
                          ? 'Mono'
                          : bounce.channels === 2
                          ? 'Stereo'
                          : `${bounce.channels}ch`}
                      </span>
                    )}
                  </div>

                  {/* Commit ID */}
                  <div className="mt-2 text-xs text-gray-400 font-mono">
                    Commit: {bounce.commit_id.substring(0, 8)}
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* Info footer */}
      <div className="card bg-blue-50 border-blue-200">
        <div className="flex items-start gap-3">
          <div className="flex-shrink-0">
            <div className="w-8 h-8 rounded-full bg-blue-100 flex items-center justify-center">
              <Music className="w-4 h-4 text-blue-600" aria-hidden="true" />
            </div>
          </div>
          <div className="flex-1">
            <p className="text-sm text-blue-900">
              <strong>About Bounces:</strong> These are audio exports of project commits.
              Click play to listen or download to save locally.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
