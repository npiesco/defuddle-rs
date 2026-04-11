/**
 * Pre-written narration segments for the defuddle demo.
 *
 * Each major surface gets its own narration block so post can fit audio to the
 * actual recorded segment duration instead of forcing one long narration track.
 */
export interface NarrationSegment {
  phase: number;
  label: string;
  text: string;
  narrationEstimate: number;
}

export const NARRATION_SCRIPT: NarrationSegment[] = [
  {
    phase: 1,
    label: 'Hook',
    text:
      'Start on a noisy technical page. The useful part is the article body, not the forum chrome around it.',
    narrationEstimate: 8,
  },
  {
    phase: 2,
    label: 'Browser Extension',
    text:
      'The browser extension captures the active SQLite page and opens the side panel on that same tab. The extraction result is already clean: title, metadata, and readable markdown.',
    narrationEstimate: 12,
  },
  {
    phase: 3,
    label: 'Rust Crate',
    text:
      'Now the same extraction runs directly from the Rust crate. This proves the extension is just one surface over the real parser core.',
    narrationEstimate: 10,
  },
  {
    phase: 4,
    label: 'Python Bindings',
    text:
      'The Python bindings expose that same engine to scripts and workflows. Same page, same parser, same core result.',
    narrationEstimate: 10,
  },
  {
    phase: 5,
    label: 'MCP',
    text:
      'Through MCP, Copilot can call defuddle as a tool, fetch the live page, and work from the extracted content instead of raw page chrome.',
    narrationEstimate: 12,
  },
  {
    phase: 6,
    label: 'Close',
    text:
      'One parser. Four surfaces. Same result.',
    narrationEstimate: 6,
  },
];

export interface SegmentGroup {
  segPrefix: string;
  phases: number[];
}

export const SEGMENT_GROUPS: SegmentGroup[] = [
  { segPrefix: 'seg_01', phases: [1] },
  { segPrefix: 'seg_02', phases: [2] },
  { segPrefix: 'seg_03', phases: [3] },
  { segPrefix: 'seg_04', phases: [4] },
  { segPrefix: 'seg_05', phases: [5] },
  { segPrefix: 'seg_06', phases: [6] },
];

export function buildSSML(
  segments: NarrationSegment[],
  voice: string = 'en-US-AndrewMultilingualNeural',
  rate: string = 'medium',
): string {
  const escapeXml = (s: string): string =>
    s
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&apos;');

  const lines: string[] = [
    '<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="en-US">',
    `  <voice name="${escapeXml(voice)}">`,
    `    <prosody rate="${escapeXml(rate)}">`,
  ];

  for (let i = 0; i < segments.length; i++) {
    lines.push(`      ${escapeXml(segments[i].text)}`);
    if (i < segments.length - 1) {
      lines.push('      <break time="1000ms"/>');
    }
  }

  lines.push('    </prosody>');
  lines.push('  </voice>');
  lines.push('</speak>');
  return lines.join('\n');
}

export function buildSegmentSSML(
  phases: number[],
  voice: string = 'en-US-AndrewMultilingualNeural',
  rate: string = 'medium',
): string {
  const segments = NARRATION_SCRIPT.filter((segment) => phases.includes(segment.phase));
  return buildSSML(segments, voice, rate);
}
