/**
 * useSignaturePosition - Signature position state management hook
 * Handles position selection with localStorage persistence.
 */

import { useState, useEffect, useCallback } from "react";
import { PdfPosition, DEFAULT_SIGNATURE_WIDTH, DEFAULT_SIGNATURE_HEIGHT } from "../lib/pdf-coordinates";

const STORAGE_KEY = "esign-signature-position";

/** Default signature position (bottom-right area of page 1) */
export const DEFAULT_POSITION: PdfPosition = {
  page: 1,
  llx: 350,
  lly: 50,
  urx: 350 + DEFAULT_SIGNATURE_WIDTH,
  ury: 50 + DEFAULT_SIGNATURE_HEIGHT,
};

export interface UseSignaturePositionState {
  position: PdfPosition;
  isLoaded: boolean;
  hasCustomPosition: boolean;
}

export interface UseSignaturePositionActions {
  /** Set complete position */
  setPosition: (position: PdfPosition) => void;
  /** Set page number */
  setPage: (page: number) => void;
  /** Update position coordinates (keeps page) */
  updateCoordinates: (coords: Omit<PdfPosition, 'page'>) => void;
  /** Reset to default position */
  reset: () => void;
  /** Clear position for new file */
  clear: () => void;
}

/**
 * Load position from localStorage with fallback to default.
 */
function loadPosition(): PdfPosition {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      const parsed = JSON.parse(saved);
      // Validate required fields
      if (
        typeof parsed.page === 'number' &&
        typeof parsed.llx === 'number' &&
        typeof parsed.lly === 'number' &&
        typeof parsed.urx === 'number' &&
        typeof parsed.ury === 'number'
      ) {
        return parsed;
      }
    }
  } catch (error) {
    console.warn("Failed to load signature position from localStorage:", error);
  }
  return { ...DEFAULT_POSITION };
}

/**
 * Save position to localStorage.
 */
function savePosition(position: PdfPosition): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(position));
  } catch (error) {
    console.warn("Failed to save signature position to localStorage:", error);
  }
}

export function useSignaturePosition(): UseSignaturePositionState & UseSignaturePositionActions {
  const [position, setPositionState] = useState<PdfPosition>(() => loadPosition());
  const [isLoaded, setIsLoaded] = useState(false);
  const [hasCustomPosition, setHasCustomPosition] = useState(false);

  // Mark as loaded after initial render
  useEffect(() => {
    setIsLoaded(true);
    // Check if position differs from default
    const isCustom = JSON.stringify(position) !== JSON.stringify(DEFAULT_POSITION);
    setHasCustomPosition(isCustom);
  }, []);

  // Persist to localStorage on changes
  useEffect(() => {
    if (isLoaded) {
      savePosition(position);
      const isCustom = JSON.stringify(position) !== JSON.stringify(DEFAULT_POSITION);
      setHasCustomPosition(isCustom);
    }
  }, [position, isLoaded]);

  const setPosition = useCallback((newPosition: PdfPosition) => {
    // Validate page number
    const validPage = Math.max(1, Math.round(newPosition.page));
    setPositionState({
      ...newPosition,
      page: validPage,
    });
  }, []);

  const setPage = useCallback((page: number) => {
    const validPage = Math.max(1, Math.round(page));
    setPositionState(prev => ({
      ...prev,
      page: validPage,
    }));
  }, []);

  const updateCoordinates = useCallback((coords: Omit<PdfPosition, 'page'>) => {
    setPositionState(prev => ({
      ...prev,
      ...coords,
    }));
  }, []);

  const reset = useCallback(() => {
    setPositionState({ ...DEFAULT_POSITION });
  }, []);

  const clear = useCallback(() => {
    setPositionState({ ...DEFAULT_POSITION });
    setHasCustomPosition(false);
  }, []);

  return {
    position,
    isLoaded,
    hasCustomPosition,
    setPosition,
    setPage,
    updateCoordinates,
    reset,
    clear,
  };
}
