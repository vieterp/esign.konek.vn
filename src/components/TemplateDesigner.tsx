/**
 * TemplateDesigner - Signature template customization panel
 * Allows users to customize font, color, size, and field visibility.
 */

import { useCallback } from "react";
import {
  SignatureTemplate,
  SignatureSizeKey,
  COLOR_PRESETS,
  SIZE_PRESETS,
  FONT_SIZE_MIN,
  FONT_SIZE_MAX,
  SIGNATURE_WIDTH_MIN,
  SIGNATURE_WIDTH_MAX,
  SIGNATURE_HEIGHT_MIN,
  SIGNATURE_HEIGHT_MAX,
  isValidHexColor,
} from "../lib/signature-template-defaults";
import { TemplatePreview } from "./TemplatePreview";

interface TemplateDesignerProps {
  /** Current template configuration */
  template: SignatureTemplate;
  /** Signer name for preview */
  signerName?: string;
  /** Callback to apply a size preset */
  onPresetChange: (sizeKey: SignatureSizeKey) => void;
  /** Callback to update custom width */
  onWidthChange: (width: number) => void;
  /** Callback to update custom height */
  onHeightChange: (height: number) => void;
  /** Callback to update font size */
  onFontSizeChange: (size: number) => void;
  /** Callback to update font color */
  onFontColorChange: (color: string) => void;
  /** Callback to toggle name visibility */
  onToggleName: () => void;
  /** Callback to toggle timestamp visibility */
  onToggleTimestamp: () => void;
  /** Callback to reset to defaults */
  onReset: () => void;
  /** Additional className */
  className?: string;
}

export function TemplateDesigner({
  template,
  signerName,
  onPresetChange,
  onWidthChange,
  onHeightChange,
  onFontSizeChange,
  onFontColorChange,
  onToggleName,
  onToggleTimestamp,
  onReset,
  className = "",
}: TemplateDesignerProps) {
  return (
    <div className={`space-y-6 ${className}`}>
      {/* Preview */}
      <TemplatePreview
        template={template}
        signerName={signerName}
      />

        {/* Signature Size */}
        <SignatureSizeSection
          width={template.width}
          height={template.height}
          onPresetChange={onPresetChange}
          onWidthChange={onWidthChange}
          onHeightChange={onHeightChange}
        />

        {/* Font Size */}
        <FontSizeSlider
          value={template.font.size}
          onChange={onFontSizeChange}
        />

        {/* Color Picker */}
        <ColorPicker
          value={template.font.color}
          onChange={onFontColorChange}
        />

        {/* Field Toggles */}
        <FieldToggles
          showName={template.fields.showName}
          showTimestamp={template.fields.showTimestamp}
          onToggleName={onToggleName}
          onToggleTimestamp={onToggleTimestamp}
        />

      {/* Reset button */}
      <button
        onClick={onReset}
        className="w-full py-2 text-sm text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-700 rounded-lg transition-colors"
      >
        Đặt lại mặc định
      </button>
    </div>
  );
}

// ============ Subcomponents ============

interface SignatureSizeSectionProps {
  width: number;
  height: number;
  onPresetChange: (sizeKey: SignatureSizeKey) => void;
  onWidthChange: (width: number) => void;
  onHeightChange: (height: number) => void;
}

function SignatureSizeSection({
  width,
  height,
  onPresetChange,
  onWidthChange,
  onHeightChange,
}: SignatureSizeSectionProps) {
  const handleWidthChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    onWidthChange(parseInt(e.target.value, 10));
  }, [onWidthChange]);

  const handleHeightChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    onHeightChange(parseInt(e.target.value, 10));
  }, [onHeightChange]);

  return (
    <div className="space-y-3">
      <label className="block text-sm font-medium text-slate-700 dark:text-slate-300">
        Kích thước chữ ký
      </label>

      {/* Preset buttons */}
      <div className="flex gap-2">
        {(Object.keys(SIZE_PRESETS) as SignatureSizeKey[]).map((key) => {
          const preset = SIZE_PRESETS[key];
          const isActive = width === preset.width && height === preset.height;
          return (
            <button
              key={key}
              onClick={() => onPresetChange(key)}
              className={`flex-1 py-1.5 px-2 text-xs rounded-lg border-2 transition-colors ${
                isActive
                  ? "border-ocean-500 bg-ocean-50 dark:bg-ocean-900/20 text-ocean-600 dark:text-ocean-400"
                  : "border-slate-300 dark:border-slate-600 text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-700"
              }`}
            >
              {key === 'small' ? 'Nhỏ' : key === 'medium' ? 'Vừa' : 'Lớn'}
            </button>
          );
        })}
      </div>

      {/* Width slider */}
      <div>
        <div className="flex justify-between text-xs text-slate-500 mb-1">
          <span>Chiều rộng: {width}pt</span>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-xs text-slate-500 w-8">{SIGNATURE_WIDTH_MIN}</span>
          <input
            type="range"
            min={SIGNATURE_WIDTH_MIN}
            max={SIGNATURE_WIDTH_MAX}
            step={5}
            value={width}
            onChange={handleWidthChange}
            className="flex-1 h-2 bg-slate-200 dark:bg-slate-700 rounded-lg appearance-none cursor-pointer accent-ocean-500"
          />
          <span className="text-xs text-slate-500 w-8">{SIGNATURE_WIDTH_MAX}</span>
        </div>
      </div>

      {/* Height slider */}
      <div>
        <div className="flex justify-between text-xs text-slate-500 mb-1">
          <span>Chiều cao: {height}pt</span>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-xs text-slate-500 w-8">{SIGNATURE_HEIGHT_MIN}</span>
          <input
            type="range"
            min={SIGNATURE_HEIGHT_MIN}
            max={SIGNATURE_HEIGHT_MAX}
            step={5}
            value={height}
            onChange={handleHeightChange}
            className="flex-1 h-2 bg-slate-200 dark:bg-slate-700 rounded-lg appearance-none cursor-pointer accent-ocean-500"
          />
          <span className="text-xs text-slate-500 w-8">{SIGNATURE_HEIGHT_MAX}</span>
        </div>
      </div>
    </div>
  );
}

interface FontSizeSliderProps {
  value: number;
  onChange: (value: number) => void;
}

function FontSizeSlider({ value, onChange }: FontSizeSliderProps) {
  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(parseInt(e.target.value, 10));
  }, [onChange]);

  return (
    <div>
      <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
        Cỡ chữ: {value}pt
      </label>
      <div className="flex items-center gap-3">
        <span className="text-xs text-slate-500">{FONT_SIZE_MIN}</span>
        <input
          type="range"
          min={FONT_SIZE_MIN}
          max={FONT_SIZE_MAX}
          step={1}
          value={value}
          onChange={handleChange}
          className="flex-1 h-2 bg-slate-200 dark:bg-slate-700 rounded-lg appearance-none cursor-pointer accent-ocean-500"
        />
        <span className="text-xs text-slate-500">{FONT_SIZE_MAX}</span>
      </div>
    </div>
  );
}

interface ColorPickerProps {
  value: string;
  onChange: (value: string) => void;
}

function ColorPicker({ value, onChange }: ColorPickerProps) {
  const handlePresetClick = useCallback((color: string) => {
    onChange(color);
  }, [onChange]);

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const newColor = e.target.value;
    if (isValidHexColor(newColor)) {
      onChange(newColor);
    }
  }, [onChange]);

  return (
    <div>
      <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
        Màu chữ
      </label>

      {/* Preset colors */}
      <div className="flex flex-wrap gap-2 mb-3">
        {COLOR_PRESETS.map((preset) => (
          <button
            key={preset.value}
            onClick={() => handlePresetClick(preset.value)}
            className={`
              w-8 h-8 rounded-full border-2 transition-transform hover:scale-110
              ${value === preset.value
                ? "border-ocean-500 ring-2 ring-ocean-500/30"
                : "border-slate-300 dark:border-slate-600"
              }
            `}
            style={{ backgroundColor: preset.value }}
            title={preset.label}
            aria-label={preset.label}
          />
        ))}
      </div>

      {/* Custom color input */}
      <div className="flex items-center gap-2">
        <input
          type="color"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="w-10 h-10 rounded border-0 cursor-pointer"
        />
        <input
          type="text"
          value={value}
          onChange={handleInputChange}
          placeholder="#000000"
          className="flex-1 px-3 py-2 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded-lg text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-2 focus:ring-ocean-500 font-mono text-sm"
        />
      </div>
    </div>
  );
}

interface FieldTogglesProps {
  showName: boolean;
  showTimestamp: boolean;
  onToggleName: () => void;
  onToggleTimestamp: () => void;
}

function FieldToggles({
  showName,
  showTimestamp,
  onToggleName,
  onToggleTimestamp,
}: FieldTogglesProps) {
  return (
    <div>
      <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
        Hiển thị
      </label>
      <div className="space-y-2">
        <ToggleItem
          label="Được ký bởi: [Tên công ty]"
          checked={showName}
          onChange={onToggleName}
        />
        <ToggleItem
          label="Ngày ký: [DD/MM/YYYY]"
          checked={showTimestamp}
          onChange={onToggleTimestamp}
        />
      </div>
    </div>
  );
}

interface ToggleItemProps {
  label: string;
  checked: boolean;
  onChange: () => void;
}

function ToggleItem({ label, checked, onChange }: ToggleItemProps) {
  return (
    <label className="flex items-center gap-3 cursor-pointer">
      <div className="relative">
        <input
          type="checkbox"
          checked={checked}
          onChange={onChange}
          className="sr-only peer"
        />
        <div className="w-10 h-5 bg-slate-200 dark:bg-slate-700 rounded-full peer-checked:bg-ocean-500 transition-colors" />
        <div className="absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full shadow-sm peer-checked:translate-x-5 transition-transform" />
      </div>
      <span className="text-sm text-slate-700 dark:text-slate-300">{label}</span>
    </label>
  );
}
