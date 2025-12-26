/**
 * PinInput - Secure PIN entry component
 * Handles PIN input with visibility toggle and submit
 */

import { useState, useCallback, FormEvent, KeyboardEvent, ChangeEvent } from "react";

// PIN validation constants
const PIN_MIN_LENGTH = 4;
const PIN_MAX_LENGTH = 16;
const PIN_PATTERN = /^[a-zA-Z0-9]+$/;

/**
 * Validate PIN format
 * @returns Error message or null if valid
 */
function validatePin(pin: string): string | null {
  if (pin.length < PIN_MIN_LENGTH) {
    return `PIN phải có ít nhất ${PIN_MIN_LENGTH} ký tự`;
  }
  if (pin.length > PIN_MAX_LENGTH) {
    return `PIN không được quá ${PIN_MAX_LENGTH} ký tự`;
  }
  if (!PIN_PATTERN.test(pin)) {
    return "PIN chỉ được chứa chữ cái và số";
  }
  return null;
}

interface PinInputProps {
  onSubmit: (pin: string) => void;
  disabled?: boolean;
  isLoading?: boolean;
  error?: string | null;
}

export function PinInput({
  onSubmit,
  disabled = false,
  isLoading = false,
  error,
}: PinInputProps) {
  const [pin, setPin] = useState("");
  const [showPin, setShowPin] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  // Filter non-alphanumeric characters on input
  const handlePinChange = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    // Only allow alphanumeric characters
    const filtered = value.replace(/[^a-zA-Z0-9]/g, "");
    setPin(filtered);
    // Clear validation error on typing
    if (validationError) {
      setValidationError(null);
    }
  }, [validationError]);

  const handleSubmit = useCallback((e: FormEvent) => {
    e.preventDefault();
    const pinError = validatePin(pin);
    if (pinError) {
      setValidationError(pinError);
      return;
    }
    if (!disabled && !isLoading) {
      onSubmit(pin);
    }
  }, [pin, disabled, isLoading, onSubmit]);

  const handleKeyDown = useCallback((e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      const pinError = validatePin(pin);
      if (pinError) {
        setValidationError(pinError);
        return;
      }
      onSubmit(pin);
    }
  }, [pin, onSubmit]);

  const toggleShowPin = useCallback(() => {
    setShowPin(prev => !prev);
  }, []);

  return (
    <div className="bg-white dark:bg-slate-800 rounded-xl shadow-sm p-4">
      <form onSubmit={handleSubmit} className="space-y-3">
        <label className="block text-sm font-medium text-slate-700 dark:text-slate-200">
          Mã PIN
        </label>

        <div className="relative">
          <input
            type={showPin ? "text" : "password"}
            value={pin}
            onChange={handlePinChange}
            onKeyDown={handleKeyDown}
            disabled={disabled || isLoading}
            placeholder="Nhập mã PIN token"
            maxLength={PIN_MAX_LENGTH}
            autoComplete="off"
            className={`
              w-full px-4 py-3 pr-20 rounded-lg
              bg-slate-50 dark:bg-slate-900
              border ${error || validationError ? "border-red-500" : "border-slate-200 dark:border-slate-700"}
              text-slate-800 dark:text-slate-100
              placeholder:text-slate-400
              focus:outline-none focus:ring-2 focus:ring-ocean-500 focus:border-transparent
              disabled:opacity-50 disabled:cursor-not-allowed
              transition-colors
            `}
          />

          <button
            type="button"
            onClick={toggleShowPin}
            disabled={disabled || isLoading}
            className="absolute right-12 top-1/2 -translate-y-1/2 p-1.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 disabled:opacity-50"
            title={showPin ? "Ẩn PIN" : "Hiện PIN"}
          >
            {showPin ? (
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
                />
              </svg>
            ) : (
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                />
              </svg>
            )}
          </button>

          <button
            type="submit"
            disabled={pin.length < PIN_MIN_LENGTH || disabled || isLoading}
            className="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-ocean-500 hover:text-ocean-600 disabled:text-slate-300 disabled:cursor-not-allowed transition-colors"
            title="Đăng nhập"
          >
            {isLoading ? (
              <svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle
                  className="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  strokeWidth="4"
                />
                <path
                  className="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
            ) : (
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M14 5l7 7m0 0l-7 7m7-7H3"
                />
              </svg>
            )}
          </button>
        </div>

        {(error || validationError) && (
          <p className="text-sm text-red-500">{validationError || error}</p>
        )}

        <p className="text-xs text-slate-500 dark:text-slate-400">
          Nhập mã PIN của USB Token để đăng nhập ({PIN_MIN_LENGTH}-{PIN_MAX_LENGTH} ký tự, chữ và số)
        </p>
      </form>
    </div>
  );
}
