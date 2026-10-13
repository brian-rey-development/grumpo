export type Rgb = readonly [number, number, number];

export interface Rgba {
  rgb: Rgb;
  alpha: number;
}

const LUMA = { red: 0.2126, green: 0.7152, blue: 0.0722 } as const;
const CONTRAST_OFFSET = 0.05;
const GAMMA_THRESHOLD = 0.04045;
const GAMMA_LINEAR_SLOPE = 12.92;
const GAMMA_OFFSET = 0.055;
const GAMMA_SCALE = 1.055;
const GAMMA_EXPONENT = 2.4;
const BYTE_MAX = 255;
const HEX_RADIX = 16;
const GAMUT_EPSILON = 1e-4;

const HEX_COLOR = /^#([0-9a-f]{3,8})$/i;
const RGB_COLOR = /^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)\s*(?:,\s*([\d.]+)\s*)?\)$/i;
const OKLCH_COLOR = /^oklch\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)\s*\)$/i;

function srgbToLinear(channel: number): number {
  if (channel <= GAMMA_THRESHOLD) return channel / GAMMA_LINEAR_SLOPE;
  return ((channel + GAMMA_OFFSET) / GAMMA_SCALE) ** GAMMA_EXPONENT;
}

function linearToSrgb(channel: number): number {
  const clamped = Math.min(Math.max(channel, 0), 1);
  if (clamped <= GAMMA_THRESHOLD / GAMMA_LINEAR_SLOPE) return clamped * GAMMA_LINEAR_SLOPE;
  return GAMMA_SCALE * clamped ** (1 / GAMMA_EXPONENT) - GAMMA_OFFSET;
}

export function relativeLuminance([red, green, blue]: Rgb): number {
  return (
    LUMA.red * srgbToLinear(red) + LUMA.green * srgbToLinear(green) + LUMA.blue * srgbToLinear(blue)
  );
}

export function contrastRatio(first: Rgb, second: Rgb): number {
  const a = relativeLuminance(first);
  const b = relativeLuminance(second);
  return (Math.max(a, b) + CONTRAST_OFFSET) / (Math.min(a, b) + CONTRAST_OFFSET);
}

export function compositeOver(foreground: Rgba, background: Rgb): Rgb {
  const { rgb, alpha } = foreground;
  return [
    rgb[0] * alpha + background[0] * (1 - alpha),
    rgb[1] * alpha + background[1] * (1 - alpha),
    rgb[2] * alpha + background[2] * (1 - alpha),
  ];
}

export function oklchToLinearSrgb(lightness: number, chroma: number, hueDegrees: number): Rgb {
  const hue = (hueDegrees * Math.PI) / 180;
  const a = chroma * Math.cos(hue);
  const b = chroma * Math.sin(hue);
  const l = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (lightness - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ];
}

export function isInSrgbGamut(linear: Rgb): boolean {
  return linear.every((channel) => channel >= -GAMUT_EPSILON && channel <= 1 + GAMUT_EPSILON);
}

function parseHex(digits: string): Rgba | null {
  const full = digits.length <= 4 ? Array.from(digits, (digit) => digit + digit).join('') : digits;
  if (full.length !== 6 && full.length !== 8) return null;
  const channel = (offset: number) =>
    parseInt(full.slice(offset, offset + 2), HEX_RADIX) / BYTE_MAX;
  const alpha = full.length === 8 ? channel(6) : 1;
  return { rgb: [channel(0), channel(2), channel(4)], alpha };
}

export function parseCssColor(value: string): Rgba | null {
  const text = value.trim();
  const hex = HEX_COLOR.exec(text);
  if (hex?.[1] !== undefined) return parseHex(hex[1]);
  const rgb = RGB_COLOR.exec(text);
  if (rgb?.[1] !== undefined && rgb[2] !== undefined && rgb[3] !== undefined) {
    const channels = [rgb[1], rgb[2], rgb[3]].map((part) => Number(part) / BYTE_MAX);
    const [red = 0, green = 0, blue = 0] = channels;
    return { rgb: [red, green, blue], alpha: rgb[4] === undefined ? 1 : Number(rgb[4]) };
  }
  const oklch = OKLCH_COLOR.exec(text);
  if (oklch?.[1] !== undefined && oklch[2] !== undefined && oklch[3] !== undefined) {
    const linear = oklchToLinearSrgb(Number(oklch[1]), Number(oklch[2]), Number(oklch[3]));
    const [red = 0, green = 0, blue = 0] = linear.map(linearToSrgb);
    return { rgb: [red, green, blue], alpha: 1 };
  }
  return null;
}

export function pickOnColor(background: Rgb, candidates: readonly [Rgb, Rgb]): Rgb {
  const [first, second] = candidates;
  return contrastRatio(first, background) >= contrastRatio(second, background) ? first : second;
}

const HALF_TURN_DEGREES = 180;
const FULL_TURN_DEGREES = 360;

export function hueDegrees([red, green, blue]: Rgb): number {
  const hue = Math.atan2(Math.sqrt(3) * (green - blue), 2 * red - green - blue);
  return ((hue * HALF_TURN_DEGREES) / Math.PI + FULL_TURN_DEGREES) % FULL_TURN_DEGREES;
}
