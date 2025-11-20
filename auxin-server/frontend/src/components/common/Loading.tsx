import { Loader2 } from 'lucide-react';

interface LoadingProps {
  message?: string;
}

export function Loading({ message = 'Loading...' }: LoadingProps) {
  return (
    <div className="flex flex-col items-center justify-center py-12" role="status" aria-live="polite">
      <Loader2 className="w-8 h-8 text-primary-600 animate-spin" aria-hidden="true" />
      <p className="mt-4 text-sm text-gray-600 animate-pulse">{message}</p>
      <span className="sr-only">{message}</span>
    </div>
  );
}
