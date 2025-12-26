/**
 * ResultModal - Success/error modal component
 * Shows signing result with output path and actions
 */

import { useCallback, useEffect } from "react";
import { SignResult } from "../lib/tauri";

interface ResultModalProps {
  isOpen: boolean;
  result: SignResult | null;
  error: string | null;
  onClose: () => void;
  onSignAnother: () => void;
}

export function ResultModal({
  isOpen,
  result,
  error,
  onClose,
  onSignAnother,
}: ResultModalProps) {
  // Close on Escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  const handleBackdropClick = useCallback((e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }, [onClose]);

  if (!isOpen) return null;

  const isSuccess = result?.success && !error;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={handleBackdropClick}
    >
      <div className="bg-white dark:bg-slate-800 rounded-2xl shadow-xl max-w-md w-full mx-4 overflow-hidden animate-in fade-in zoom-in-95 duration-200">
        {/* Header */}
        <div className={`p-6 ${isSuccess ? "bg-green-50 dark:bg-green-900/20" : "bg-red-50 dark:bg-red-900/20"}`}>
          <div className="flex items-center gap-4">
            {isSuccess ? (
              <div className="p-3 bg-green-100 dark:bg-green-800/50 rounded-full">
                <svg
                  className="w-8 h-8 text-green-600 dark:text-green-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              </div>
            ) : (
              <div className="p-3 bg-red-100 dark:bg-red-800/50 rounded-full">
                <svg
                  className="w-8 h-8 text-red-600 dark:text-red-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </div>
            )}
            <div>
              <h2 className={`text-xl font-semibold ${isSuccess ? "text-green-800 dark:text-green-200" : "text-red-800 dark:text-red-200"}`}>
                {isSuccess ? "Ký số thành công!" : "Ký số thất bại"}
              </h2>
              {result?.signing_time && (
                <p className="text-sm text-slate-500 dark:text-slate-400 mt-1">
                  {new Date(result.signing_time).toLocaleString("vi-VN")}
                </p>
              )}
            </div>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 space-y-4">
          {isSuccess && result ? (
            <>
              <div>
                <p className="text-sm text-slate-500 dark:text-slate-400 mb-1">
                  File đã ký được lưu tại:
                </p>
                <div className="p-3 bg-slate-100 dark:bg-slate-900 rounded-lg">
                  <p className="font-mono text-sm text-slate-700 dark:text-slate-200 break-all">
                    {result.output_path}
                  </p>
                </div>
              </div>
              {result.message && (
                <p className="text-sm text-slate-600 dark:text-slate-300">
                  {result.message}
                </p>
              )}
            </>
          ) : (
            <div className="p-4 bg-red-50 dark:bg-red-900/20 rounded-lg">
              <p className="text-red-700 dark:text-red-300">
                {error || result?.message || "Đã xảy ra lỗi không xác định"}
              </p>
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="p-4 bg-slate-50 dark:bg-slate-900/50 flex gap-3 justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-700 rounded-lg transition-colors"
          >
            Đóng
          </button>
          <button
            onClick={onSignAnother}
            className={`px-4 py-2 text-white rounded-lg transition-colors ${
              isSuccess
                ? "bg-green-600 hover:bg-green-700"
                : "bg-ocean-500 hover:bg-ocean-600"
            }`}
          >
            {isSuccess ? "Ký file khác" : "Thử lại"}
          </button>
        </div>
      </div>
    </div>
  );
}
