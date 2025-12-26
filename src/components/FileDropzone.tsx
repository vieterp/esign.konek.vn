/**
 * FileDropzone - PDF file selection component
 * Supports click to select and drag-and-drop
 */

import { useCallback, useState, DragEvent } from "react";

interface FileDropzoneProps {
  onFileSelect: (path: string) => void;
  onBrowse: () => void;
  selectedFile: string | null;
  fileName: string | null;
  disabled?: boolean;
  isLoading?: boolean;
}

export function FileDropzone({
  onFileSelect,
  onBrowse,
  selectedFile,
  fileName,
  disabled = false,
  isLoading = false,
}: FileDropzoneProps) {
  const [isDragging, setIsDragging] = useState(false);

  const handleDragOver = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    if (!disabled) {
      setIsDragging(true);
    }
  }, [disabled]);

  const handleDragLeave = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    if (disabled) return;

    const files = e.dataTransfer.files;
    if (files.length > 0) {
      const file = files[0];
      if (file.name.toLowerCase().endsWith(".pdf")) {
        // In Tauri, we get the file path from the webkitRelativePath or name
        // For drag-drop, we need to use the File System Access API path
        // This is a workaround - actual path comes from Tauri file drop event
        const path = (file as File & { path?: string }).path || file.name;
        onFileSelect(path);
      }
    }
  }, [disabled, onFileSelect]);

  const handleClick = useCallback(() => {
    if (!disabled && !isLoading) {
      onBrowse();
    }
  }, [disabled, isLoading, onBrowse]);

  return (
    <div
      onClick={handleClick}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      className={`
        relative rounded-xl p-8 text-center transition-all cursor-pointer
        border-2 border-dashed
        ${isDragging
          ? "border-ocean-500 bg-ocean-50 dark:bg-ocean-900/20"
          : selectedFile
            ? "border-sky-400 bg-sky-50 dark:bg-sky-900/20"
            : "border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-800"
        }
        ${disabled ? "opacity-50 cursor-not-allowed" : "hover:border-ocean-400 hover:bg-slate-50 dark:hover:bg-slate-750"}
      `}
    >
      {isLoading ? (
        <div className="flex flex-col items-center gap-3">
          <div className="w-8 h-8 border-2 border-ocean-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-slate-500 dark:text-slate-400">Đang xử lý...</span>
        </div>
      ) : selectedFile ? (
        <div className="flex flex-col items-center gap-3">
          <svg
            className="w-12 h-12 text-sky-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
          <div>
            <p className="font-medium text-slate-700 dark:text-slate-200 truncate max-w-xs">
              {fileName}
            </p>
            <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
              Nhấn để chọn file khác
            </p>
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center gap-3">
          <svg
            className={`w-12 h-12 ${isDragging ? "text-ocean-500" : "text-slate-400"}`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
            />
          </svg>
          <div>
            <p className="text-slate-600 dark:text-slate-300">
              Kéo thả file PDF vào đây
            </p>
            <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
              hoặc nhấn để chọn file
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
