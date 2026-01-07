/**
 * SignaturePositioner - Draggable overlay component for signature position selection
 * Shows a visual indicator at the selected position on the PDF canvas.
 */

import { useRef, useState, useCallback, useEffect } from "react";
import { pdfToScreenCoords, screenToPdfCoords, PdfPosition } from "../lib/pdf-coordinates";

interface SignaturePositionerProps {
  /** Selected position in PDF coordinates */
  position: Omit<PdfPosition, 'page'>;
  /** PDF page width in points */
  pageWidth: number;
  /** PDF page height in points */
  pageHeight: number;
  /** Canvas width in pixels */
  canvasWidth: number;
  /** Canvas height in pixels */
  canvasHeight: number;
  /** Callback when position changes via drag */
  onPositionChange?: (newPosition: Omit<PdfPosition, 'page'>) => void;
}

export function SignaturePositioner({
  position,
  pageWidth,
  pageHeight,
  canvasWidth,
  canvasHeight,
  onPositionChange,
}: SignaturePositionerProps) {
  const boxRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });

  // Convert PDF coordinates to screen coordinates
  const screenPosition = pdfToScreenCoords(position, pageWidth, pageHeight, canvasWidth, canvasHeight);

  // Handle drag start
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const rect = boxRef.current?.getBoundingClientRect();
    if (rect) {
      setDragOffset({
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      });
      setIsDragging(true);
    }
  }, []);

  // Handle drag move
  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      // Get parent container (PDF canvas wrapper)
      const parent = boxRef.current?.parentElement;
      if (!parent) return;

      const parentRect = parent.getBoundingClientRect();

      // Calculate new position relative to parent
      let newX = e.clientX - parentRect.left - dragOffset.x;
      let newY = e.clientY - parentRect.top - dragOffset.y;

      // Constrain to parent bounds
      const maxX = canvasWidth - screenPosition.width;
      const maxY = canvasHeight - screenPosition.height;
      newX = Math.max(0, Math.min(newX, maxX));
      newY = Math.max(0, Math.min(newY, maxY));

      // Convert back to PDF coordinates, preserving original signature size
      if (onPositionChange) {
        // Calculate signature size from current position
        const sigWidth = position.urx - position.llx;
        const sigHeight = position.ury - position.lly;

        const newPdfCoords = screenToPdfCoords(
          newX + screenPosition.width / 2, // center point
          newY + screenPosition.height / 2,
          pageWidth,
          pageHeight,
          canvasWidth,
          canvasHeight,
          sigWidth,
          sigHeight
        );
        onPositionChange(newPdfCoords);
      }
    };

    const handleMouseUp = () => {
      setIsDragging(false);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
    // position.llx/lly/urx/ury intentionally omitted - we read latest during drag
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isDragging, dragOffset, canvasWidth, canvasHeight, pageWidth, pageHeight, screenPosition.width, screenPosition.height, onPositionChange]);

  return (
    <div
      ref={boxRef}
      className={`absolute select-none ${isDragging ? 'cursor-grabbing' : 'cursor-grab'}`}
      style={{
        left: screenPosition.x,
        top: screenPosition.y,
        width: screenPosition.width,
        height: screenPosition.height,
        zIndex: 10,
      }}
      onMouseDown={handleMouseDown}
    >
      {/* Signature placeholder box */}
      <div
        className={`
          w-full h-full
          border-2 ${isDragging ? 'border-ocean-600 bg-ocean-500/20' : 'border-dashed border-ocean-500 bg-ocean-500/10'}
          rounded-sm
          transition-colors duration-150
        `}
      >
        {/* Corner handles */}
        <div className="absolute -left-1 -top-1 w-2 h-2 bg-ocean-500 rounded-full" />
        <div className="absolute -right-1 -top-1 w-2 h-2 bg-ocean-500 rounded-full" />
        <div className="absolute -left-1 -bottom-1 w-2 h-2 bg-ocean-500 rounded-full" />
        <div className="absolute -right-1 -bottom-1 w-2 h-2 bg-ocean-500 rounded-full" />

        {/* Label */}
        <div className="absolute -top-6 left-0 text-xs text-ocean-600 dark:text-ocean-400 whitespace-nowrap font-medium">
          {isDragging ? 'Đang di chuyển...' : 'Kéo để di chuyển'}
        </div>

        {/* Center icon */}
        <div className="absolute inset-0 flex items-center justify-center">
          <svg className="w-6 h-6 text-ocean-500/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 11.5V14m0-2.5v-6a1.5 1.5 0 113 0m-3 6a1.5 1.5 0 00-3 0v2a7.5 7.5 0 0015 0v-5a1.5 1.5 0 00-3 0m-6-3V11m0-5.5v-1a1.5 1.5 0 013 0v1m0 0V11m0-5.5a1.5 1.5 0 013 0v3m0 0V11" />
          </svg>
        </div>
      </div>
    </div>
  );
}
