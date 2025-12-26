/**
 * TokenStatus - Token and certificate display component
 * Shows connection state, certificate info, and logout button
 */

import { CertificateInfo, DetectedLibrary, TokenInfo } from "../lib/tauri";
import { ConnectionState } from "../hooks/useToken";

interface TokenStatusProps {
  connectionState: ConnectionState;
  certificate: CertificateInfo | null;
  selectedLibrary: DetectedLibrary | null;
  tokens: TokenInfo[];
  error: string | null;
  isLoading: boolean;
  onLogout: () => void;
  onRefresh: () => void;
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString("vi-VN", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
    });
  } catch {
    return dateStr;
  }
}

function getStatusColor(state: ConnectionState): string {
  switch (state) {
    case "logged_in":
      return "bg-green-500";
    case "ready":
    case "library_found":
      return "bg-yellow-500";
    case "detecting":
    case "initializing":
    case "logging_in":
      return "bg-ocean-500 animate-pulse";
    case "error":
      return "bg-red-500";
    default:
      return "bg-slate-400";
  }
}

function getStatusText(state: ConnectionState): string {
  switch (state) {
    case "logged_in":
      return "Đã đăng nhập";
    case "ready":
      return "Sẵn sàng - Nhập PIN để đăng nhập";
    case "library_found":
      return "Đã tìm thấy thư viện";
    case "detecting":
      return "Đang tìm kiếm token...";
    case "initializing":
      return "Đang khởi tạo...";
    case "logging_in":
      return "Đang đăng nhập...";
    case "error":
      return "Lỗi kết nối";
    default:
      return "Chưa kết nối";
  }
}

export function TokenStatus({
  connectionState,
  certificate,
  selectedLibrary,
  tokens,
  error,
  isLoading,
  onLogout,
  onRefresh,
}: TokenStatusProps) {
  const tokenWithSlot = tokens.find(t => t.has_token);

  return (
    <div className="bg-white dark:bg-slate-800 rounded-xl shadow-sm overflow-hidden">
      {/* Status Header */}
      <div className="p-4 border-b border-slate-200 dark:border-slate-700">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className={`w-3 h-3 rounded-full ${getStatusColor(connectionState)}`} />
            <span className="font-medium text-slate-700 dark:text-slate-200">
              {getStatusText(connectionState)}
            </span>
          </div>
          <div className="flex items-center gap-2">
            {connectionState === "logged_in" && (
              <button
                onClick={onLogout}
                disabled={isLoading}
                className="text-sm px-3 py-1 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors disabled:opacity-50"
              >
                Đăng xuất
              </button>
            )}
            <button
              onClick={onRefresh}
              disabled={isLoading}
              className="p-1.5 text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-lg transition-colors disabled:opacity-50"
              title="Làm mới"
            >
              <svg
                className={`w-4 h-4 ${isLoading ? "animate-spin" : ""}`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                />
              </svg>
            </button>
          </div>
        </div>

        {/* Error message */}
        {error && (
          <div className="mt-3 p-3 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-sm rounded-lg">
            {error}
          </div>
        )}

        {/* Library info */}
        {selectedLibrary && connectionState !== "logged_in" && (
          <div className="mt-3 text-sm text-slate-500 dark:text-slate-400">
            <span className="font-medium">{selectedLibrary.ca_name}</span>
            {tokenWithSlot && (
              <span className="ml-2">
                - {tokenWithSlot.label || tokenWithSlot.model}
              </span>
            )}
          </div>
        )}
      </div>

      {/* Certificate Info */}
      {certificate && connectionState === "logged_in" && (
        <div className="p-4 space-y-3">
          <div className="flex items-start gap-3">
            <div className="p-2 bg-green-100 dark:bg-green-900/30 rounded-lg">
              <svg
                className="w-5 h-5 text-green-600 dark:text-green-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                />
              </svg>
            </div>
            <div className="flex-1 min-w-0">
              <p className="font-medium text-slate-800 dark:text-slate-100 truncate">
                {certificate.subject}
              </p>
              <p className="text-sm text-slate-500 dark:text-slate-400 truncate">
                {certificate.issuer}
              </p>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <p className="text-slate-500 dark:text-slate-400">Số serial</p>
              <p className="font-mono text-slate-700 dark:text-slate-200 truncate" title={certificate.serial}>
                {certificate.serial.length > 16
                  ? certificate.serial.slice(0, 8) + "..." + certificate.serial.slice(-8)
                  : certificate.serial}
              </p>
            </div>
            <div>
              <p className="text-slate-500 dark:text-slate-400">Hiệu lực</p>
              <p className="text-slate-700 dark:text-slate-200">
                {formatDate(certificate.valid_from)} - {formatDate(certificate.valid_to)}
              </p>
            </div>
          </div>

          {selectedLibrary && (
            <div className="pt-2 border-t border-slate-200 dark:border-slate-700 text-sm text-slate-500 dark:text-slate-400">
              {selectedLibrary.ca_name}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
