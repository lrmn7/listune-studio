import fs from 'node:fs';
import path from 'node:path';

const tempStylesDir = path.join(process.env.TEMP || 'C:\\Users\\pagar\\AppData\\Local\\Temp', 'lemo-opuscar', 'styles');

if (!fs.existsSync(tempStylesDir)) {
  console.error('Source styles directory not found:', tempStylesDir);
  process.exit(1);
}

const slugs = fs.readdirSync(tempStylesDir).filter((f) => {
  return !f.startsWith('_') && fs.statSync(path.join(tempStylesDir, f)).isDirectory();
}).sort();

console.log(`Found ${slugs.length} styles in ${tempStylesDir}`);

function cleanText(text) {
  return text.replace(/\r\n/g, '\n').replace(/\r/g, '\n').trim();
}

function extractSection(content, sectionNames) {
  const lines = cleanText(content).split('\n');
  let capturing = false;
  const capturedLines = [];

  for (const line of lines) {
    const headingMatch = line.match(/^##\s+(?:\d+\.\s+)?(.*)/);
    if (headingMatch) {
      const heading = headingMatch[1].toLowerCase();
      const matches = sectionNames.some((name) => heading.includes(name.toLowerCase()));
      if (matches) {
        capturing = true;
        continue;
      } else if (capturing) {
        break;
      }
    }
    if (capturing) {
      capturedLines.push(line);
    }
  }

  return capturedLines.join('\n').trim();
}

function extractDemoHexes(dir) {
  const demoDir = path.join(dir, 'demo');
  if (!fs.existsSync(demoDir)) return [];
  const found = [];
  function scan(p) {
    for (const f of fs.readdirSync(p)) {
      const full = path.join(p, f);
      if (fs.statSync(full).isDirectory()) {
        if (!['fonts', 'music', 'voices'].includes(f)) scan(full);
      } else if (/\.(js|json|html|css|py|mjs)$/i.test(f)) {
        const txt = fs.readFileSync(full, 'utf8');
        const m = txt.match(/#[0-9a-fA-F]{6}\b/g);
        if (m) found.push(...m);
      }
    }
  }
  scan(demoDir);
  return [...new Set(found.map((h) => h.toUpperCase()))];
}

const catalog = [];
const fits = {};

const manifestPath = path.resolve('dist/templates/templates.json');
let templateManifests = {};
if (fs.existsSync(manifestPath)) {
  templateManifests = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
}

const tauriCatalogPath = path.join('apps', 'desktop', 'src-tauri', 'src', 'builtin_styles.json');
let existingBundled = [];
if (fs.existsSync(tauriCatalogPath)) {
  existingBundled = JSON.parse(fs.readFileSync(tauriCatalogPath, 'utf8'));
}
const existingById = new Map(existingBundled.map((s) => [s.id, s]));

for (const slug of slugs) {
  const dir = path.join(tempStylesDir, slug);
  const styleJsonPath = path.join(dir, 'style.json');
  const styleMdPath = path.join(dir, 'STYLE.md');

  if (!fs.existsSync(styleJsonPath) || !fs.existsSync(styleMdPath)) {
    console.warn(`Skipping ${slug}: missing style.json or STYLE.md`);
    continue;
  }

  const meta = JSON.parse(fs.readFileSync(styleJsonPath, 'utf8'));
  const md = cleanText(fs.readFileSync(styleMdPath, 'utf8'));

  // Prompt: blockquote or section 1
  let prompt = meta.line || '';
  const bqMatch = md.match(/>\s*([\s\S]*?)(?=\n##|$)/);
  if (bqMatch) {
    prompt = bqMatch[1].replace(/^>\s*/gm, '').trim();
  }

  // Camera
  let camera = extractSection(md, ['camera grammar', 'camera on', 'camera', 'frame and layout', 'frame', 'materials & rendering', 'materials']);

  // Pacing
  let pacing = extractSection(md, ['motion quality', 'motion and drawing', 'motion', 'pacing', 'editing grammar', 'edit grammar', 'cutting rhythm']);

  // Type
  let typeNote = extractSection(md, ['type & subtitles', 'type', 'typography', 'caption card', 'captions']);

  // Color
  let colorNote = extractSection(md, ['colour logic', 'color logic', 'colour identity', 'colour', 'color', 'grade and image quality', 'graphics', 'materials & rendering']);

  // Captions
  let captionsNote = '';
  if (typeNote.includes('subtitle') || typeNote.includes('caption') || typeNote.includes('Subtitles')) {
    const subLines = typeNote.split('\n').filter((l) => /subtitle|caption|words|lead-in|whisper/i.test(l));
    captionsNote = subLines.length ? subLines.join('\n').trim() : typeNote.slice(0, 300);
  } else {
    captionsNote = typeNote.slice(0, 300);
  }

  // Transitions
  let transitionsNote = '';
  if (pacing.includes('transition') || pacing.includes('wipe') || pacing.includes('cut') || pacing.includes('snap') || pacing.includes('dissolve')) {
    const transLines = pacing.split('\n').filter((l) => /transition|cut|wipe|snap|dissolve|fade/i.test(l));
    transitionsNote = transLines.length ? transLines.join('\n').trim() : pacing.slice(0, 300);
  } else {
    transitionsNote = pacing.slice(0, 300);
  }

  // Build / Recipe
  let build = extractSection(md, ['engine', 'materials & rendering', 'materials', 'formats', 'package parts', 'native moves']);

  // Palette
  const mdHexes = md.match(/#[0-9a-fA-F]{6}\b/g) || [];
  let hexList = [...new Set(mdHexes.map((h) => h.toUpperCase()))];
  if (hexList.length < 2) {
    const demoHexes = extractDemoHexes(dir);
    hexList = [...new Set([...hexList, ...demoHexes])];
  }
  const palette = hexList.slice(0, 8);

  const prev = existingById.get(slug);
  if (existingBundled.length > 0 && (!prev?.previewUrl || !prev?.previewVideoUrl)) {
    continue;
  }
  const is3dOrComplex =
    /\b(?:three\.js|webgl|3d|isometric|lowpoly|point cloud|gltf|obj|mesh)\b/i.test(md) ||
    /\b(?:three\.js|webgl|3d)\b/i.test(build);
  const engine = prev?.engine || (templateManifests[slug] || is3dOrComplex ? 'remotion' : 'hyperframes');

  const usesStr = Array.isArray(meta.uses) ? meta.uses.join(', ').toLowerCase() : '';
  fits[slug] = usesStr || meta.category_en.toLowerCase();

  const notes = {};
  if (camera) notes.camera = camera.slice(0, 400);
  if (pacing) notes.pacing = pacing.slice(0, 400);
  if (typeNote) notes.type = typeNote.slice(0, 400);
  if (colorNote) notes.color = colorNote.slice(0, 400);
  if (captionsNote) notes.captions = captionsNote.slice(0, 400);
  if (transitionsNote) notes.transitions = transitionsNote.slice(0, 400);

  catalog.push({
    id: slug,
    name: meta.en,
    source: 'builtin',
    sourceUrl: 'https://github.com/lemomo-ai/lemo-opuscar',
    prompt,
    notes,
    createdAt: '2026-10-29T00:00:00Z',
    engine,
    ...(prev?.previewUrl ? { previewUrl: prev.previewUrl } : {}),
    ...(prev?.previewVideoUrl ? { previewVideoUrl: prev.previewVideoUrl } : {}),
    template: templateManifests[slug] || null,
    build: build ? build.slice(0, 600) : null,
    palette,
    category: meta.category_en || 'Other',
    categoryCn: meta.category_cn || '',
    nameCn: meta.cn || '',
    uses: meta.uses || [],
  });
}

console.log(`Parsed ${catalog.length} styles.`);

const tsContent = `// Generated from https://github.com/lemomo-ai/lemo-opuscar styles library.
// \${catalog.length} curated motion video styles across categories with active visual previews.

import type { Style } from '@listune/shared';
import type { TopicStyleRef } from './style';

export interface OpuscarStyleMeta extends Style {
  category: string;
  categoryCn: string;
  nameCn: string;
  uses: string[];
}

export const OPUSCAR_STYLES: TopicStyleRef[] = [
${catalog.map((s) => `  { id: ${JSON.stringify(s.id)}, name: ${JSON.stringify(s.name)} },`).join('\n')}
];

export const OPUSCAR_STYLE_FITS: Record<string, string> = {
${Object.entries(fits).map(([k, v]) => `  ${JSON.stringify(k)}: ${JSON.stringify(v)},`).join('\n')}
};

export const OPUSCAR_CATALOG: OpuscarStyleMeta[] = ${JSON.stringify(catalog, null, 2)};

export function findOpuscarStyle(id: string): OpuscarStyleMeta | undefined {
  return OPUSCAR_CATALOG.find((s) => s.id === id);
}

export function opuscarStylesByCategory(category: string): OpuscarStyleMeta[] {
  return OPUSCAR_CATALOG.filter((s) => s.category.toLowerCase() === category.toLowerCase());
}
`;

fs.writeFileSync(path.join('packages', 'skills', 'src', 'opuscarStyles.ts'), tsContent, 'utf8');
console.log('Written packages/skills/src/opuscarStyles.ts successfully.');

const opuscarIds = new Set(catalog.map((s) => s.id));
const preservedDefaults = existingBundled.filter((s) => !opuscarIds.has(s.id));
const unifiedCatalog = [...preservedDefaults, ...catalog];
fs.writeFileSync(tauriCatalogPath, JSON.stringify(unifiedCatalog, null, 2), 'utf8');
console.log(`Written ${unifiedCatalog.length} styles to apps/desktop/src-tauri/src/builtin_styles.json successfully.`);
