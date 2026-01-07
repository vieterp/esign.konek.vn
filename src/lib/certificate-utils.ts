/**
 * Certificate utility functions
 * Parse and extract fields from X.509 certificate subject strings
 */

/**
 * Extract Common Name (CN) from certificate subject string.
 * Example input: "C=VN, L=ĐÀ NẴNG, CN=CÔNG TY TNHH VIETERP, 0.9.2342..."
 * Example output: "CÔNG TY TNHH VIETERP"
 */
export function extractCommonName(subject: string): string {
  // Try to find CN= field
  const cnMatch = subject.match(/CN=([^,]+)/i);
  if (cnMatch && cnMatch[1]) {
    return cnMatch[1].trim();
  }

  // Fallback: return first meaningful part before any OID
  const parts = subject.split(',');
  for (const part of parts) {
    const trimmed = part.trim();
    // Skip country, locality, and OID fields
    if (
      !trimmed.startsWith('C=') &&
      !trimmed.startsWith('L=') &&
      !trimmed.startsWith('O=') &&
      !trimmed.startsWith('OU=') &&
      !trimmed.match(/^\d+\./)
    ) {
      return trimmed;
    }
  }

  return subject;
}

/**
 * Format timestamp for signature display.
 * Returns: "HH:mm:ss DD/MM/YYYY"
 */
export function formatSigningTime(date: Date = new Date()): string {
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const year = date.getFullYear();

  return `${hours}:${minutes}:${seconds} ${day}/${month}/${year}`;
}
