/**
 * PDFPreviewModal - Modal container for PDF preview and position selection
 * Combines PDF preview with draggable position indicator and page navigation.
 */

import { useCallback, useState, useEffect, useRef } from "react";
import { PDFPreview, PageDimensions } from "./PDFPreview";
import { SignaturePositioner } from "./SignaturePositioner";
import { PageNavigator } from "./PageNavigator";
import { PdfPosition, DEFAULT_SIGNATURE_WIDTH, DEFAULT_SIGNATURE_HEIGHT } from "../lib/pdf-coordinates";

interface PDFPreviewModalProps {
  /** Whether modal is open */
  isOpen: boolean;
  /** PDF file path */
  filePath: string;
  /** Initial position (if any) */
  initialPosition?: PdfPosition;
  /** Signature width in PDF points (from template) */
  signatureWidth?: number;
  /** Signature height in PDF points (from template) */
  signatureHeight?: number;
  /** Callback when position is confirmed */
  onConfirm: (position: PdfPosition) => void;
  /** Callback when modal is closed without confirming */
  onCancel: () => void;
}

// Default position at bottom right of page
function getDefaultPosition(pageWidth: number, _pageHeight: number, sigWidth: number, sigHeight: number): Omit<PdfPosition, 'page'> {
  const margin = 50;
  return {
    llx: pageWidth - sigWidth - margin,
    lly: margin,
    urx: pageWidth - margin,
    ury: margin + sigHeight,
  };
}

export function PDFPreviewModal({
  isOpen,
  filePath,
  initialPosition,
  signatureWidth = DEFAULT_SIGNATURE_WIDTH,
  signatureHeight = DEFAULT_SIGNATURE_HEIGHT,
  onConfirm,
  onCancel,
}: PDFPreviewModalProps) {
  const [currentPage, setCurrentPage] = useState(initialPosition?.page ?? 1);
  const [totalPages, setTotalPages] = useState(1);
  const [pageDims, setPageDims] = useState<PageDimensions | null>(null);
  const [canvasDims, setCanvasDims] = useState<{ width: number; height: number } | null>(null);
  const [position, setPosition] = useState<Omit<PdfPosition, 'page'> | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Reset state when modal opens
  useEffect(() => {
    if (isOpen) {
      setCurrentPage(initialPosition?.page ?? 1);
      if (initialPosition) {
        setPosition({
          llx: initialPosition.llx,
          lly: initialPosition.lly,
          urx: initialPosition.urx,
          ury: initialPosition.ury,
        });
      }
    }
  }, [isOpen, initialPosition]);

  // Set default position when page dimensions are loaded
  useEffect(() => {
    if (pageDims && !position) {
      setPosition(getDefaultPosition(pageDims.width, pageDims.height, signatureWidth, signatureHeight));
    }
  }, [pageDims, position, signatureWidth, signatureHeight]);

  // Update canvas dimensions when PDF loads
  const handlePageLoad = useCallback((dims: PageDimensions, total: number) => {
    setPageDims(dims);
    setTotalPages(total);

    // Get actual canvas size after render
    setTimeout(() => {
      const canvas = containerRef.current?.querySelector('canvas');
      if (canvas) {
        setCanvasDims({
          width: canvas.clientWidth,
          height: canvas.clientHeight,
        });
      }
    }, 50);
  }, []);

  const handlePositionChange = useCallback((newPosition: Omit<PdfPosition, 'page'>) => {
    setPosition(newPosition);
  }, []);

  const handleConfirm = useCallback(() => {
    if (position) {
      onConfirm({
        page: currentPage,
        ...position,
      });
    }
  }, [position, currentPage, onConfirm]);

  const handlePageChange = useCallback((page: number) => {
    setCurrentPage(page);
    // Reset position for new page
    if (pageDims) {
      setPosition(getDefaultPosition(pageDims.width, pageDims.height, signatureWidth, signatureHeight));
    }
  }, [pageDims, signatureWidth, signatureHeight]);

  if (!isOpen) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50"
        onClick={onCancel}
      />

      {/* Modal content */}
      <div className="relative bg-white dark:bg-slate-800 rounded-xl shadow-2xl max-w-4xl w-full mx-4 max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-200 dark:border-slate-700">
          <h2 className="text-lg font-semibold text-slate-800 dark:text-white">
            Chọn vị trí chữ ký
          </h2>
          <button
            onClick={onCancel}
            className="p-2 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-lg transition-colors"
            aria-label="Đóng"
          >
            <svg className="w-5 h-5 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* PDF Preview area */}
        <div className="flex-1 overflow-auto p-6">
          <div ref={containerRef} className="relative inline-block">
            <PDFPreview
              filePath={filePath}
              currentPage={currentPage}
              scale={1.2}
              onPageLoad={handlePageLoad}
            />

            {/* Draggable position indicator */}
            {position && pageDims && canvasDims && (
              <SignaturePositioner
                position={position}
                pageWidth={pageDims.width}
                pageHeight={pageDims.height}
                canvasWidth={canvasDims.width}
                canvasHeight={canvasDims.height}
                onPositionChange={handlePositionChange}
              />
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-6 py-4 border-t border-slate-200 dark:border-slate-700">
          {/* Page navigator */}
          <PageNavigator
            currentPage={currentPage}
            totalPages={totalPages}
            onPageChange={handlePageChange}
          />

          {/* Action buttons */}
          <div className="flex items-center gap-3">
            <button
              onClick={onCancel}
              className="px-4 py-2 text-sm text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-lg transition-colors"
            >
              Hủy
            </button>
            <button
              onClick={handleConfirm}
              disabled={!position}
              className={`
                px-4 py-2 text-sm font-medium rounded-lg transition-colors
                ${position
                  ? "bg-ocean-500 hover:bg-ocean-600 text-white"
                  : "bg-slate-200 dark:bg-slate-700 text-slate-400 cursor-not-allowed"
                }
              `}
            >
              Xác nhận
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
