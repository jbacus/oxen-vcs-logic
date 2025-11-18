import { useState, useRef, useEffect } from 'react';
import { Play, Pause, Volume2, VolumeX, Music } from 'lucide-react';

interface AudioPlayerProps {
  src: string;
  title?: string;
  duration?: number;
  onPlayStateChange?: (isPlaying: boolean) => void;
}

export function AudioPlayer({ src, title, duration, onPlayStateChange }: AudioPlayerProps) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [totalDuration, setTotalDuration] = useState(duration || 0);
  const [isMuted, setIsMuted] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const handleTimeUpdate = () => setCurrentTime(audio.currentTime);
    const handleLoadedMetadata = () => {
      setTotalDuration(audio.duration);
      setIsLoading(false);
    };
    const handleEnded = () => {
      setIsPlaying(false);
      setCurrentTime(0);
      onPlayStateChange?.(false);
    };
    const handleError = () => {
      setError('Failed to load audio');
      setIsLoading(false);
    };
    const handleWaiting = () => setIsLoading(true);
    const handleCanPlay = () => setIsLoading(false);

    audio.addEventListener('timeupdate', handleTimeUpdate);
    audio.addEventListener('loadedmetadata', handleLoadedMetadata);
    audio.addEventListener('ended', handleEnded);
    audio.addEventListener('error', handleError);
    audio.addEventListener('waiting', handleWaiting);
    audio.addEventListener('canplay', handleCanPlay);

    return () => {
      audio.removeEventListener('timeupdate', handleTimeUpdate);
      audio.removeEventListener('loadedmetadata', handleLoadedMetadata);
      audio.removeEventListener('ended', handleEnded);
      audio.removeEventListener('error', handleError);
      audio.removeEventListener('waiting', handleWaiting);
      audio.removeEventListener('canplay', handleCanPlay);
    };
  }, [onPlayStateChange]);

  const togglePlay = async () => {
    const audio = audioRef.current;
    if (!audio) return;

    try {
      if (isPlaying) {
        audio.pause();
        setIsPlaying(false);
        onPlayStateChange?.(false);
      } else {
        setIsLoading(true);
        await audio.play();
        setIsPlaying(true);
        onPlayStateChange?.(true);
      }
    } catch (err) {
      setError('Playback failed');
      setIsLoading(false);
    }
  };

  const toggleMute = () => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.muted = !isMuted;
    setIsMuted(!isMuted);
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const audio = audioRef.current;
    if (!audio) return;
    const time = parseFloat(e.target.value);
    audio.currentTime = time;
    setCurrentTime(time);
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const progress = totalDuration > 0 ? (currentTime / totalDuration) * 100 : 0;

  if (error) {
    return (
      <div className="flex items-center space-x-2 text-red-500 text-sm">
        <Music className="w-4 h-4" />
        <span>{error}</span>
      </div>
    );
  }

  return (
    <div className="bg-gray-100 rounded-lg p-3">
      <audio ref={audioRef} src={src} preload="metadata" />

      {title && (
        <div className="text-xs text-gray-600 mb-2 truncate flex items-center space-x-1">
          <Music className="w-3 h-3" />
          <span>{title}</span>
        </div>
      )}

      <div className="flex items-center space-x-3">
        <button
          onClick={togglePlay}
          disabled={isLoading}
          className="flex-shrink-0 w-8 h-8 rounded-full bg-blue-500 text-white flex items-center justify-center hover:bg-blue-600 transition-colors disabled:opacity-50"
        >
          {isLoading ? (
            <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
          ) : isPlaying ? (
            <Pause className="w-4 h-4" />
          ) : (
            <Play className="w-4 h-4 ml-0.5" />
          )}
        </button>

        <div className="flex-1 min-w-0">
          <input
            type="range"
            min={0}
            max={totalDuration || 100}
            value={currentTime}
            onChange={handleSeek}
            className="w-full h-1 bg-gray-300 rounded-lg appearance-none cursor-pointer"
            style={{
              background: `linear-gradient(to right, #3b82f6 0%, #3b82f6 ${progress}%, #d1d5db ${progress}%, #d1d5db 100%)`
            }}
          />
          <div className="flex justify-between text-xs text-gray-500 mt-1">
            <span>{formatTime(currentTime)}</span>
            <span>{formatTime(totalDuration)}</span>
          </div>
        </div>

        <button
          onClick={toggleMute}
          className="flex-shrink-0 p-1.5 rounded hover:bg-gray-200 transition-colors"
        >
          {isMuted ? (
            <VolumeX className="w-4 h-4 text-gray-500" />
          ) : (
            <Volume2 className="w-4 h-4 text-gray-500" />
          )}
        </button>
      </div>
    </div>
  );
}
