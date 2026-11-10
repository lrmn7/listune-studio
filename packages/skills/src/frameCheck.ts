/**
 * What the agent writes, word for word in any language, once every still is clean. Listune looks for it to
 * show the "frames checked" badge and hides it from the message.
 */
export const FRAMES_CHECKED_TOKEN = '[listune:frames-checked]';

/** Legacy tokens/phrases still accepted for chats written before the rebrand or token existed. */
const LEGACY_TOKENS = ['[Listune:frames-checked]'] as const;
const LEGACY_PHRASE = 'Frames checked';

export const hasFramesChecked = (text: string): boolean =>
  text.includes(FRAMES_CHECKED_TOKEN) ||
  LEGACY_TOKENS.some((t) => text.includes(t)) ||
  text.includes(LEGACY_PHRASE);

const escapeRe = (t: string) => t.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const ALL_TOKENS = [FRAMES_CHECKED_TOKEN, ...LEGACY_TOKENS];
const WRAPPED_TOKEN = new RegExp(`\`*(?:${ALL_TOKENS.map(escapeRe).join('|')})\`*`, 'g');
/** Every proper prefix of every supported token, longest first, so a half-streamed one can be hidden. */
const PREFIXES = ALL_TOKENS.flatMap((tok) =>
  Array.from({ length: tok.length - 1 }, (_, i) => tok.slice(0, tok.length - 1 - i)),
).sort((a, b) => b.length - a.length);
const PARTIAL_TOKEN = new RegExp(`\`*(?:${PREFIXES.map(escapeRe).join('|')})$`);

/**
 * The message as the reader sees it: the token is plumbing, also when the agent wraps it in
 * backticks, and also while it is still being typed at the end of streaming text.
 */
export const stripFramesToken = (text: string): string => {
  const whole = text.replace(WRAPPED_TOKEN, '');
  const cut = whole.replace(PARTIAL_TOKEN, '');
  if (cut === text) return text;
  return cut.replace(/(\S) {2,}/g, '$1 ').replace(/[ \t]+\n/g, '\n').replace(/\n{3,}/g, '\n\n').trim();
};

/** Agents can't speak after their turn ends: long work must run inside the turn, where Listune shows it. */
export const FOREGROUND_RULE =
  "Never end your turn by saying what you will do next (\"I'll render it again\"): do it in the same turn. End your turn only when the work is done, or when you need the user to answer or allow something. Run installs, renders and pipelines in the foreground and wait for them to finish before you reply. Never start a long job in the background and end your turn: you can't message the user after your turn ends, and Listune shows the user that you're still working while your turn runs. If a command needs more than a few minutes, say so first, then run it and wait. Keep command output short: use the command's quiet flag, or end a long install or render with `2>&1 | tail -n 30` and read those last lines for errors; progress bars and install logs only fill your context.";

/** Listune's preview and status read the render from renders/; a template pipeline may write it elsewhere. */
export const RENDERS_RULE =
  "When the video is finished, make sure the final mp4 is in the project's renders/ folder (copy it there if your pipeline writes it elsewhere). Listune's preview and status read it from there.";

/**
 * Look at all stills as one small image instead of one full-size image each: a still costs the
 * agent as many tokens as a whole sheet. `%02d` instead of a glob, which Windows builds of ffmpeg lack.
 */
export const SHEET_STEP =
  'Put the stills on one contact sheet and look at that one image, not at each still: `ffmpeg -y -i stills/scene-%02d.png -vf "scale=360:-2,tile=4x3" -frames:v 1 stills/sheet.jpg` (more than 12 scenes: run it again with `-start_number 13` before `-i` and `stills/sheet-2.jpg` as the output). Look for overlapping elements, text that is cut off or outside the safe area, and timing that does not match the voice or captions. Open a single still only when the sheet shows a problem you need to see closer.';

/** Folder in a project where Listune puts the style's reference frames. */
export const STYLE_REFS_DIR = 'style-refs';

/** The quality loop for the project's engine; with references, every still is held against them too. */
export function frameCheckPrompt(opts: { engine?: 'hyperframes' | 'remotion'; references?: boolean } = {}): string {
  const still =
    opts.engine === 'remotion'
      ? '1. Render a still of every scene with `npx remotion still` (one frame per scene)'
      : '1. Render a still of every scene with `npx hyperframes`';
  return [
    'Check the frames before you report this video as done.',
    `${still}, named in order: stills/scene-01.png, stills/scene-02.png, …`,
    `2. ${SHEET_STEP}`,
    ...(opts.references
      ? [
          `   Then make a contact sheet of your render the same way as the references (\`ffmpeg -i <your render> -vf "fps=12/<duration>,scale=320:-2,tile=4x3" -frames:v 1 renders/check-sheet.jpg\`) and look at it side by side with the contact sheets in ${STYLE_REFS_DIR}/: the same rendering technique, character design, palette, lighting, camera framing, caption style and pacing. Anything that looks different is a problem to fix.`,
        ]
      : []),
    '3. Fix every problem you find, then render the affected stills again and look at them.',
    `4. When all stills are clean, write ${FRAMES_CHECKED_TOKEN} exactly as shown, whatever language you reply in, then a one-line summary of what you fixed.`,
  ].join('\n');
}

/** The default (HyperFrames, no references) quality loop. */
export const FRAME_CHECK_PROMPT = frameCheckPrompt();

