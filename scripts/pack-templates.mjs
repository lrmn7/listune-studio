import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import zlib from 'node:zlib';

const templatesDir = path.resolve('templates');
const outDir = path.resolve('dist/templates');
fs.mkdirSync(outDir, { recursive: true });

const templateSlugs = ['swiss-motion', 'dataviz', 'crayon-book', 'satellite-briefing'];
const results = {};

function collectFiles(dir, base = '') {
  const entries = [];
  const items = fs.readdirSync(dir).sort();
  for (const item of items) {
    const full = path.join(dir, item);
    const rel = base ? `${base}/${item}` : item;
    const stat = fs.statSync(full);
    if (stat.isDirectory()) {
      entries.push(...collectFiles(full, rel));
    } else {
      let data = fs.readFileSync(full);
      // Normalize text file line endings to LF (\n) for cross-platform deterministic hashing
      if (/\.(ts|tsx|js|mjs|json|md|css|html|txt)$/i.test(item)) {
        data = Buffer.from(data.toString('utf8').replace(/\r\n/g, '\n'), 'utf8');
      }
      entries.push({
        name: rel.replace(/\\/g, '/'),
        data,
      });
    }
  }
  return entries;
}

function buildDeterministicZip(files) {
  const localChunks = [];
  const cdChunks = [];
  let offset = 0;

  // Fixed DOS timestamp: 2026-01-01 00:00:00
  const dosTime = 0;
  const dosDate = (46 << 9) | (1 << 5) | 1;

  for (const file of files) {
    const nameBuf = Buffer.from(file.name.replace(/\\/g, '/'), 'utf8');
    const dataBuf = file.data;
    const compressed = zlib.deflateRawSync(dataBuf);
    const crc = zlib.crc32(dataBuf);

    // Local file header (30 bytes)
    const localHdr = Buffer.alloc(30);
    localHdr.writeUInt32LE(0x04034b50, 0); // signature
    localHdr.writeUInt16LE(20, 4); // version needed
    localHdr.writeUInt16LE(0x0800, 6); // flags: UTF-8
    localHdr.writeUInt16LE(8, 8); // method: DEFLATE
    localHdr.writeUInt16LE(dosTime, 10);
    localHdr.writeUInt16LE(dosDate, 12);
    localHdr.writeUInt32LE(crc, 14);
    localHdr.writeUInt32LE(compressed.length, 18);
    localHdr.writeUInt32LE(dataBuf.length, 22);
    localHdr.writeUInt16LE(nameBuf.length, 26);
    localHdr.writeUInt16LE(0, 28); // extra length

    const localEntry = Buffer.concat([localHdr, nameBuf, compressed]);
    localChunks.push(localEntry);

    // Central directory header (46 bytes)
    const cdHdr = Buffer.alloc(46);
    cdHdr.writeUInt32LE(0x02014b50, 0); // signature
    cdHdr.writeUInt16LE((3 << 8) | 20, 4); // made by: UNIX, zip spec 2.0
    cdHdr.writeUInt16LE(20, 6); // version needed
    cdHdr.writeUInt16LE(0x0800, 8); // flags: UTF-8
    cdHdr.writeUInt16LE(8, 10); // method: DEFLATE
    cdHdr.writeUInt16LE(dosTime, 12);
    cdHdr.writeUInt16LE(dosDate, 14);
    cdHdr.writeUInt32LE(crc, 16);
    cdHdr.writeUInt32LE(compressed.length, 20);
    cdHdr.writeUInt32LE(dataBuf.length, 24);
    cdHdr.writeUInt16LE(nameBuf.length, 28);
    cdHdr.writeUInt16LE(0, 30); // extra length
    cdHdr.writeUInt16LE(0, 32); // comment length
    cdHdr.writeUInt16LE(0, 34); // disk start
    cdHdr.writeUInt16LE(0, 36); // internal attrs
    cdHdr.writeUInt32LE((0o100644 << 16) >>> 0, 38); // external attrs (-rw-r--r--)
    cdHdr.writeUInt32LE(offset, 42); // local header offset

    cdChunks.push(Buffer.concat([cdHdr, nameBuf]));
    offset += localEntry.length;
  }

  const cdBuf = Buffer.concat(cdChunks);

  // End of central directory (22 bytes)
  const eocd = Buffer.alloc(22);
  eocd.writeUInt32LE(0x06054b50, 0); // signature
  eocd.writeUInt16LE(0, 4); // disk num
  eocd.writeUInt16LE(0, 6); // start disk
  eocd.writeUInt16LE(files.length, 8); // entries on disk
  eocd.writeUInt16LE(files.length, 10); // total entries
  eocd.writeUInt32LE(cdBuf.length, 12); // size of CD
  eocd.writeUInt32LE(offset, 16); // offset of CD
  eocd.writeUInt16LE(0, 20); // comment length

  return Buffer.concat([...localChunks, cdBuf, eocd]);
}

for (const slug of templateSlugs) {
  const dir = path.join(templatesDir, slug);
  if (!fs.existsSync(dir)) {
    console.error('Template folder missing:', dir);
    continue;
  }

  const files = collectFiles(dir);
  const zipBuffer = buildDeterministicZip(files);
  const zipPath = path.join(outDir, `${slug}.zip`);
  fs.writeFileSync(zipPath, zipBuffer);

  const sha256 = crypto.createHash('sha256').update(zipBuffer).digest('hex');
  const size = zipBuffer.length;

  results[slug] = { sha256, size, version: 1 };
  console.log(`Packed ${slug}: size=${size} bytes, sha256=${sha256}, files=${files.length}`);
}

const manifestPath = path.join(outDir, 'templates.json');
fs.writeFileSync(manifestPath, JSON.stringify(results, null, 2) + '\n', 'utf8');
console.log('Template manifests written to', manifestPath);
