/**
 * PageNavigator - PDF page navigation controls
 * Provides previous/next buttons and page number input.
 */

import { useCallback, useState, KeyboardEvent } from "react";

interface PageNavigatorProps {
  /** Current page number (1-indexed) */
  currentPage: number;
  /** Total number of pages */
  totalPages: number;
  /** Callback when page changes */
  onPageChange: (page: number) => void;
  /** Whether navigation is disabled */
  disabled?: boolean;
}

export function PageNavigator({
  currentPage,
  totalPages,
  onPageChange,
  disabled = false,
}: PageNavigatorProps) {
  const [inputValue, setInputValue] = useState(String(currentPage));

  const goToPrevious = useCallback(() => {
    if (currentPage > 1) {
      const newPage = currentPage - 1;
      onPageChange(newPage);
      setInputValue(String(newPage));
    }
  }, [currentPage, onPageChange]);

  const goToNext = useCallback(() => {
    if (currentPage < totalPages) {
      const newPage = currentPage + 1;
      onPageChange(newPage);
      setInputValue(String(newPage));
    }
  }, [currentPage, totalPages, onPageChange]);

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    setInputValue(e.target.value);
  }, []);

  const handleInputBlur = useCallback(() => {
    const page = parseInt(inputValue, 10);
    if (!isNaN(page) && page >= 1 && page <= totalPages) {
      onPageChange(page);
    } else {
      setInputValue(String(currentPage));
    }
  }, [inputValue, totalPages, currentPage, onPageChange]);

  const handleInputKeyDown = useCallback(
    (e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter") {
        handleInputBlur();
      } else if (e.key === "Escape") {
        setInputValue(String(currentPage));
      }
    },
    [handleInputBlur, currentPage]
  );

  // Sync input value when currentPage changes externally
  if (inputValue !== String(currentPage) && document.activeElement?.tagName !== "INPUT") {
    setInputValue(String(currentPage));
  }

  return (
    <div className="flex items-center gap-2">
      {/* Previous button */}
      <button
        onClick={goToPrevious}
        disabled={disabled || currentPage <= 1}
        className={`
          p-2 rounded-lg transition-colors
          ${disabled || currentPage <= 1
            ? "bg-slate-200 dark:bg-slate-700 text-slate-400 cursor-not-allowed"
            : "bg-slate-100 dark:bg-slate-700 hover:bg-slate-200 dark:hover:bg-slate-600 text-slate-700 dark:text-slate-200"
          }
        `}
        title="Trang trước"
        aria-label="Trang trước"
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
        </svg>
      </button>

      {/* Page indicator */}
      <div className="flex items-center gap-1.5 text-sm">
        <span className="text-slate-500 dark:text-slate-400">Trang</span>
        <input
          type="text"
          value={inputValue}
          onChange={handleInputChange}
          onBlur={handleInputBlur}
          onKeyDown={handleInputKeyDown}
          disabled={disabled}
          className={`
            w-12 px-2 py-1 text-center rounded border
            bg-white dark:bg-slate-800
            border-slate-300 dark:border-slate-600
            text-slate-700 dark:text-slate-200
            focus:outline-none focus:ring-2 focus:ring-ocean-500
            disabled:bg-slate-100 dark:disabled:bg-slate-700 disabled:cursor-not-allowed
          `}
          aria-label="Số trang hiện tại"
        />
        <span className="text-slate-500 dark:text-slate-400">/ {totalPages}</span>
      </div>

      {/* Next button */}
      <button
        onClick={goToNext}
        disabled={disabled || currentPage >= totalPages}
        className={`
          p-2 rounded-lg transition-colors
          ${disabled || currentPage >= totalPages
            ? "bg-slate-200 dark:bg-slate-700 text-slate-400 cursor-not-allowed"
            : "bg-slate-100 dark:bg-slate-700 hover:bg-slate-200 dark:hover:bg-slate-600 text-slate-700 dark:text-slate-200"
          }
        `}
        title="Trang sau"
        aria-label="Trang sau"
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
        </svg>
      </button>
    </div>
  );
}
