/**
 * PDFPreview - PDF page rendering component using pdfjs-dist
 * Renders a single page of a PDF file with support for click position detection.
 */

import { useRef, useEffect, useState } from "react";
import * as pdfjsLib from "pdfjs-dist";
import { readFile } from "@tauri-apps/plugin-fs";

// Configure PDF.js worker
pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.mjs",
  import.meta.url
).toString();

export interface PageDimensions {
  width: number;
  height: number;
}

interface PDFPreviewProps {
  /** File path or URL to PDF */
  filePath: string;
  /** Current page number (1-indexed) */
  currentPage: number;
  /** Render scale (1.0 = 100%) */
  scale?: number;
  /** Callback with page dimensions after render */
  onPageLoad?: (dims: PageDimensions, totalPages: number) => void;
  /** Optional className for container */
  className?: string;
}

export function PDFPreview({
  filePath,
  currentPage,
  scale = 1.0,
  onPageLoad,
  className = "",
}: PDFPreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [pdfDoc, setPdfDoc] = useState<pdfjsLib.PDFDocumentProxy | null>(null);
  const [_pageDims, setPageDims] = useState<PageDimensions | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Load PDF document
  useEffect(() => {
    let isMounted = true;

    async function loadPdf() {
      setIsLoading(true);
      setError(null);

      try {
        // Read file as bytes using Tauri fs plugin
        const fileData = await readFile(filePath);

        // Convert Uint8Array to ArrayBuffer for pdfjs
        const loadingTask = pdfjsLib.getDocument({ data: fileData });
        const pdf = await loadingTask.promise;

        if (isMounted) {
          setPdfDoc(pdf);
        }
      } catch (err) {
        if (isMounted) {
          console.error("Failed to load PDF:", err);
          setError("Không thể tải file PDF");
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    }

    loadPdf();

    return () => {
      isMounted = false;
    };
  }, [filePath]);

  // Render current page
  useEffect(() => {
    if (!pdfDoc || !canvasRef.current) return;

    let isMounted = true;
    const currentPdfDoc = pdfDoc;

    async function renderPage() {
      try {
        const page = await currentPdfDoc.getPage(currentPage);
        const viewport = page.getViewport({ scale });

        const canvas = canvasRef.current!;
        const context = canvas.getContext("2d")!;

        canvas.height = viewport.height;
        canvas.width = viewport.width;

        await page.render({
          canvasContext: context,
          viewport: viewport,
          canvas: canvas,
        }).promise;

        if (isMounted) {
          // Get original page dimensions (without scale)
          const originalViewport = page.getViewport({ scale: 1.0 });
          const dims: PageDimensions = {
            width: originalViewport.width,
            height: originalViewport.height,
          };
          setPageDims(dims);
          onPageLoad?.(dims, currentPdfDoc.numPages);
        }
      } catch (err) {
        if (isMounted) {
          console.error("Failed to render page:", err);
          setError("Không thể hiển thị trang PDF");
        }
      }
    }

    renderPage();

    return () => {
      isMounted = false;
    };
  }, [pdfDoc, currentPage, scale, onPageLoad]);

  if (error) {
    return (
      <div className={`flex items-center justify-center bg-slate-100 dark:bg-slate-800 rounded-lg p-8 ${className}`}>
        <p className="text-red-500">{error}</p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className={`flex items-center justify-center bg-slate-100 dark:bg-slate-800 rounded-lg p-8 ${className}`}>
        <div className="flex flex-col items-center gap-3">
          <div className="w-8 h-8 border-2 border-ocean-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-slate-500 dark:text-slate-400">Đang tải PDF...</span>
        </div>
      </div>
    );
  }

  return (
    <div className={`relative ${className}`}>
      <canvas
        ref={canvasRef}
        className="shadow-lg rounded-sm"
        style={{ maxWidth: "100%", height: "auto" }}
      />
    </div>
  );
}
