/**
 * useSigning - PDF signing workflow hook
 * Manages file selection, signing process, and result handling
 */

import { useState, useCallback } from "react";
import { signPdf, selectPdfFile, SignResult } from "../lib/tauri";

export type SigningState =
  | "idle"
  | "selecting_file"
  | "file_selected"
  | "signing"
  | "success"
  | "error";

export interface SigningOptions {
  visible?: boolean;
  reason?: string;
  signerName?: string;
  page?: number;
}

export interface UseSigningState {
  state: SigningState;
  selectedFile: string | null;
  fileName: string | null;
  result: SignResult | null;
  error: string | null;
  isProcessing: boolean;
}

export interface UseSigningActions {
  selectFile: () => Promise<void>;
  setFile: (path: string) => void;
  sign: (options?: SigningOptions) => Promise<void>;
  reset: () => void;
  clearResult: () => void;
}

function getFileName(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] || path;
}

function getOutputPath(inputPath: string): string {
  const lastDot = inputPath.lastIndexOf(".");
  if (lastDot === -1) {
    return inputPath + "_signed";
  }
  return inputPath.substring(0, lastDot) + "_signed" + inputPath.substring(lastDot);
}

// Error mapping from English to Vietnamese
const ERROR_MAP: Record<string, string> = {
  "Not logged in": "Chưa đăng nhập token. Vui lòng nhập mã PIN.",
  "Token manager not initialized": "Chưa khởi tạo token. Vui lòng kết nối USB Token.",
  "file not found": "Không tìm thấy file PDF.",
  "No such file": "Không tìm thấy file PDF.",
  "permission denied": "Không có quyền truy cập file.",
  "PIN_INCORRECT": "Mã PIN không đúng.",
  "PIN incorrect": "Mã PIN không đúng.",
  "PIN_LOCKED": "Token đã bị khóa do nhập sai PIN quá nhiều lần.",
  "locked": "Token đã bị khóa.",
  "TOKEN_NOT_PRESENT": "Token không được kết nối.",
  "token not found": "Không tìm thấy USB Token.",
  "Certificate not found": "Không tìm thấy chứng thư số trên token.",
  "Private key not found": "Không tìm thấy khóa bí mật trên token.",
  "Signing failed": "Ký số thất bại.",
  "PDF error": "Lỗi xử lý file PDF.",
  "TSA error": "Lỗi kết nối máy chủ thời gian.",
  "PIN must be 4-16 characters": "Mã PIN phải từ 4-16 ký tự.",
  "PIN contains invalid characters": "Mã PIN chỉ được chứa chữ cái và số.",
  "Paths cannot be empty": "Đường dẫn file không được trống.",
  "Invalid page number": "Số trang không hợp lệ (phải từ 1-1000).",
  "Reason too long": "Lý do ký quá dài (tối đa 500 ký tự).",
  "Signer name too long": "Tên người ký quá dài (tối đa 200 ký tự).",
};

function mapErrorToVietnamese(error: string): string {
  const lowerError = error.toLowerCase();
  for (const [key, message] of Object.entries(ERROR_MAP)) {
    if (lowerError.includes(key.toLowerCase())) {
      return message;
    }
  }
  return error; // Return original if no match
}

export function useSigning(): UseSigningState & UseSigningActions {
  const [state, setState] = useState<SigningState>("idle");
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [result, setResult] = useState<SignResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  const selectFile = useCallback(async () => {
    setState("selecting_file");
    setIsProcessing(true);
    setError(null);

    try {
      const path = await selectPdfFile();
      if (path) {
        setSelectedFile(path);
        setFileName(getFileName(path));
        setState("file_selected");
      } else {
        // User cancelled
        setState(selectedFile ? "file_selected" : "idle");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setState("error");
    } finally {
      setIsProcessing(false);
    }
  }, [selectedFile]);

  const setFile = useCallback((path: string) => {
    setSelectedFile(path);
    setFileName(getFileName(path));
    setState("file_selected");
    setError(null);
    setResult(null);
  }, []);

  const sign = useCallback(async (options?: SigningOptions) => {
    if (!selectedFile) {
      setError("Chưa chọn file PDF");
      setState("error");
      return;
    }

    setState("signing");
    setIsProcessing(true);
    setError(null);
    setResult(null);

    try {
      const outputPath = getOutputPath(selectedFile);
      const signResult = await signPdf(
        selectedFile,
        outputPath,
        options?.visible ?? true,
        options?.reason,
        options?.signerName,
        options?.page
      );

      if (signResult.success) {
        setResult(signResult);
        setState("success");
      } else {
        setError(signResult.message || "Ký số thất bại");
        setState("error");
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(mapErrorToVietnamese(errorMsg));
      setState("error");
    } finally {
      setIsProcessing(false);
    }
  }, [selectedFile]);

  const reset = useCallback(() => {
    setState("idle");
    setSelectedFile(null);
    setFileName(null);
    setResult(null);
    setError(null);
    setIsProcessing(false);
  }, []);

  const clearResult = useCallback(() => {
    setResult(null);
    setError(null);
    if (state === "success" || state === "error") {
      setState(selectedFile ? "file_selected" : "idle");
    }
  }, [state, selectedFile]);

  return {
    state,
    selectedFile,
    fileName,
    result,
    error,
    isProcessing,
    selectFile,
    setFile,
    sign,
    reset,
    clearResult,
  };
}
