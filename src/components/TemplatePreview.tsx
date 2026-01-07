/**
 * TemplatePreview - Live preview of signature appearance
 * Renders signature text with current template styling.
 * Uses CSS pt units to match PDF pt units exactly.
 */

import { SignatureTemplate, getCssFontFamily } from "../lib/signature-template-defaults";
import { extractCommonName } from "../lib/certificate-utils";

interface TemplatePreviewProps {
  /** Current template configuration */
  template: SignatureTemplate;
  /** Signer name (certificate subject) to display */
  signerName?: string;
  /** Additional className */
  className?: string;
}

export function TemplatePreview({ template, signerName, className = "" }: TemplatePreviewProps) {
  // Use template width/height directly (in PDF points)
  // CSS pt unit = PDF pt unit, browser handles DPI conversion automatically
  const { width, height } = template;

  // Extract company name from certificate subject
  const companyName = signerName
    ? extractCommonName(signerName)
    : "CÔNG TY TNHH VIETERP";

  const fontFamily = getCssFontFamily(template.font.family);
  const { showName, showTimestamp } = template.fields;

  // If no fields are shown, display a placeholder
  const hasContent = showName || showTimestamp;

  return (
    <div className={`${className}`}>
      <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
        Xem trước ({width}×{height} pt)
      </label>
      <div
        className="border-2 bg-white overflow-hidden"
        style={{
          width: `${width}pt`,
          height: `${height}pt`,
          borderColor: template.font.color,
          padding: '4pt 6pt',
        }}
      >
        {hasContent ? (
          <div
            style={{
              fontFamily,
              fontSize: `${template.font.size}pt`,
              color: template.font.color,
              lineHeight: 1.3,
              overflow: 'hidden',
            }}
          >
            <div className="flex items-center gap-1 whitespace-nowrap">
              <span>Signature Valid</span>
              <svg className="w-3 h-3 text-green-500 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
              </svg>
            </div>
            {showName && (
              <div className="whitespace-nowrap">Được ký bởi: <span className="font-semibold">{companyName}</span></div>
            )}
            {showTimestamp && (
              <div className="whitespace-nowrap">Ngày ký: {formatCurrentDate()}</div>
            )}
          </div>
        ) : (
          <div className="text-slate-400 dark:text-slate-500 text-xs italic text-center h-full flex items-center justify-center">
            Chưa chọn nội dung
          </div>
        )}
      </div>
    </div>
  );
}

/** Format current date in Vietnamese format (DD/MM/YYYY) */
function formatCurrentDate(): string {
  const now = new Date();
  const day = String(now.getDate()).padStart(2, "0");
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const year = now.getFullYear();

  return `${day}/${month}/${year}`;
}
