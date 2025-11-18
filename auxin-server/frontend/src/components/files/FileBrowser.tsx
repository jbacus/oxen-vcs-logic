import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { getFileTree } from '@/services/api';
import { Folder, FolderOpen, File, ChevronRight, ChevronDown, Download } from 'lucide-react';

interface FileBrowserProps {
  namespace: string;
  name: string;
  commit: string;
}

interface FileEntry {
  name: string;
  path: string;
  type: 'file' | 'dir';
  size?: number;
  children?: FileEntry[];
}

function formatFileSize(bytes?: number): string {
  if (!bytes) return '';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function FileTreeNode({ entry, depth = 0 }: { entry: FileEntry; depth?: number }) {
  const [isExpanded, setIsExpanded] = useState(depth < 2);

  const isDirectory = entry.type === 'dir';

  return (
    <div>
      <div
        className={`flex items-center py-1.5 px-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded cursor-pointer ${
          isDirectory ? 'cursor-pointer' : ''
        }`}
        style={{ paddingLeft: `${depth * 20 + 8}px` }}
        onClick={() => isDirectory && setIsExpanded(!isExpanded)}
      >
        {isDirectory ? (
          <>
            {isExpanded ? (
              <ChevronDown className="w-4 h-4 text-gray-400 mr-1 flex-shrink-0" />
            ) : (
              <ChevronRight className="w-4 h-4 text-gray-400 mr-1 flex-shrink-0" />
            )}
            {isExpanded ? (
              <FolderOpen className="w-4 h-4 text-yellow-500 mr-2 flex-shrink-0" />
            ) : (
              <Folder className="w-4 h-4 text-yellow-500 mr-2 flex-shrink-0" />
            )}
          </>
        ) : (
          <>
            <span className="w-4 mr-1" />
            <File className="w-4 h-4 text-gray-400 mr-2 flex-shrink-0" />
          </>
        )}
        <span className="text-sm text-gray-900 dark:text-gray-100 truncate flex-1">
          {entry.name}
        </span>
        {!isDirectory && entry.size && (
          <span className="text-xs text-gray-500 dark:text-gray-400 ml-2">
            {formatFileSize(entry.size)}
          </span>
        )}
        {!isDirectory && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              // TODO: Implement download
            }}
            className="ml-2 p-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded opacity-0 group-hover:opacity-100"
          >
            <Download className="w-3 h-3 text-gray-500" />
          </button>
        )}
      </div>
      {isDirectory && isExpanded && entry.children && (
        <div>
          {entry.children.map((child) => (
            <FileTreeNode key={child.path} entry={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
}

export function FileBrowser({ namespace, name, commit }: FileBrowserProps) {
  const { data: files = [], isLoading, error } = useQuery({
    queryKey: ['files', namespace, name, commit],
    queryFn: () => getFileTree(namespace, name, commit).then((res) => res.data),
    enabled: !!commit,
  });

  if (isLoading) {
    return (
      <div className="animate-pulse space-y-2">
        {[1, 2, 3, 4, 5].map((i) => (
          <div key={i} className="h-8 bg-gray-200 dark:bg-gray-700 rounded" />
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-8 text-gray-500 dark:text-gray-400">
        Failed to load file tree
      </div>
    );
  }

  if (files.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500 dark:text-gray-400">
        No files in this commit
      </div>
    );
  }

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
      <div className="bg-gray-50 dark:bg-gray-800 px-4 py-2 border-b border-gray-200 dark:border-gray-700">
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
          Files at {commit.substring(0, 8)}
        </span>
      </div>
      <div className="p-2 max-h-96 overflow-y-auto">
        {files.map((file: FileEntry) => (
          <FileTreeNode key={file.path} entry={file} />
        ))}
      </div>
    </div>
  );
}
