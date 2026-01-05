/**
 * useToken - Token state management hook
 * Manages USB token connection, login, and certificate state
 */

import { useState, useCallback, useEffect } from "react";
import {
  TokenInfo,
  CertificateInfo,
  DetectedLibrary,
  checkTokenStatus,
  detectLibraries,
  initTokenManager,
  listTokens,
  loginToken,
  logoutToken,
  getCertificate,
  loadSettings,
  saveSettings,
} from "../lib/tauri";

export type ConnectionState =
  | "disconnected"
  | "detecting"
  | "library_found"
  | "initializing"
  | "ready"
  | "logging_in"
  | "logged_in"
  | "error";

export interface UseTokenState {
  connectionState: ConnectionState;
  error: string | null;
  detectedLibraries: DetectedLibrary[];
  selectedLibrary: DetectedLibrary | null;
  tokens: TokenInfo[];
  selectedSlot: number | null;
  certificate: CertificateInfo | null;
  isLoading: boolean;
}

export interface UseTokenActions {
  detectAndInit: () => Promise<void>;
  selectLibrary: (library: DetectedLibrary) => Promise<void>;
  login: (pin: string) => Promise<void>;
  logout: () => Promise<void>;
  refresh: () => Promise<void>;
}

export function useToken(): UseTokenState & UseTokenActions {
  const [connectionState, setConnectionState] = useState<ConnectionState>("disconnected");
  const [error, setError] = useState<string | null>(null);
  const [detectedLibraries, setDetectedLibraries] = useState<DetectedLibrary[]>([]);
  const [selectedLibrary, setSelectedLibrary] = useState<DetectedLibrary | null>(null);
  const [tokens, setTokens] = useState<TokenInfo[]>([]);
  const [selectedSlot, setSelectedSlot] = useState<number | null>(null);
  const [certificate, setCertificate] = useState<CertificateInfo | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // Define selectLibrary BEFORE detectAndInit to avoid circular dependency
  const selectLibrary = useCallback(async (library: DetectedLibrary) => {
    setIsLoading(true);
    setError(null);
    setConnectionState("initializing");

    try {
      await initTokenManager(library.path);
      setSelectedLibrary(library);

      const tokenList = await listTokens();
      setTokens(tokenList);

      // Find first slot with token
      const slotWithToken = tokenList.find(t => t.has_token);
      if (slotWithToken) {
        setSelectedSlot(slotWithToken.slot_id);
      }

      // Save settings
      saveSettings({
        libraryPath: library.path,
        lastUsedSlot: slotWithToken?.slot_id
      });

      setConnectionState("ready");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setConnectionState("error");
    } finally {
      setIsLoading(false);
    }
  }, []); // No deps needed - uses only setters which are stable

  const detectAndInit = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    setConnectionState("detecting");

    try {
      // Check if already initialized
      const status = await checkTokenStatus();

      if (status.initialized && status.logged_in && status.certificate) {
        setCertificate(status.certificate);
        setConnectionState("logged_in");
        setIsLoading(false);
        return;
      }

      if (status.initialized) {
        // Already initialized, list tokens
        const tokenList = await listTokens();
        setTokens(tokenList);

        // Auto-select first slot with token
        const slotWithToken = tokenList.find(t => t.has_token);
        if (slotWithToken) {
          setSelectedSlot(slotWithToken.slot_id);
        }

        setConnectionState("ready");
        setIsLoading(false);
        return;
      }

      // Try to load saved settings
      const settings = loadSettings();
      if (settings?.libraryPath) {
        try {
          await initTokenManager(settings.libraryPath);
          const tokenList = await listTokens();
          setTokens(tokenList);

          // Use saved slot if valid, otherwise auto-select first available
          const savedSlotExists = tokenList.some(t => t.slot_id === settings.lastUsedSlot);
          if (savedSlotExists) {
            setSelectedSlot(settings.lastUsedSlot!);
          } else {
            const slotWithToken = tokenList.find(t => t.has_token);
            if (slotWithToken) {
              setSelectedSlot(slotWithToken.slot_id);
            }
          }

          setConnectionState("ready");
          setIsLoading(false);
          return;
        } catch {
          // Saved library no longer available, continue detection
        }
      }

      // Detect available libraries
      const libraries = await detectLibraries();
      setDetectedLibraries(libraries);

      if (libraries.length === 0) {
        setError("Không tìm thấy thư viện PKCS#11. Vui lòng cài đặt driver USB Token.");
        setConnectionState("error");
        setIsLoading(false);
        return;
      }

      // Auto-select first library
      setConnectionState("library_found");
      await selectLibrary(libraries[0]);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setConnectionState("error");
    } finally {
      setIsLoading(false);
    }
  }, [selectLibrary]); // Now properly includes selectLibrary

  // Auto-detect on mount
  useEffect(() => {
    detectAndInit();
  }, [detectAndInit]);

  const login = useCallback(async (pin: string) => {
    if (selectedSlot === null) {
      setError("Chưa chọn slot token");
      return;
    }

    setIsLoading(true);
    setError(null);
    setConnectionState("logging_in");

    try {
      await loginToken(selectedSlot, pin);
      const cert = await getCertificate();
      setCertificate(cert);
      setConnectionState("logged_in");

      // Update saved slot
      const settings = loadSettings();
      if (settings) {
        saveSettings({ ...settings, lastUsedSlot: selectedSlot });
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      // Map PKCS#11 errors to Vietnamese
      if (errorMsg.includes("PIN_INCORRECT") || errorMsg.includes("PIN incorrect")) {
        setError("Mã PIN không đúng");
      } else if (errorMsg.includes("PIN_LOCKED") || errorMsg.includes("locked")) {
        setError("Token đã bị khóa do nhập sai PIN quá nhiều lần");
      } else if (errorMsg.includes("TOKEN_NOT_PRESENT")) {
        setError("Token không được kết nối");
      } else {
        setError(errorMsg);
      }
      setConnectionState("ready");
    } finally {
      setIsLoading(false);
    }
  }, [selectedSlot]);

  const logout = useCallback(async () => {
    setIsLoading(true);
    try {
      await logoutToken();
      setCertificate(null);
      setConnectionState("ready");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  const refresh = useCallback(async () => {
    await detectAndInit();
  }, [detectAndInit]);

  return {
    connectionState,
    error,
    detectedLibraries,
    selectedLibrary,
    tokens,
    selectedSlot,
    certificate,
    isLoading,
    detectAndInit,
    selectLibrary,
    login,
    logout,
    refresh,
  };
}
