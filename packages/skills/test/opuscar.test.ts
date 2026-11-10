import { describe, expect, it } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import type { Project, Style } from '@listune/shared';
import {
  ALL_BUILTIN_STYLES,
  ALL_TOPIC_STYLES,
  DEFAULT_STYLES_CATALOG,
  OPUSCAR_CATALOG,
  OPUSCAR_STYLES,
  OPUSCAR_STYLE_FITS,
  buildProjectBrief,
  buildTopicIdeasPrompt,
  findBuiltinStyle,
  findOpuscarStyle,
  opuscarStylesByCategory,
} from '../src';

describe('Lemo-Opuscar styles catalog', () => {
  it('contains all 25 active curated film styles with preview media', () => {
    expect(OPUSCAR_STYLES).toHaveLength(25);
    expect(OPUSCAR_CATALOG).toHaveLength(25);
    expect(Object.keys(OPUSCAR_STYLE_FITS)).toHaveLength(25);
    for (const s of OPUSCAR_CATALOG) {
      expect(s.previewUrl).toBeTruthy();
      expect(s.previewVideoUrl).toBeTruthy();
    }
  });

  it('includes key styles from across the categories', () => {
    const ids = OPUSCAR_STYLES.map((s) => s.id);
    expect(ids).toContain('swiss-motion');
    expect(ids).toContain('archive-lighttable');
    expect(ids).toContain('dataviz');
    expect(ids).toContain('crayon-book');
    expect(ids).toContain('satellite-briefing');
    expect(ids).toContain('briefing-board');
    expect(ids).toContain('caption-news');
    expect(ids).toContain('kino-montage');
    expect(ids).toContain('papercut-red');
    expect(ids).toContain('urban-sketch');
    expect(ids).toContain('whiteboard');
  });

  it('every style in catalog has required fields, valid notes, and non-empty palette', () => {
    for (const style of OPUSCAR_CATALOG) {
      expect(style.id).toBeTruthy();
      expect(style.name).toBeTruthy();
      expect(style.source).toBe('builtin');
      expect(style.sourceUrl).toContain('lemomo-ai/lemo-opuscar');
      expect(style.prompt.length).toBeGreaterThan(10);
      expect(style.category).toBeTruthy();
      expect(style.uses.length).toBeGreaterThan(0);
      expect(['remotion', 'hyperframes']).toContain(style.engine);
      expect(style.notes.camera).toBeTruthy();
      expect(style.notes.pacing).toBeTruthy();
      expect(style.notes.type).toBeTruthy();
      expect(style.notes.color).toBeTruthy();
      expect(Array.isArray(style.palette)).toBe(true);
      expect(style.palette?.length).toBeGreaterThan(0);
      // Palette hex formatting
      for (const color of style.palette || []) {
        expect(color).toMatch(/^#[0-9A-F]{6}$/);
      }
    }
  });

  it('findOpuscarStyle retrieves style metadata by id', () => {
    const swiss = findOpuscarStyle('swiss-motion');
    expect(swiss).toBeDefined();
    expect(swiss?.name).toBe('Swiss Motion Graphics');
    expect(swiss?.notes.color).toBeTruthy();
    expect(swiss?.template).toBeDefined();

    const crayon = findOpuscarStyle('crayon-book');
    expect(crayon).toBeDefined();
    expect(crayon?.name).toBe('Crayon Picture Book');
    expect(crayon?.template).toBeDefined();
  });

  it('opuscarStylesByCategory filters styles by category', () => {
    const handDrawn = opuscarStylesByCategory('Hand-drawn & Painting');
    expect(handDrawn.length).toBeGreaterThanOrEqual(6);
    const slugs = handDrawn.map((s) => s.id);
    expect(slugs).toContain('chalkboard');
    expect(slugs).toContain('one-line');
    expect(slugs).toContain('whiteboard');

    const evidence = opuscarStylesByCategory('Evidence Films');
    expect(evidence.length).toBeGreaterThanOrEqual(4);
    expect(evidence.map((s) => s.id)).toContain('satellite-briefing');
  });

  it('buildTopicIdeasPrompt includes Opuscar styles and their fitting descriptions', () => {
    const prompt = buildTopicIdeasPrompt(['explainer', 'data'], 'en', [
      { id: 'swiss-motion', name: 'Swiss Motion Graphics' },
      { id: 'crayon-book', name: 'Crayon Picture Book' },
    ]);
    expect(prompt).toContain('Swiss Motion Graphics');
    expect(prompt).toContain('styleId "swiss-motion"');
    expect(prompt).toContain('brand identity, events, design education');
    expect(prompt).toContain('styleId "crayon-book"');
    expect(prompt).toContain("picture books, bedtime stories, kids' learning");
  });

  it('ALL_TOPIC_STYLES combines default styles with the Opuscar library', () => {
    expect(ALL_TOPIC_STYLES.length).toBe(28);
    expect(ALL_TOPIC_STYLES[0]?.id).toBe('atlas-journey');
    expect(ALL_TOPIC_STYLES[1]?.id).toBe('cartoon-3d-world');
    expect(ALL_TOPIC_STYLES[2]?.id).toBe('cartoon-3d-what-if');
    expect(ALL_TOPIC_STYLES.some((s) => s.id === 'swiss-motion')).toBe(true);
  });

  it('ALL_BUILTIN_STYLES combines the 3 original styles and all 25 active Opuscar styles with zero duplicates and preserved previews', () => {
    expect(DEFAULT_STYLES_CATALOG).toHaveLength(3);
    expect(ALL_BUILTIN_STYLES).toHaveLength(28);

    const ids = ALL_BUILTIN_STYLES.map((s) => s.id);
    expect(new Set(ids).size).toBe(28);

    for (const id of ['atlas-journey', 'cartoon-3d-world', 'cartoon-3d-what-if']) {
      const style = findBuiltinStyle(id);
      expect(style).toBeDefined();
      expect(style?.engine).toBe('remotion');
      expect(style?.previewUrl).toContain(id);
      expect(style?.previewVideoUrl).toContain(id);
      expect(style?.template).toBeTruthy();
    }

    for (const id of [
      'whiteboard',
      'dataviz',
      'iso-infographic',
      'transit-map',
      'chalkboard',
      'swiss-motion',
      'watercolor',
      'shadow-puppet',
      'papercut-red',
      'midcentury-toon',
    ]) {
      const style = findBuiltinStyle(id);
      expect(style).toBeDefined();
      expect(style?.previewUrl).toContain(`/style-previews/${id}`);
      expect(style?.previewVideoUrl).toContain(`/style-previews/${id}`);
    }

    // Verify exact parity with Rust bundled builtin_styles.json
    const rustBundledPath = path.resolve(__dirname, '../../../apps/desktop/src-tauri/src/builtin_styles.json');
    const rustBundled = JSON.parse(fs.readFileSync(rustBundledPath, 'utf8')) as Style[];
    expect(rustBundled).toHaveLength(28);
    expect(rustBundled.map((s) => s.id)).toEqual(ids);
    for (let i = 0; i < rustBundled.length; i++) {
      const r = rustBundled[i]!;
      const t = ALL_BUILTIN_STYLES[i]!;
      expect(r.name).toBe(t.name);
      expect(r.source).toBe(t.source);
      expect(r.sourceUrl ?? null).toBe(t.sourceUrl ?? null);
      expect(r.prompt).toBe(t.prompt);
      expect(r.notes).toEqual(t.notes);
      expect(r.createdAt).toBe(t.createdAt);
      expect(r.engine).toBe(t.engine);
      expect(r.previewUrl ?? null).toBe(t.previewUrl ?? null);
      expect(r.previewVideoUrl ?? null).toBe(t.previewVideoUrl ?? null);
      expect(r.template ?? null).toEqual(t.template ?? null);
      expect(r.version).toBe(t.version);
      expect(r.build ?? null).toBe(t.build ?? null);
      expect(r.palette ?? []).toEqual(t.palette ?? []);
    }
  });
});

describe('Starter templates for Opuscar Remotion styles', () => {
  const templatesRoot = path.resolve(__dirname, '../../../templates');
  const templateSlugs = ['swiss-motion', 'dataviz', 'crayon-book', 'satellite-briefing'];

  it.each(templateSlugs)('template %s has all required files and valid listune.json', (slug) => {
    const dir = path.join(templatesRoot, slug);
    expect(fs.existsSync(dir)).toBe(true);

    const listuneJsonPath = path.join(dir, 'listune.json');
    expect(fs.existsSync(listuneJsonPath)).toBe(true);
    const listuneJson = JSON.parse(fs.readFileSync(listuneJsonPath, 'utf8'));
    expect(listuneJson.engine).toBe('remotion');
    expect(listuneJson.render).toEqual(['npx', 'remotion', 'render', 'src/index.ts', 'Main', '{output}']);
    expect(listuneJson.install).toEqual(['npm', 'install']);
    expect(listuneJson.readme).toBe('STYLE.md');

    expect(fs.existsSync(path.join(dir, 'STYLE.md'))).toBe(true);
    expect(fs.existsSync(path.join(dir, 'package.json'))).toBe(true);
    expect(fs.existsSync(path.join(dir, 'remotion.config.ts'))).toBe(true);
    expect(fs.existsSync(path.join(dir, 'src/index.ts'))).toBe(true);
    expect(fs.existsSync(path.join(dir, 'src/Root.tsx'))).toBe(true);
    expect(fs.existsSync(path.join(dir, 'src/Composition.tsx'))).toBe(true);
  });

  it('buildProjectBrief formats starter template instructions correctly', () => {
    const swiss = findOpuscarStyle('swiss-motion') as Style;
    const project = {
      title: 'Desain Swiss Modern',
      slug: 'desain-swiss-modern',
      kind: 'explainer',
      format: '9:16',
      language: 'id',
    } as Project;

    const brief = buildProjectBrief(project, swiss, { fps: 30, frameCheck: true });
    expect(brief).toContain('# Desain Swiss Modern');
    expect(brief).toContain('This project starts from the Swiss Motion Graphics template.');
    expect(brief).toContain('Read STYLE.md first, then run the install command in listune.json once.');
    expect(brief).toContain('Render with the command in listune.json.');
    expect(brief).toContain('Style: Swiss Motion Graphics');
    expect(brief).toContain('Color:');
  });

  it('packed template archives match manifest and have safe forward-slash paths', () => {
    const distTemplates = path.resolve(__dirname, '../../../dist/templates');
    const manifestPath = path.join(distTemplates, 'templates.json');
    expect(fs.existsSync(manifestPath)).toBe(true);

    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));

    for (const slug of templateSlugs) {
      const zipPath = path.join(distTemplates, `${slug}.zip`);
      expect(fs.existsSync(zipPath)).toBe(true);
      const bytes = fs.readFileSync(zipPath);

      // Verify SHA256 and size
      const crypto = require('node:crypto');
      const sha256 = crypto.createHash('sha256').update(bytes).digest('hex');
      expect(manifest[slug]).toBeDefined();
      expect(manifest[slug].sha256).toBe(sha256);
      expect(manifest[slug].size).toBe(bytes.length);

      // Verify OPUSCAR_CATALOG matches manifest
      const catalogStyle = findOpuscarStyle(slug);
      expect(catalogStyle?.template?.sha256).toBe(sha256);
      expect(catalogStyle?.template?.size).toBe(bytes.length);

      // Check zip header and inspect entry paths
      expect(bytes.readUInt32LE(0)).toBe(0x04034b50);

      // Search through Central Directory headers (0x02014b50)
      let pos = 0;
      let foundEntries = 0;
      while (pos < bytes.length - 4) {
        if (bytes.readUInt32LE(pos) === 0x02014b50) {
          const nameLen = bytes.readUInt16LE(pos + 28);
          const extraLen = bytes.readUInt16LE(pos + 30);
          const commentLen = bytes.readUInt16LE(pos + 32);
          const name = bytes.toString('utf8', pos + 46, pos + 46 + nameLen);

          // Verify safe paths: must NOT contain backslash, colons, nulls, or path traversal
          expect(name).not.toContain('\\');
          expect(name).not.toContain(':');
          expect(name).not.toContain('\0');
          expect(name).not.toContain('..');
          foundEntries++;
          pos += 46 + nameLen + extraLen + commentLen;
        } else {
          pos++;
        }
      }
      expect(foundEntries).toBe(9);
    }
  });
});
