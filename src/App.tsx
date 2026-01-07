/**
 * eSign Desktop - Main Application
 * PDF signing with Vietnamese USB tokens
 */

import { useCallback, useState, useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { useToken } from "./hooks/useToken";
import { useSigning } from "./hooks/useSigning";
import { useSignatureTemplate } from "./hooks/useSignatureTemplate";
import { useSignaturePosition } from "./hooks/useSignaturePosition";
import { TokenStatus } from "./components/TokenStatus";
import { PinInput } from "./components/PinInput";
import { FileDropzone } from "./components/FileDropzone";
import { ResultModal } from "./components/ResultModal";
import { PDFPreviewModal } from "./components/PDFPreviewModal";
import { TemplateDesigner } from "./components/TemplateDesigner";
import { PdfPosition } from "./lib/pdf-coordinates";
import { extractCommonName } from "./lib/certificate-utils";

function App() {
  const token = useToken();
  const signing = useSigning();
  const template = useSignatureTemplate();
  const position = useSignaturePosition();
  const [appVersion, setAppVersion] = useState("0.0.0");
  const [showPreviewModal, setShowPreviewModal] = useState(false);
  const [showTemplateDesigner, setShowTemplateDesigner] = useState(false);

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => setAppVersion("0.0.0"));
  }, []);

  // Position is persisted in localStorage via useSignaturePosition hook
  // No need to clear on file change - user may want same position for multiple files

  // Update position size when template width/height changes
  useEffect(() => {
    if (position.hasCustomPosition) {
      const newUrx = position.position.llx + template.template.width;
      const newUry = position.position.lly + template.template.height;
      // Only update if size actually changed
      if (newUrx !== position.position.urx || newUry !== position.position.ury) {
        position.updateCoordinates({
          llx: position.position.llx,
          lly: position.position.lly,
          urx: newUrx,
          ury: newUry,
        });
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [template.template.width, template.template.height]);

  const handleSign = useCallback(async () => {
    // Extract company name (CN) from certificate subject
    const companyName = token.certificate?.subject
      ? extractCommonName(token.certificate.subject)
      : undefined;

    await signing.sign({
      visible: true,
      signerName: companyName,
      position: position.hasCustomPosition ? position.position : undefined,
      appearance: {
        fontFamily: template.template.font.family,
        fontSize: template.template.font.size,
        colorHex: template.template.font.color,
        showName: template.template.fields.showName,
        showTimestamp: template.template.fields.showTimestamp,
        showReason: template.template.fields.showReason,
      },
    });
  }, [signing, token.certificate, position, template.template]);

  const handleSignAnother = useCallback(() => {
    signing.reset();
    position.clear();
  }, [signing, position]);

  const handleCloseModal = useCallback(() => {
    signing.clearResult();
  }, [signing]);

  const handleOpenPreview = useCallback(() => {
    setShowPreviewModal(true);
  }, []);

  const handleClosePreview = useCallback(() => {
    setShowPreviewModal(false);
  }, []);

  const handleConfirmPosition = useCallback((newPosition: PdfPosition) => {
    position.setPosition(newPosition);
    setShowPreviewModal(false);
  }, [position]);

  const handleToggleTemplate = useCallback(() => {
    setShowTemplateDesigner(prev => !prev);
  }, []);

  const isReadyToSign =
    token.connectionState === "logged_in" &&
    signing.selectedFile !== null &&
    !signing.isProcessing;

  const showPinInput =
    token.connectionState === "ready" ||
    token.connectionState === "library_found";

  const canSelectPosition =
    token.connectionState === "logged_in" &&
    signing.selectedFile !== null;

  return (
    <div className="h-full flex flex-col bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800">
      <div className="flex-1 min-h-0 overflow-y-auto">
        <div className="max-w-2xl mx-auto px-6 py-4">
          {/* Header */}
          <header className="text-center py-2">
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

          {/* Position & Template Controls */}
          {canSelectPosition && (
            <div className="flex gap-3">
              {/* Position Selection Button */}
              <button
                onClick={handleOpenPreview}
                className="flex-1 py-3 px-4 rounded-xl font-medium text-sm border-2 border-ocean-500 text-ocean-600 dark:text-ocean-400 hover:bg-ocean-50 dark:hover:bg-ocean-900/20 transition-colors"
              >
                <span className="flex items-center justify-center gap-2">
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  {position.hasCustomPosition ? "Đổi vị trí" : "Chọn vị trí"}
                </span>
              </button>

              {/* Template Designer Toggle */}
              <button
                onClick={handleToggleTemplate}
                className={`flex-1 py-3 px-4 rounded-xl font-medium text-sm border-2 transition-colors ${
                  showTemplateDesigner
                    ? "border-sky-500 bg-sky-50 dark:bg-sky-900/20 text-sky-600 dark:text-sky-400"
                    : "border-slate-300 dark:border-slate-600 text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-700"
                }`}
              >
                <span className="flex items-center justify-center gap-2">
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
                  </svg>
                  Tùy chỉnh chữ ký
                </span>
              </button>
            </div>
          )}

          {/* Position Indicator */}
          {position.hasCustomPosition && (
            <div className="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400 bg-slate-100 dark:bg-slate-800 rounded-lg px-4 py-2">
              <svg className="w-4 h-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              <span>
                Vị trí: Trang {position.position.page}, ({Math.round(position.position.llx)}, {Math.round(position.position.lly)})
              </span>
              <button
                onClick={position.reset}
                className="ml-auto text-slate-400 hover:text-slate-600 dark:hover:text-slate-300"
                title="Xóa vị trí đã chọn"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          )}

          {/* Template Designer Panel */}
          {showTemplateDesigner && (
            <div className="bg-white dark:bg-slate-800 rounded-xl shadow-lg p-6 border border-slate-200 dark:border-slate-700">
              <div className="flex items-center justify-between mb-4">
                <h3 className="font-semibold text-slate-800 dark:text-white">Tùy chỉnh chữ ký</h3>
                <button
                  onClick={handleToggleTemplate}
                  className="p-1 hover:bg-slate-100 dark:hover:bg-slate-700 rounded"
                >
                  <svg className="w-5 h-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
              <TemplateDesigner
                template={template.template}
                signerName={token.certificate?.subject}
                onPresetChange={template.applyPreset}
                onWidthChange={template.setWidth}
                onHeightChange={template.setHeight}
                onFontSizeChange={template.setFontSize}
                onFontColorChange={template.setFontColor}
                onToggleName={template.toggleShowName}
                onToggleTimestamp={template.toggleShowTimestamp}
                onReset={template.reset}
              />
            </div>
          )}

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

          {/* Footer - scrolls with content */}
          <footer className="mt-8 pt-4 text-center text-sm text-slate-400 dark:text-slate-500 border-t border-slate-200 dark:border-slate-700">
            <p>Konek eSign v{appVersion} • VNPT-CA, Viettel-CA, FPT-CA</p>
          </footer>
        </div>

        {/* PDF Preview Modal */}
        {signing.selectedFile && (
          <PDFPreviewModal
            isOpen={showPreviewModal}
            filePath={signing.selectedFile}
            initialPosition={position.hasCustomPosition ? position.position : undefined}
            signatureWidth={template.template.width}
            signatureHeight={template.template.height}
            onConfirm={handleConfirmPosition}
            onCancel={handleClosePreview}
          />
        )}

        {/* Result Modal */}
        <ResultModal
          isOpen={signing.state === "success" || signing.state === "error"}
          result={signing.result}
          error={signing.error}
          onClose={handleCloseModal}
          onSignAnother={handleSignAnother}
        />

        </div>
      </div>

    </div>
  );
}

export default App;
