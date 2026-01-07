/**
 * Tauri IPC wrapper for eSign Desktop
 * Type-safe wrappers for backend commands
 */

import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

// ============ Types ============

export interface DetectedLibrary {
  ca_name: string;
  path: string;
}

export interface TokenInfo {
  slot_id: number;
  label: string;
  manufacturer: string;
  model: string;
  serial: string;
  has_token: boolean;
}

export interface CertificateInfo {
  serial: string;
  subject: string;
  issuer: string;
  valid_from: string;
  valid_to: string;
  thumbprint: string;
  der_base64: string;
}

export interface TokenStatus {
  initialized: boolean;
  logged_in: boolean;
  library_path?: string;
  certificate?: CertificateInfo;
  detected_libraries?: DetectedLibrary[];
}

export interface SignResult {
  success: boolean;
  output_path: string;
  message: string;
  signing_time: string;
}

/** Signature position in PDF coordinates */
export interface PdfPosition {
  page: number;
  llx: number;
  lly: number;
  urx: number;
  ury: number;
}

/** Signature appearance customization */
export interface SignatureAppearance {
  /** Font family (maps to PDF font) */
  fontFamily: 'sans-serif' | 'serif' | 'handwriting';
  /** Font size in points */
  fontSize: number;
  /** Color in hex format (#RRGGBB) */
  colorHex: string;
  /** Show signer name */
  showName: boolean;
  /** Show timestamp */
  showTimestamp: boolean;
  /** Show signing reason */
  showReason: boolean;
}

export interface AppInfo {
  name: string;
  version: string;
  description: string;
}

// ============ App Commands ============

export async function getAppInfo(): Promise<AppInfo> {
  return invoke("get_app_info");
}

// ============ Token Commands ============

export async function detectLibraries(): Promise<DetectedLibrary[]> {
  return invoke("detect_libraries");
}

export async function initTokenManager(libraryPath: string): Promise<void> {
  return invoke("init_token_manager", { libraryPath });
}

export async function listTokens(): Promise<TokenInfo[]> {
  return invoke("list_tokens");
}

export async function loginToken(slotId: number, pin: string): Promise<void> {
  return invoke("login_token", { slotId, pin });
}

export async function getCertificate(): Promise<CertificateInfo> {
  return invoke("get_certificate");
}

export async function logoutToken(): Promise<void> {
  return invoke("logout_token");
}

export async function checkTokenStatus(): Promise<TokenStatus> {
  return invoke("check_token_status");
}

// ============ Signing Commands ============

export async function signPdf(
  pdfPath: string,
  outputPath: string,
  visible: boolean = true,
  reason?: string,
  signerName?: string,
  position?: PdfPosition,
  appearance?: SignatureAppearance
): Promise<SignResult> {
  return invoke("sign_pdf", {
    pdfPath,
    outputPath,
    visible,
    reason,
    signerName,
    page: position?.page,
    llx: position?.llx,
    lly: position?.lly,
    urx: position?.urx,
    ury: position?.ury,
    fontSize: appearance?.fontSize,
    colorRgb: appearance?.colorHex,
    showName: appearance?.showName,
    showTimestamp: appearance?.showTimestamp,
    showReason: appearance?.showReason,
  });
}

export async function signData(dataBase64: string): Promise<string> {
  return invoke("sign_data", { dataBase64 });
}

// ============ Dialog Helpers ============

export async function selectPdfFile(): Promise<string | null> {
  const result = await open({
    multiple: false,
    filters: [{ name: "PDF", extensions: ["pdf"] }],
    title: "Chọn file PDF để ký",
  });
  return result as string | null;
}

/** Open file with system default application */
export async function openFile(path: string): Promise<void> {
  return invoke("open_file", { path });
}

// ============ Settings ============

const SETTINGS_KEY = "esign-settings";

export interface Settings {
  libraryPath: string;
  lastUsedSlot?: number;
}

export function loadSettings(): Settings | null {
  try {
    const data = localStorage.getItem(SETTINGS_KEY);
    return data ? JSON.parse(data) : null;
  } catch {
    return null;
  }
}

export function saveSettings(settings: Settings): void {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
}

export function clearSettings(): void {
  localStorage.removeItem(SETTINGS_KEY);
}
