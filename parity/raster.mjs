// Rasterises PDFs with poppler's pdftoppm and compares pages pixel by pixel, with no image library:
// pages come back as PPM (a header and raw RGB) and diffs are written as PNG with node:zlib.
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readdirSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { crc32, deflateSync } from 'node:zlib';

// Every page of a PDF as { width, height, rgb }.
export function rasterise(pdf, dpi) {
  const dir = mkdtempSync(join(tmpdir(), 'axonpdf-raster-'));
  try {
    execFileSync('pdftoppm', ['-r', String(dpi), pdf, join(dir, 'p')]);
    return readdirSync(dir)
      .sort((a, b) => a.localeCompare(b, 'en', { numeric: true }))
      .map((f) => readPpm(readFileSync(join(dir, f))));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function readPpm(buf) {
  const [header, , width, height] = buf.toString('latin1', 0, 64).match(/^(P6)\s+(\d+)\s+(\d+)\s+255\s/);
  return { width: +width, height: +height, rgb: buf.subarray(header.length) };
}

// Share of pixels (0 to 100) whose channels differ by more than `tolerance`, and a PNG with the
// reference, ours, and the differences in red over a faded reference, side by side.
export function diffPages(ref, ours, tolerance = 48) {
  const width = ref.width + ours.width + Math.max(ref.width, ours.width);
  const height = Math.max(ref.height, ours.height);
  const out = Buffer.alloc(width * height * 3, 255);
  const put = (x, y, r, g, b) => out.set([r, g, b], (y * width + x) * 3);
  const at = (img, x, y) => (x < img.width && y < img.height ? img.rgb.subarray((y * img.width + x) * 3, (y * img.width + x) * 3 + 3) : [255, 255, 255]);
  let changed = 0;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < Math.max(ref.width, ours.width); x++) {
      const a = at(ref, x, y);
      const b = at(ours, x, y);
      if (x < ref.width && y < ref.height) put(x, y, ...a);
      if (x < ours.width && y < ours.height) put(ref.width + x, y, ...b);
      const differs = Math.abs(a[0] - b[0]) > tolerance || Math.abs(a[1] - b[1]) > tolerance || Math.abs(a[2] - b[2]) > tolerance;
      if (differs) changed++;
      const faded = 255 - ((255 - (a[0] + a[1] + a[2]) / 3) >> 2);
      put(ref.width + ours.width + x, y, ...(differs ? [230, 0, 0] : [faded, faded, faded]));
    }
  }
  return { percent: (100 * changed) / (Math.max(ref.width, ours.width) * height), png: encodePng(width, height, out) };
}

export function encodePng(width, height, rgb) {
  const raw = Buffer.alloc((width * 3 + 1) * height); // each row starts with filter type 0
  for (let y = 0; y < height; y++) rgb.copy(raw, y * (width * 3 + 1) + 1, y * width * 3, (y + 1) * width * 3);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr.set([8, 2, 0, 0, 0], 8); // 8-bit RGB
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk('IHDR', ihdr), chunk('IDAT', deflateSync(raw)), chunk('IEND', Buffer.alloc(0))]);
}

function chunk(type, data) {
  const out = Buffer.alloc(12 + data.length);
  out.writeUInt32BE(data.length, 0);
  out.write(type, 4, 'latin1');
  data.copy(out, 8);
  out.writeUInt32BE(crc32(out.subarray(4, 8 + data.length)), 8 + data.length);
  return out;
}
