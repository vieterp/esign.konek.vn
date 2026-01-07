/**
 * useSignatureTemplate - Signature template state management hook
 * Handles template customization with localStorage persistence.
 */

import { useState, useEffect, useCallback } from "react";
import {
  SignatureTemplate,
  DEFAULT_TEMPLATE,
  FontFamily,
  SignatureSizeKey,
  SIZE_PRESETS,
  SIGNATURE_WIDTH_MIN,
  SIGNATURE_WIDTH_MAX,
  SIGNATURE_HEIGHT_MIN,
  SIGNATURE_HEIGHT_MAX,
} from "../lib/signature-template-defaults";

const STORAGE_KEY = "esign-signature-template";

export interface UseSignatureTemplateState {
  template: SignatureTemplate;
  isLoaded: boolean;
}

export interface UseSignatureTemplateActions {
  /** Update font properties */
  updateFont: (updates: Partial<SignatureTemplate['font']>) => void;
  /** Update field visibility toggles */
  updateFields: (updates: Partial<SignatureTemplate['fields']>) => void;
  /** Apply a size preset (sets width, height, and sizeKey) */
  applyPreset: (sizeKey: SignatureSizeKey) => void;
  /** Set signature size (deprecated, use applyPreset) */
  setSizeKey: (sizeKey: SignatureSizeKey) => void;
  /** Set custom width */
  setWidth: (width: number) => void;
  /** Set custom height */
  setHeight: (height: number) => void;
  /** Set font family */
  setFontFamily: (family: FontFamily) => void;
  /** Set font size */
  setFontSize: (size: number) => void;
  /** Set font color */
  setFontColor: (color: string) => void;
  /** Toggle name field visibility */
  toggleShowName: () => void;
  /** Toggle timestamp field visibility */
  toggleShowTimestamp: () => void;
  /** Toggle reason field visibility */
  toggleShowReason: () => void;
  /** Reset to default template */
  reset: () => void;
}

/**
 * Load template from localStorage with fallback to default.
 */
function isValidSizeKey(key: unknown): key is SignatureSizeKey {
  return key === 'small' || key === 'medium' || key === 'large';
}

function loadTemplate(): SignatureTemplate {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      const parsed = JSON.parse(saved);
      // Merge with defaults to handle schema changes
      const sizeKey: SignatureSizeKey = isValidSizeKey(parsed.sizeKey)
        ? parsed.sizeKey
        : DEFAULT_TEMPLATE.sizeKey;
      // Use saved width/height or fall back to preset values
      const width = typeof parsed.width === 'number'
        ? parsed.width
        : SIZE_PRESETS[sizeKey].width;
      const height = typeof parsed.height === 'number'
        ? parsed.height
        : SIZE_PRESETS[sizeKey].height;
      return {
        ...DEFAULT_TEMPLATE,
        ...parsed,
        sizeKey,
        width,
        height,
        font: { ...DEFAULT_TEMPLATE.font, ...parsed.font },
        fields: { ...DEFAULT_TEMPLATE.fields, ...parsed.fields },
      };
    }
  } catch (error) {
    console.warn("Failed to load signature template from localStorage:", error);
  }
  return { ...DEFAULT_TEMPLATE };
}

/**
 * Save template to localStorage.
 */
function saveTemplate(template: SignatureTemplate): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(template));
  } catch (error) {
    console.warn("Failed to save signature template to localStorage:", error);
  }
}

export function useSignatureTemplate(): UseSignatureTemplateState & UseSignatureTemplateActions {
  const [template, setTemplate] = useState<SignatureTemplate>(() => loadTemplate());
  const [isLoaded, setIsLoaded] = useState(false);

  // Mark as loaded after initial render
  useEffect(() => {
    setIsLoaded(true);
  }, []);

  // Persist to localStorage on changes
  useEffect(() => {
    if (isLoaded) {
      saveTemplate(template);
    }
  }, [template, isLoaded]);

  const updateTemplate = useCallback((updater: (prev: SignatureTemplate) => SignatureTemplate) => {
    setTemplate(prev => {
      const updated = updater(prev);
      return {
        ...updated,
        updatedAt: new Date().toISOString(),
      };
    });
  }, []);

  const updateFont = useCallback((updates: Partial<SignatureTemplate['font']>) => {
    updateTemplate(prev => ({
      ...prev,
      font: { ...prev.font, ...updates },
    }));
  }, [updateTemplate]);

  const updateFields = useCallback((updates: Partial<SignatureTemplate['fields']>) => {
    updateTemplate(prev => ({
      ...prev,
      fields: { ...prev.fields, ...updates },
    }));
  }, [updateTemplate]);

  const applyPreset = useCallback((sizeKey: SignatureSizeKey) => {
    const preset = SIZE_PRESETS[sizeKey];
    updateTemplate(prev => ({
      ...prev,
      sizeKey,
      width: preset.width,
      height: preset.height,
    }));
  }, [updateTemplate]);

  // Deprecated: use applyPreset instead
  const setSizeKey = useCallback((sizeKey: SignatureSizeKey) => {
    applyPreset(sizeKey);
  }, [applyPreset]);

  const setWidth = useCallback((width: number) => {
    const clampedWidth = Math.max(SIGNATURE_WIDTH_MIN, Math.min(SIGNATURE_WIDTH_MAX, width));
    updateTemplate(prev => ({ ...prev, width: clampedWidth }));
  }, [updateTemplate]);

  const setHeight = useCallback((height: number) => {
    const clampedHeight = Math.max(SIGNATURE_HEIGHT_MIN, Math.min(SIGNATURE_HEIGHT_MAX, height));
    updateTemplate(prev => ({ ...prev, height: clampedHeight }));
  }, [updateTemplate]);

  const setFontFamily = useCallback((family: FontFamily) => {
    updateFont({ family });
  }, [updateFont]);

  const setFontSize = useCallback((size: number) => {
    // Clamp to valid range
    const clampedSize = Math.max(8, Math.min(16, size));
    updateFont({ size: clampedSize });
  }, [updateFont]);

  const setFontColor = useCallback((color: string) => {
    // Validate hex color format
    if (/^#[0-9A-Fa-f]{6}$/.test(color)) {
      updateFont({ color });
    }
  }, [updateFont]);

  const toggleShowName = useCallback(() => {
    updateFields({ showName: !template.fields.showName });
  }, [updateFields, template.fields.showName]);

  const toggleShowTimestamp = useCallback(() => {
    updateFields({ showTimestamp: !template.fields.showTimestamp });
  }, [updateFields, template.fields.showTimestamp]);

  const toggleShowReason = useCallback(() => {
    updateFields({ showReason: !template.fields.showReason });
  }, [updateFields, template.fields.showReason]);

  const reset = useCallback(() => {
    setTemplate({
      ...DEFAULT_TEMPLATE,
      updatedAt: new Date().toISOString(),
    });
  }, []);

  return {
    template,
    isLoaded,
    updateFont,
    updateFields,
    applyPreset,
    setSizeKey,
    setWidth,
    setHeight,
    setFontFamily,
    setFontSize,
    setFontColor,
    toggleShowName,
    toggleShowTimestamp,
    toggleShowReason,
    reset,
  };
}
