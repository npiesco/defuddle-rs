export interface DefuddleResult {
  title: string;
  author: string | null;
  published: string | null;
  site: string | null;
  description: string | null;
  image: string | null;
  language: string | null;
  content_html: string;
  content_markdown: string;
  word_count: number;
  schema_org: unknown | null;
}

export declare function initDefuddleWasm(input?: unknown): Promise<unknown>;
export declare function parseJson(html: string, url: string): string;
export declare function parse(html: string, url: string): DefuddleResult;
