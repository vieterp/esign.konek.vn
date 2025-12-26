/**
 * eSign Desktop - Main Application
 * PDF signing with Vietnamese USB tokens
 */

import { useCallback } from "react";
import { useToken } from "./hooks/useToken";
import { useSigning } from "./hooks/useSigning";
import { TokenStatus } from "./components/TokenStatus";
import { PinInput } from "./components/PinInput";
import { FileDropzone } from "./components/FileDropzone";
import { ResultModal } from "./components/ResultModal";

function App() {
  const token = useToken();
  const signing = useSigning();

  const handleSign = useCallback(async () => {
    await signing.sign({
      visible: true,
      signerName: token.certificate?.subject,
    });
  }, [signing, token.certificate]);

  const handleSignAnother = useCallback(() => {
    signing.reset();
  }, [signing]);

  const handleCloseModal = useCallback(() => {
    signing.clearResult();
  }, [signing]);

  const isReadyToSign =
    token.connectionState === "logged_in" &&
    signing.selectedFile !== null &&
    !signing.isProcessing;

  const showPinInput =
    token.connectionState === "ready" ||
    token.connectionState === "library_found";

  return (
    <div className="h-full bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800">
      <div className="max-w-2xl mx-auto px-6 py-4">
        {/* Header */}
        <header className="text-center py-4">
          <h1 className="text-3xl font-bold text-slate-800 dark:text-white mb-2">
            Konek eSign
          </h1>
          <p className="text-slate-600 dark:text-slate-400">
            Ký số PDF với USB Token
          </p>
        </header>

        <div className="space-y-6">
          {/* Token Status */}
          <TokenStatus
            connectionState={token.connectionState}
            certificate={token.certificate}
            selectedLibrary={token.selectedLibrary}
            tokens={token.tokens}
            error={token.error}
            isLoading={token.isLoading}
            onLogout={token.logout}
            onRefresh={token.refresh}
          />

          {/* PIN Input - Show when token ready but not logged in */}
          {showPinInput && (
            <PinInput
              onSubmit={token.login}
              disabled={token.connectionState !== "ready"}
              isLoading={token.connectionState === "logging_in"}
              error={token.error}
            />
          )}

          {/* File Dropzone */}
          <FileDropzone
            onFileSelect={signing.setFile}
            onBrowse={signing.selectFile}
            selectedFile={signing.selectedFile}
            fileName={signing.fileName}
            disabled={token.connectionState !== "logged_in"}
            isLoading={signing.isProcessing}
          />

          {/* Sign Button */}
          <button
            onClick={handleSign}
            disabled={!isReadyToSign}
            className={`
              w-full py-4 px-6 rounded-xl font-medium text-lg
              transition-all duration-200 shadow-lg
              ${isReadyToSign
                ? "bg-navy-700 hover:bg-navy-600 active:bg-navy-800 text-white hover:shadow-xl hover:-translate-y-0.5"
                : "bg-slate-300 dark:bg-slate-700 text-slate-500 dark:text-slate-400 cursor-not-allowed"
              }
            `}
          >
            {signing.isProcessing ? (
              <span className="flex items-center justify-center gap-2">
                <svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle
                    className="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    strokeWidth="4"
                  />
                  <path
                    className="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  />
                </svg>
                Đang ký số...
              </span>
            ) : (
              "Ký số PDF"
            )}
          </button>

          {/* Help text */}
          {token.connectionState !== "logged_in" && (
            <p className="text-center text-sm text-slate-500 dark:text-slate-400">
              {token.connectionState === "disconnected" || token.connectionState === "detecting"
                ? "Đang tìm kiếm USB Token..."
                : token.connectionState === "error"
                  ? "Vui lòng kiểm tra kết nối USB Token"
                  : "Nhập mã PIN để đăng nhập token"}
            </p>
          )}
        </div>

        {/* Result Modal */}
        <ResultModal
          isOpen={signing.state === "success" || signing.state === "error"}
          result={signing.result}
          error={signing.error}
          onClose={handleCloseModal}
          onSignAnother={handleSignAnother}
        />

        {/* Footer */}
        <footer className="mt-6 text-center text-sm text-slate-400 dark:text-slate-500">
          <p>Konek eSign v0.1.0 • VNPT-CA, Viettel-CA, FPT-CA</p>
        </footer>
      </div>
    </div>
  );
}

export default App;
