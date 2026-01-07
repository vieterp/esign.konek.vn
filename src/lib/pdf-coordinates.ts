/**
 * PDF Coordinate Conversion Utilities
 * Handles conversion between screen coordinates and PDF coordinates.
 *
 * PDF coordinate system: origin at bottom-left, Y-axis upward
 * Screen coordinate system: origin at top-left, Y-axis downward
 */

/** Signature position in PDF coordinate system (points) */
export interface PdfPosition {
  /** Page number (1-indexed) */
  page: number;
  /** Lower-left X coordinate in PDF points */
  llx: number;
  /** Lower-left Y coordinate in PDF points */
  lly: number;
  /** Upper-right X coordinate in PDF points */
  urx: number;
  /** Upper-right Y coordinate in PDF points */
  ury: number;
}

/** Default signature dimensions in PDF points */
export const DEFAULT_SIGNATURE_WIDTH = 200;
export const DEFAULT_SIGNATURE_HEIGHT = 50;

/**
 * Convert screen click coordinates to PDF signature position.
 * Centers the signature on the click point.
 *
 * @param screenX - Click X coordinate relative to canvas
 * @param screenY - Click Y coordinate relative to canvas
 * @param pageWidth - PDF page width in points
 * @param pageHeight - PDF page height in points
 * @param canvasWidth - Rendered canvas width in pixels
 * @param canvasHeight - Rendered canvas height in pixels
 * @param sigWidth - Signature width in PDF points
 * @param sigHeight - Signature height in PDF points
 * @returns PDF coordinates (llx, lly, urx, ury)
 */
export function screenToPdfCoords(
  screenX: number,
  screenY: number,
  pageWidth: number,
  pageHeight: number,
  canvasWidth: number,
  canvasHeight: number,
  sigWidth: number = DEFAULT_SIGNATURE_WIDTH,
  sigHeight: number = DEFAULT_SIGNATURE_HEIGHT
): Omit<PdfPosition, 'page'> {
  // Scale factors from canvas pixels to PDF points
  const scaleX = pageWidth / canvasWidth;
  const scaleY = pageHeight / canvasHeight;

  // Convert screen coords to PDF coords (invert Y-axis)
  const pdfX = screenX * scaleX;
  const pdfY = pageHeight - (screenY * scaleY);

  // Center signature on click point, clamp to page bounds
  const halfWidth = sigWidth / 2;
  const halfHeight = sigHeight / 2;

  return {
    llx: Math.max(0, Math.min(pageWidth - sigWidth, pdfX - halfWidth)),
    lly: Math.max(0, Math.min(pageHeight - sigHeight, pdfY - halfHeight)),
    urx: Math.min(pageWidth, Math.max(sigWidth, pdfX + halfWidth)),
    ury: Math.min(pageHeight, Math.max(sigHeight, pdfY + halfHeight)),
  };
}

/**
 * Convert PDF coordinates to screen position for overlay rendering.
 *
 * @param pdfPosition - PDF coordinates
 * @param pageWidth - PDF page width in points
 * @param pageHeight - PDF page height in points
 * @param canvasWidth - Rendered canvas width in pixels
 * @param canvasHeight - Rendered canvas height in pixels
 * @returns Screen position and dimensions for overlay
 */
export function pdfToScreenCoords(
  pdfPosition: Omit<PdfPosition, 'page'>,
  pageWidth: number,
  pageHeight: number,
  canvasWidth: number,
  canvasHeight: number
): { x: number; y: number; width: number; height: number } {
  const scaleX = canvasWidth / pageWidth;
  const scaleY = canvasHeight / pageHeight;

  // Convert PDF coords to screen coords (invert Y-axis)
  const screenX = pdfPosition.llx * scaleX;
  const screenY = (pageHeight - pdfPosition.ury) * scaleY;
  const width = (pdfPosition.urx - pdfPosition.llx) * scaleX;
  const height = (pdfPosition.ury - pdfPosition.lly) * scaleY;

  return { x: screenX, y: screenY, width, height };
}

/**
 * Validate that position is within page bounds.
 */
export function isValidPosition(
  position: Omit<PdfPosition, 'page'>,
  pageWidth: number,
  pageHeight: number
): boolean {
  return (
    position.llx >= 0 &&
    position.lly >= 0 &&
    position.urx <= pageWidth &&
    position.ury <= pageHeight &&
    position.llx < position.urx &&
    position.lly < position.ury
  );
}
