import { Music, Gauge, Disc, Tag } from 'lucide-react';
import type { LogicProMetadata } from '@/types';

interface MetadataViewerProps {
  metadata: LogicProMetadata | null;
  isLoading?: boolean;
}

export function MetadataViewer({ metadata, isLoading }: MetadataViewerProps) {
  if (isLoading) {
    return (
      <div className="card">
        <p className="text-sm text-gray-500">Loading metadata...</p>
      </div>
    );
  }

  if (!metadata) {
    return (
      <div className="card">
        <p className="text-sm text-gray-500">No Logic Pro metadata available for this commit</p>
      </div>
    );
  }

  return (
    <div className="card">
      <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center space-x-2">
        <Music className="w-5 h-5 text-primary-600" />
        <span>Logic Pro Metadata</span>
      </h3>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {metadata.bpm !== undefined && (
          <div className="flex items-center space-x-3 p-3 bg-blue-50 rounded-lg">
            <div className="bg-blue-100 p-2 rounded">
              <Gauge className="w-5 h-5 text-blue-600" />
            </div>
            <div>
              <p className="text-xs text-blue-600 font-medium">BPM</p>
              <p className="text-lg font-semibold text-blue-900">{metadata.bpm}</p>
            </div>
          </div>
        )}

        {metadata.sample_rate !== undefined && (
          <div className="flex items-center space-x-3 p-3 bg-green-50 rounded-lg">
            <div className="bg-green-100 p-2 rounded">
              <Disc className="w-5 h-5 text-green-600" />
            </div>
            <div>
              <p className="text-xs text-green-600 font-medium">Sample Rate</p>
              <p className="text-lg font-semibold text-green-900">
                {metadata.sample_rate / 1000}kHz
              </p>
            </div>
          </div>
        )}

        {metadata.key_signature && (
          <div className="flex items-center space-x-3 p-3 bg-purple-50 rounded-lg">
            <div className="bg-purple-100 p-2 rounded">
              <Music className="w-5 h-5 text-purple-600" />
            </div>
            <div>
              <p className="text-xs text-purple-600 font-medium">Key Signature</p>
              <p className="text-lg font-semibold text-purple-900">{metadata.key_signature}</p>
            </div>
          </div>
        )}

        {metadata.tags && metadata.tags.length > 0 && (
          <div className="col-span-full">
            <div className="flex items-center space-x-2 mb-2">
              <Tag className="w-4 h-4 text-gray-600" />
              <p className="text-sm font-medium text-gray-700">Tags</p>
            </div>
            <div className="flex flex-wrap gap-2">
              {metadata.tags.map((tag) => (
                <span key={tag} className="badge-gray">
                  {tag}
                </span>
              ))}
            </div>
          </div>
        )}

        {metadata.custom && Object.keys(metadata.custom).length > 0 && (
          <div className="col-span-full">
            <p className="text-sm font-medium text-gray-700 mb-2">Custom Metadata</p>
            <pre className="bg-gray-50 p-3 rounded text-xs overflow-x-auto">
              {JSON.stringify(metadata.custom, null, 2)}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
}
