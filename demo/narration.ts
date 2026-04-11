/**
 * Pre-written narration segments for the fsgdb demo.
 *
 * This mirrors the Duckcells approach: narration is written ahead of time,
 * attached to major scenes, then aligned to edited segments in post.
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
    label: 'General Problem Setup',
    text:
      'Playwright already supports browser targets like Chromium, Edge, and Firefox. ' +
      'That works because those runtimes expose stable seams for launching, attaching, and driving them. ' +
      'The question is what it would take for another engine to fit that model.',
    narrationEstimate: 11,
  },
  {
    phase: 2,
    label: 'Shared Human Agent Model',
    text:
      'This is not a solo investigation. You inspect the codebase directly and visually. ' +
      'I reason over it structurally as the agent. f s g d b is the shared map between us.',
    narrationEstimate: 9,
  },
  {
    phase: 3,
    label: 'Clone Servo and Build the Graph',
    text:
      'For the case study, we use Servo. We clone the repository and scan it once into a local graph database. ' +
      'From this point on, both of us work from the same structural model of the codebase.',
    narrationEstimate: 12,
  },
  {
    phase: 4,
    label: 'CLI Proof of Life',
    text:
      'You start with the raw database. Before trusting any visualization or any agent answer, ' +
      'you inspect the graph directly from the command line. Stats and targeted queries give you a first pass ' +
      'at where startup, embedding, testing, and control surfaces might live.',
    narrationEstimate: 14,
  },
  {
    phase: 5,
    label: 'Open Isolated VS Code Session',
    text:
      'Now we switch to the editor surface. The extension runs in an isolated V S Code session, ' +
      'but it points at the same graph database. This is the same structural model, just in a different interface.',
    narrationEstimate: 11,
  },
  {
    phase: 6,
    label: 'Query Editor Investigation',
    text:
      'In the query editor, you move from the whole repository to a smaller set of candidate areas. ' +
      'You use Cypher to narrow the search around startup, embedding, testing, and other automation relevant seams.',
    narrationEstimate: 11,
  },
  {
    phase: 7,
    label: 'Graph Visualization Investigation',
    text:
      'The graph view turns query results into structure. Now you can see whether likely control points are centralized, ' +
      'fragmented, or deeply coupled. This is where architectural intuition starts to form.',
    narrationEstimate: 11,
  },
  {
    phase: 8,
    label: 'Agent Deep Dive',
    text:
      'Once you have narrowed the search visually, I use the same graph as a structural reasoning layer. ' +
      'I trace likely entry points, dependencies, call paths, tests, and change risk. ' +
      'I am not guessing from file names. I am following how the codebase is actually connected.',
    narrationEstimate: 16,
  },
  {
    phase: 9,
    label: 'Joint Conclusion',
    text:
      'Now we can answer the real question with shared evidence. We know which seams look promising, ' +
      'which areas are more tightly coupled, and what we would validate next if we wanted to pursue a Playwright integration path.',
    narrationEstimate: 13,
  },
  {
    phase: 10,
    label: 'Broader Payoff',
    text:
      'Servo is the example. The broader point is that f s g d b gives a developer and an agent the same map of a codebase. ' +
      'You can inspect it directly. I can reason over it structurally. ' +
      'And together we can evaluate unfamiliar systems with less guesswork.',
    narrationEstimate: 14,
  },
];

export interface SegmentGroup {
  segPrefix: string;
  phases: number[];
}

export const SEGMENT_GROUPS: SegmentGroup[] = [
  { segPrefix: 'seg_01', phases: [1, 2, 3] },
  { segPrefix: 'seg_02', phases: [4] },
  { segPrefix: 'seg_03', phases: [5, 6] },
  { segPrefix: 'seg_04', phases: [7] },
  { segPrefix: 'seg_05', phases: [8, 9, 10] },
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
