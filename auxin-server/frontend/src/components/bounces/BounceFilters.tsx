import { useState } from 'react';
import { Search, Filter, X } from 'lucide-react';

export interface BounceFilterValues {
  format?: string;
  pattern?: string;
  minDuration?: number;
  maxDuration?: number;
  minSize?: number;
  maxSize?: number;
  user?: string;
}

interface BounceFiltersProps {
  onFilter: (filters: BounceFilterValues) => void;
  onClear: () => void;
}

export function BounceFilters({ onFilter, onClear }: BounceFiltersProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [filters, setFilters] = useState<BounceFilterValues>({});

  const handleChange = (key: keyof BounceFilterValues, value: string | number | undefined) => {
    const newFilters = { ...filters, [key]: value || undefined };
    setFilters(newFilters);
  };

  const handleApply = () => {
    onFilter(filters);
  };

  const handleClear = () => {
    setFilters({});
    onClear();
  };

  const hasFilters = Object.values(filters).some(v => v !== undefined && v !== '');

  return (
    <div className="mb-4">
      <div className="flex items-center space-x-2">
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className={`flex items-center space-x-2 px-3 py-2 rounded-lg border transition-colors ${
            isExpanded || hasFilters
              ? 'bg-blue-50 border-blue-200 text-blue-700'
              : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50'
          }`}
        >
          <Filter className="w-4 h-4" />
          <span>Filter</span>
          {hasFilters && (
            <span className="bg-blue-500 text-white text-xs px-1.5 py-0.5 rounded-full">
              {Object.values(filters).filter(v => v !== undefined && v !== '').length}
            </span>
          )}
        </button>

        {hasFilters && (
          <button
            onClick={handleClear}
            className="flex items-center space-x-1 px-2 py-1 text-sm text-gray-500 hover:text-gray-700"
          >
            <X className="w-3 h-3" />
            <span>Clear</span>
          </button>
        )}
      </div>

      {isExpanded && (
        <div className="mt-3 p-4 bg-gray-50 rounded-lg border">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {/* Format filter */}
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Format
              </label>
              <select
                value={filters.format || ''}
                onChange={(e) => handleChange('format', e.target.value)}
                className="w-full px-2 py-1.5 text-sm border rounded-md focus:ring-1 focus:ring-blue-500"
              >
                <option value="">All formats</option>
                <option value="wav">WAV</option>
                <option value="aiff">AIFF</option>
                <option value="mp3">MP3</option>
                <option value="flac">FLAC</option>
                <option value="m4a">M4A</option>
              </select>
            </div>

            {/* Filename pattern */}
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Filename contains
              </label>
              <div className="relative">
                <Search className="absolute left-2 top-1/2 transform -translate-y-1/2 w-3 h-3 text-gray-400" />
                <input
                  type="text"
                  value={filters.pattern || ''}
                  onChange={(e) => handleChange('pattern', e.target.value)}
                  placeholder="Search..."
                  className="w-full pl-7 pr-2 py-1.5 text-sm border rounded-md focus:ring-1 focus:ring-blue-500"
                />
              </div>
            </div>

            {/* User filter */}
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Added by
              </label>
              <input
                type="text"
                value={filters.user || ''}
                onChange={(e) => handleChange('user', e.target.value)}
                placeholder="Username..."
                className="w-full px-2 py-1.5 text-sm border rounded-md focus:ring-1 focus:ring-blue-500"
              />
            </div>

            {/* Duration range */}
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Min duration (sec)
              </label>
              <input
                type="number"
                value={filters.minDuration || ''}
                onChange={(e) => handleChange('minDuration', e.target.value ? Number(e.target.value) : undefined)}
                placeholder="0"
                min="0"
                className="w-full px-2 py-1.5 text-sm border rounded-md focus:ring-1 focus:ring-blue-500"
              />
            </div>

            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Max duration (sec)
              </label>
              <input
                type="number"
                value={filters.maxDuration || ''}
                onChange={(e) => handleChange('maxDuration', e.target.value ? Number(e.target.value) : undefined)}
                placeholder="No limit"
                min="0"
                className="w-full px-2 py-1.5 text-sm border rounded-md focus:ring-1 focus:ring-blue-500"
              />
            </div>

            {/* Size range */}
            <div>
              <label className="block text-xs font-medium text-gray-700 mb-1">
                Min size (MB)
              </label>
              <input
                type="number"
                value={filters.minSize ? filters.minSize / 1_000_000 : ''}
                onChange={(e) => handleChange('minSize', e.target.value ? Number(e.target.value) * 1_000_000 : undefined)}
                placeholder="0"
                min="0"
                step="0.1"
                className="w-full px-2 py-1.5 text-sm border rounded-md focus:ring-1 focus:ring-blue-500"
              />
            </div>
          </div>

          <div className="mt-4 flex justify-end">
            <button
              onClick={handleApply}
              className="px-4 py-2 bg-blue-500 text-white text-sm rounded-md hover:bg-blue-600 transition-colors"
            >
              Apply Filters
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
