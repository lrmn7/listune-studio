import type { Style, StyleAspect, VideoKind, VideoLanguage } from '@listune/shared';
import { OPUSCAR_CATALOG, OPUSCAR_STYLE_FITS, OPUSCAR_STYLES } from './opuscarStyles';

export const VIDEO_KINDS: VideoKind[] = ['explainer', 'promo', 'quote', 'news', 'tutorial', 'data', 'event'];
const ASPECTS: StyleAspect[] = ['camera', 'pacing', 'type', 'color', 'captions', 'transitions'];


function firstJsonBlock(text: string): string | null {
  const m = /```json\s*\n([\s\S]*?)```/.exec(text);
  return m ? m[1]! : null;
}

export interface TopicStyleRef {
  id: string;
  name: string;
}

/** Used when no styles are synced yet. */
export const DEFAULT_TOPIC_STYLES: TopicStyleRef[] = [
  { id: 'atlas-journey', name: 'Atlas journey' },
  { id: 'cartoon-3d-world', name: 'Cartoon 3D world' },
  { id: 'cartoon-3d-what-if', name: 'Cartoon 3D what-if' },
];

/** All topic styles including the active 25-style Lemo-Opuscar catalog. */
export const ALL_TOPIC_STYLES: TopicStyleRef[] = [
  ...DEFAULT_TOPIC_STYLES,
  ...OPUSCAR_STYLES,
];

/** Complete metadata for the 3 original built-in styles. */
export const DEFAULT_STYLES_CATALOG: Style[] = [
  {
    id: 'atlas-journey',
    name: 'Atlas journey',
    source: 'builtin',
    sourceUrl: null,
    notes: {
      camera: 'One continuous flight over an illustrated globe: spins, swoops to a country, follows a ship or plane, never a hard cut',
      pacing: 'About 60 s, 10 to 14 shots of at least 3 s, a re-hook every 15 s, the last shot loops into the first',
      type: 'Big rounded words pop in one at a time near the bottom, a stat card counts up at the top',
      color: 'Picture-book atlas: teal sea, sand land, thin borders, paper grain, terracotta subject, deep teal for comparisons',
      captions: 'Word by word, about six key words in colour, at most one struck-through word',
      transitions: 'The camera travels between places instead of cutting',
    },
    prompt:
      'A 60-second vertical geography short on a warm illustrated globe.\nOne surprising true fact as a "Did you know" hook, then why it is strange, the reason, the evidence on the map and a payoff.\nOne continuous camera journey of 10 to 14 shots; small illustrated sprites (ships, trains, planes) travel the real routes.\nTerracotta for the subject, deep teal for comparisons, words pop in one at a time, a calm narrator, 140 to 155 words.',
    createdAt: '2026-11-09T13:58:24Z',
    engine: 'remotion',
    previewUrl: '/style-previews/atlas-journey.jpg',
    previewVideoUrl: '/style-previews/atlas-journey.mp4',
    template: {
      sha256: '2f9e57c160e899f26156bd7cbac3bc0937a5a6ece48b6cd504e81572b7fda498',
      size: 81199,
      version: 1,
    },
    version: 1,
  },
  {
    id: 'cartoon-3d-world',
    name: 'Cartoon 3D world',
    source: 'builtin',
    sourceUrl: null,
    notes: {
      camera: 'Smooth gliding moves, hard cuts on big jumps, every hold creeps in, 4-5° rolls, flip-shakes on impacts',
      pacing: '55-70 s story, about 3.7 words per second, every reaction lands 0-3 frames before its word',
      type: 'No captions or labels in the picture',
      color: 'Saturated sky and painted town with black ink outlines, golden-hour shots with violet-blue shadows',
      captions: 'None unless asked for',
      transitions: 'Hard cuts, crossfades on place changes, a white flash after the hook',
    },
    prompt:
      'A 55-70 second vertical cartoon story short: flat bean-head 2D characters with plum ink outlines standing in a painted 3D town with black ink lines, rendered with three.js inside Remotion.\nOne surprising true story with a physical cause and effect: hook, setting, escalation, the snap, a deadpan double line, the explanation through an everyday analogy, the consequence, a callback twist.\nGliding cameras with depth of field, a steady narrator, synthesized sound effects and an original score under the voice. No captions.',
    createdAt: '2026-11-09T13:58:24Z',
    engine: 'remotion',
    previewUrl: '/style-previews/cartoon-3d-world.jpg',
    previewVideoUrl: '/style-previews/cartoon-3d-world.mp4',
    template: {
      sha256: 'ad49f832e04ecb31c26723ae6e266999bcfd1315f1023c9d3726eb5afa8ef889',
      size: 4604773,
      version: 2,
    },
    version: 2,
  },
  {
    id: 'cartoon-3d-what-if',
    name: 'Cartoon 3D what-if',
    source: 'builtin',
    sourceUrl: null,
    notes: {
      camera: 'Smooth gliding moves, hard cuts on big jumps, every hold creeps in, crossfades between places',
      pacing: 'About 55 s: a "What if you..." hook, then Day 1 to Day 30 beats that escalate, each step a real fact',
      type: 'Flat 2D overlay layer: day chips, simple charts and diagrams in a heavy rounded sans with dark ink outlines',
      color: 'Saturated painted 3D world with black ink outlines, golden and dimmed light for the darker turns',
      captions: 'None in the picture; the overlay carries the data',
      transitions: 'Crossfades on place changes, a white flash after the hook, the world dims for the turn',
    },
    prompt:
      'A vertical "What if you..." cartoon short: you, a flat bean-head 2D character, in a painted 3D world with black ink lines, rendered with three.js inside Remotion.\nA one-line hypothetical hook, then day-by-day beats (Day 1, Day 3, ... Day 30) that escalate from fun to strange to serious; every step is a real fact or a real study.\nA recurring cast (you, a parent, friends, strangers) and recurring places; a flat 2D overlay shows the day chips and the evidence as simple charts and diagrams.\nA steady narrator, synthesized sound effects and an original score under the voice, ending on a twist that turns back to the viewer.',
    createdAt: '2026-11-09T13:58:24Z',
    engine: 'remotion',
    previewUrl: '/style-previews/cartoon-3d-what-if.jpg',
    previewVideoUrl: '/style-previews/cartoon-3d-what-if.mp4',
    template: {
      sha256: 'b883d397a5fe7e5cee4e8842f060b4f7cce69dbed6d80a99ddc6e67499113eaa',
      size: 4512125,
      version: 2,
    },
    version: 2,
  },
];

/** Complete catalog of all 28 local built-in styles (3 original + 25 Lemo-Opuscar). */
export const ALL_BUILTIN_STYLES: Style[] = [
  ...DEFAULT_STYLES_CATALOG,
  ...OPUSCAR_CATALOG,
];

export function findBuiltinStyle(id: string): Style | undefined {
  return ALL_BUILTIN_STYLES.find((s) => s.id === id);
}

const STYLE_FIT: Record<string, string> = {
  'atlas-journey': 'places, routes, borders and trade',
  'cartoon-3d-world': 'short true stories',
  'cartoon-3d-what-if': '"What if you…" escalations',
  ...OPUSCAR_STYLE_FITS,
};

/** What every topic must be: something Listune can make in code. */
export const MOTION_RULES = [
  '- Listune makes videos built entirely in code, as motion design or animation: kinetic type, charts and data, maps and globes, illustrated or cartoon 3D worlds, diagrams.',
  "- Never suggest anything that needs filming, a camera, real footage, screen recordings, stock video of real people, a presenter, or a real person's likeness or voice.",
  '- Pick motion-design-friendly topics: explainers, "why" and "how" questions, data and comparisons, history and geography, science what-ifs, product or promo messages told with type and illustration.',
];
export const FACTS_RULE = '- Facts must be real and checkable. No made-up numbers.';

/** How the agent starts a picked video: Listune's local endpoint (see the desktop's ai_proxy.rs). */
export const VIDEO_CALL =
  `curl -s -X POST "$LISTUNE_AI_URL/video" -H "Authorization: Bearer $LISTUNE_AI_TOKEN" -d '{"title":"<the idea title>","kind":"<kind>","lengthSec":<seconds>,"styleId":"<style id or empty>","notes":"<or empty>"}'`;

export function buildTopicIdeasPrompt(
  kinds: VideoKind[],
  language: VideoLanguage,
  styles: TopicStyleRef[] = DEFAULT_TOPIC_STYLES,
): string {
  const lang = language === 'id' ? 'Indonesian' : 'English';
  const list = styles.length ? styles : DEFAULT_TOPIC_STYLES;
  return [
    `Suggest exactly 3 topic ideas for short videos. Write them in ${lang}. Allowed kinds: ${kinds.join(', ')}.`,
    '',
    'Rules for every idea:',
    ...MOTION_RULES,
    '- Fit each idea to one of these styles, or to a plain kinetic-type explainer (empty styleId):',
    ...list.map((s) => `  - ${s.name} (styleId "${s.id}")${STYLE_FIT[s.id] ? `: best for ${STYLE_FIT[s.id]}` : ''}`),
    FACTS_RULE,
    '',
    'How to reply:',
    `1. Start with one short, friendly sentence in ${lang}, as normal text.`,
    '2. Then ask the user to pick one with your tool for asking the user (Claude Code: AskUserQuestion; opencode: question; Antigravity: ask_question; Codex: request_user_input): one question, one option per idea, the title as the option and one line on why it works as a motion video as its description. Without such a tool, list the 3 ideas numbered and ask the user to reply with a number.',
    '3. When the user picks one (or writes their own), start it, then end your turn: Listune makes the video in a new chat.',
    `   ${VIDEO_CALL}`,
    '   `notes`: only what the user said that the title does not say, in their words; never your own choices; or empty.',
    '4. Never write JSON or markup for the choice.',
  ].join('\n');
}

/** Shortest and longest video a topic may ask for, in seconds. */
export const TOPIC_LENGTH = { min: 5, max: 600 } as const;
export const clampTopicLength = (sec: number) => Math.min(TOPIC_LENGTH.max, Math.max(TOPIC_LENGTH.min, Math.round(sec)));

/** The longest `notes` a topic card may carry into the project chat. */
const NOTES_MAX = 600;

/**
 * What a pick_topic button press carries, or null when it has no title. `notes` are the user's own wishes
 * for the video that the title can't hold (no voice-over, a fact to include), passed on to the project chat.
 */
export function topicIdeaFromAction(
  context: Record<string, unknown>,
): { title: string; kind: VideoKind; lengthSec: number; styleId?: string; notes?: string } | null {
  const title = typeof context.title === 'string' ? context.title.trim() : '';
  if (!title) return null;
  const kind = VIDEO_KINDS.includes(context.kind as VideoKind) ? (context.kind as VideoKind) : 'explainer';
  const len = Number(context.lengthSec);
  const lengthSec = Number.isFinite(len) && len > 0 ? clampTopicLength(len) : 30;
  const styleId = typeof context.styleId === 'string' && context.styleId ? context.styleId : undefined;
  const notes = typeof context.notes === 'string' ? context.notes.trim().slice(0, NOTES_MAX) : '';
  return { title, kind, lengthSec, ...(styleId ? { styleId } : {}), ...(notes ? { notes } : {}) };
}

/** Agents that still answer with a ```json block: pull the ideas out and drop the block from the text. */
export function splitTopicJson(text: string): {
  text: string;
  ideas: { title: string; kind: VideoKind; lengthSec: number }[];
} {
  const m = /```json\s*\n([\s\S]*?)```/.exec(text);
  if (!m) return { text, ideas: [] };
  const ideas = parseTopicIdeas(text);
  if (!ideas.length) return { text, ideas: [] };
  const rest = (text.slice(0, m.index) + text.slice(m.index + m[0].length)).replace(/\n{3,}/g, '\n\n').trim();
  return { text: rest, ideas };
}

export function parseTopicIdeas(agentText: string): { title: string; kind: VideoKind; lengthSec: number }[] {
  const raw = firstJsonBlock(agentText);
  if (raw === null) return [];
  try {
    const data: unknown = JSON.parse(raw);
    if (!Array.isArray(data)) return [];
    const out: { title: string; kind: VideoKind; lengthSec: number }[] = [];
    for (const item of data) {
      if (
        !item || typeof item !== 'object' ||
        typeof item.title !== 'string' || !item.title ||
        !VIDEO_KINDS.includes(item.kind) ||
        typeof item.lengthSec !== 'number' || !Number.isFinite(item.lengthSec)
      ) {
        return [];
      }
      out.push({ title: item.title, kind: item.kind, lengthSec: clampTopicLength(item.lengthSec) });
    }
    return out;
  } catch {
    return [];
  }
}

export function buildStudyStylePrompt(url: string, outDir: string): string {
  // Paths are in backticks and quoted for the shell: the folder can contain spaces.
  const sep = outDir.includes('\\') && !outDir.includes('/') ? '\\' : '/';
  const base = outDir.replace(/[\\/]+$/, '');
  const stylePath = `${base}${sep}style.json`;
  const sheetsDir = `${base}${sep}sheets`;
  const sourceTemplate = `${base}${sep}source.%(ext)s`;
  return [
    `Study the visual style of this video so another agent can recreate the look faithfully: ${url}`,
    `1. Download it with \`yt-dlp\` into the folder \`"${outDir}"\`: \`yt-dlp -f "bv*[height<=1080]+ba/b[height<=1080]/b" -o "${sourceTemplate}" <the link>\`.`,
    `2. Make contact sheets so you can see the whole video at once: 48 frames sampled evenly across it, 4x3 per image, in time order (left to right, top to bottom). Get the duration with ffprobe, then: \`ffmpeg -i <source> -vf "fps=48/<duration>,scale=320:-2,tile=4x3" -frames:v 4 -q:v 4 "${sheetsDir}/sheet-%02d.jpg"\` (create \`"${sheetsDir}"\` first). If a moment needs a closer look, take a single full-size frame of it too.`,
    '3. Look at every sheet closely and describe the camera, pacing, type, color, captions and transitions. Use all the frames, not just the first ones: changes across the sheets are the pacing and the transitions.',
    `4. Write \`"${stylePath}"\` as: { "name": string, "notes": { "camera": string, "pacing": string, "type": string, "color": string, "captions": string, "transitions": string }, "prompt": string, "engine": "remotion" | "hyperframes", "build": string, "palette": string[] }.`,
    '   - "name": a short style name. "prompt": a few short lines of direct instructions for the look.',
    '   - "engine": "remotion" when the look needs a real 3D world, 3D or 2.5D characters, depth, lighting or a moving 3D camera (build it with Remotion plus three.js via @remotion/three); "hyperframes" when it is flat 2D motion graphics, kinetic type or illustration.',
    '   - "build": a concrete technical recipe a developer can follow to rebuild this look with that engine: how the world and characters are made (geometry, materials, toon or flat shading, outlines), lighting, the camera rig and its moves (lens, speed), how captions are drawn (font style, size as % of frame height, stroke, position) and how transitions are done. Be specific; this is what decides whether the copy looks right.',
    '   - "palette": 5 to 8 of the main colours as #RRGGBB, measured from the frames.',
    '5. Delete any single frames you took (keep the sheets folder). Leave source.* in place; Listune makes its own contact sheets and a short preview from it, then deletes it.',
    'Copy the look, never the footage: no text, logos, characters\' names, music or clips from the source in the prompt or the recipe. The contact sheets are only for comparing the look.',
  ].join('\n');
}

/** `#abc` / `#AABBCC` as `#AABBCC`; anything else is dropped. */
function hex(c: string): string[] {
  const m = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(c.trim());
  if (!m) return [];
  const h = m[1]!.length === 3 ? [...m[1]!].map((x) => x + x).join('') : m[1]!;
  return [`#${h.toUpperCase()}`];
}

export function parseStyleResult(json: string, url: string): Omit<Style, 'id' | 'createdAt'> | null {
  try {
    const d: unknown = JSON.parse(json);
    if (!d || typeof d !== 'object') return null;
    const o = d as Record<string, unknown>;
    if (typeof o.name !== 'string' || !o.name.trim()) return null;
    if (typeof o.prompt !== 'string' || !o.prompt.trim()) return null;
    if (!o.notes || typeof o.notes !== 'object') return null;
    const notes: Partial<Record<StyleAspect, string>> = {};
    for (const a of ASPECTS) {
      const v = (o.notes as Record<string, unknown>)[a];
      if (typeof v === 'string' && v.trim()) notes[a] = v;
    }
    const engine = o.engine === 'remotion' || o.engine === 'hyperframes' ? o.engine : undefined;
    const build = typeof o.build === 'string' && o.build.trim() ? o.build.trim() : null;
    const palette = Array.isArray(o.palette) ? o.palette.flatMap((c) => (typeof c === 'string' ? hex(c) : [])).slice(0, 8) : [];
    return { name: o.name, source: 'copied', sourceUrl: url, notes, prompt: o.prompt, ...(engine ? { engine } : {}), build, palette };
  } catch {
    return null;
  }
}
