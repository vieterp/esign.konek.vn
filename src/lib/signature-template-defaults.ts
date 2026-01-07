/**
 * Signature Template Defaults and Constants
 * Defines default values, font options, and color presets for signature templates.
 */

/** Font family options for signature text */
export type FontFamily = 'sans-serif' | 'serif' | 'handwriting';

/** Signature size preset key */
export type SignatureSizeKey = 'small' | 'medium' | 'large';

/** Signature size dimensions */
export interface SignatureSize {
  width: number;  // PDF points
  height: number; // PDF points
  label: string;
}

/** Signature template configuration */
export interface SignatureTemplate {
  /** Unique template identifier */
  id: string;
  /** User-friendly template name */
  name: string;
  /** Signature size key (for presets) */
  sizeKey: SignatureSizeKey;
  /** Custom signature width in PDF points */
  width: number;
  /** Custom signature height in PDF points */
  height: number;
  /** Font configuration */
  font: {
    /** Font family type */
    family: FontFamily;
    /** Font size in points (8-16) */
    size: number;
    /** Font color in hex format (#RRGGBB) */
    color: string;
  };
  /** Field visibility toggles */
  fields: {
    /** Show signer name */
    showName: boolean;
    /** Show signing timestamp */
    showTimestamp: boolean;
    /** Show signing reason */
    showReason: boolean;
  };
  /** Optional image signature (Phase 3+) */
  image?: {
    /** Base64-encoded PNG/JPG data URL */
    base64: string;
    /** Image width in pixels */
    width: number;
    /** Image height in pixels */
    height: number;
  };
  /** Template creation timestamp */
  createdAt: string;
  /** Last update timestamp */
  updatedAt: string;
}

/** Available signature size presets */
export const SIZE_PRESETS: Record<SignatureSizeKey, SignatureSize> = {
  small: { width: 150, height: 50, label: 'Nhỏ (150×50)' },
  medium: { width: 200, height: 60, label: 'Vừa (200×60)' },
  large: { width: 280, height: 80, label: 'Lớn (280×80)' },
} as const;

/** Get signature size from key */
export function getSignatureSize(key: SignatureSizeKey): SignatureSize {
  return SIZE_PRESETS[key];
}

/** Get signature size from template (uses custom width/height) */
export function getTemplateSizeInfo(template: SignatureTemplate): { width: number; height: number } {
  return { width: template.width, height: template.height };
}

/** Default signature template */
export const DEFAULT_TEMPLATE: SignatureTemplate = {
  id: 'default',
  name: 'Mặc định',
  sizeKey: 'medium',
  width: 200,  // Medium preset width
  height: 60,  // Medium preset height
  font: {
    family: 'sans-serif',
    size: 10,
    color: '#dc2626', // Red as default
  },
  fields: {
    showName: true,
    showTimestamp: true,
    showReason: false,
  },
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
};

/** Available font family options */
export const FONT_OPTIONS: ReadonlyArray<{
  value: FontFamily;
  label: string;
  cssFontFamily: string;
}> = [
  { value: 'sans-serif', label: 'Sans Serif', cssFontFamily: 'Arial, Helvetica, sans-serif' },
  { value: 'serif', label: 'Serif', cssFontFamily: 'Georgia, "Times New Roman", serif' },
  { value: 'handwriting', label: 'Chữ viết tay', cssFontFamily: '"Segoe Script", cursive' },
] as const;

/** Preset color options for signature */
export const COLOR_PRESETS: ReadonlyArray<{
  value: string;
  label: string;
}> = [
  { value: '#dc2626', label: 'Đỏ' },
  { value: '#000000', label: 'Đen' },
  { value: '#1a365d', label: 'Xanh Navy' },
  { value: '#1e40af', label: 'Xanh dương' },
  { value: '#065f46', label: 'Xanh lá' },
] as const;

/** Font size constraints */
export const FONT_SIZE_MIN = 8;
export const FONT_SIZE_MAX = 16;
export const FONT_SIZE_DEFAULT = 10;

/** Signature size constraints */
export const SIGNATURE_WIDTH_MIN = 100;
export const SIGNATURE_WIDTH_MAX = 400;
export const SIGNATURE_HEIGHT_MIN = 30;
export const SIGNATURE_HEIGHT_MAX = 150;

/** Get CSS font-family for template font */
export function getCssFontFamily(family: FontFamily): string {
  const option = FONT_OPTIONS.find(f => f.value === family);
  return option?.cssFontFamily ?? 'sans-serif';
}

/** Validate hex color format */
export function isValidHexColor(color: string): boolean {
  return /^#[0-9A-Fa-f]{6}$/.test(color);
}
