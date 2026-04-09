import { initDefuddleWasm, parse } from "./wasm/index.js";

const STORAGE_KEY = "defuddle_rs_extension_state_v1";
const CAPTURE_KEY = "defuddle_latest_capture";
const REPO_URL = "https://github.com/npiesco/defuddle-rs";
const SAMPLE_URL = "https://example.com/articles/clean-room-port";
const SAMPLE_HTML = `<!DOCTYPE html>
<html lang="en">
  <head>
    <title>Clean-Room Porting Notes</title>
    <meta name="author" content="Ada Rivers" />
    <meta name="description" content="What changes when a parser moves from TypeScript into Rust and WebAssembly." />
  </head>
  <body>
    <header>
      <nav><a href="/">Home</a> <a href="/archive">Archive</a></nav>
    </header>
    <main>
      <article>
        <h1>Clean-Room Porting Notes</h1>
        <p>Browser delivery changes the shape of a parser. The code still needs strong heuristics, but the user experience changes once the parser can run locally, without a server round-trip.</p>
        <p>Extension-first delivery means the parser can capture the current tab directly, bypassing website CORS limits while keeping the product local to the browser.</p>
        <p>This sample exists so the extension page can be exercised end to end before you capture a real tab.</p>
      </article>
      <aside>Newsletter signup</aside>
      <footer>Footer links</footer>
    </main>
  </body>
</html>`;

const state = {
  mode: "url",
  wasmReady: false,
  htmlDraft: "",
  htmlUrl: "",
  fetchUrl: "",
  importedFileName: "",
  importedFileContent: "",
  activeTab: "markdown",
  result: null,
  latestCapture: null,
};

const elements = {
  captureActiveTab: document.querySelector("#capture-active-tab"),
  githubLink: document.querySelector("#github-link"),
  statusBanner: document.querySelector("#status-banner"),
  wasmState: document.querySelector("#wasm-state"),
  lastRun: document.querySelector("#last-run"),
  draftBytes: document.querySelector("#draft-bytes"),
  htmlInput: document.querySelector("#html-input"),
  htmlUrlInput: document.querySelector("#html-url-input"),
  urlInput: document.querySelector("#url-input"),
  fileInput: document.querySelector("#file-input"),
  fileName: document.querySelector("#file-name"),
  summaryEmpty: document.querySelector("#summary-empty"),
  summaryPanel: document.querySelector("#summary-panel"),
  markdownOutput: document.querySelector("#markdown-output"),
  htmlOutput: document.querySelector("#html-output"),
  jsonOutput: document.querySelector("#json-output"),
  previewFrame: document.querySelector("#preview-frame"),
  summaryTitle: document.querySelector("#summary-title"),
  summarySite: document.querySelector("#summary-site"),
  summaryAuthor: document.querySelector("#summary-author"),
  summaryPublished: document.querySelector("#summary-published"),
  summaryLanguage: document.querySelector("#summary-language"),
  summaryWordCount: document.querySelector("#summary-word-count"),
  captureTitle: document.querySelector("#capture-title"),
  captureUrl: document.querySelector("#capture-url"),
  captureTime: document.querySelector("#capture-time"),
};

function saveState() {
  const snapshot = {
    mode: state.mode,
    htmlDraft: state.htmlDraft,
    htmlUrl: state.htmlUrl,
    fetchUrl: state.fetchUrl,
    importedFileName: state.importedFileName,
    importedFileContent: state.importedFileContent,
    activeTab: state.activeTab,
  };
  localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
}

function restoreState() {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return;
  }

  try {
    const saved = JSON.parse(raw);
    state.mode = saved.mode || state.mode;
    state.htmlDraft = saved.htmlDraft || "";
    state.htmlUrl = saved.htmlUrl || "";
    state.fetchUrl = saved.fetchUrl || "";
    state.importedFileName = saved.importedFileName || "";
    state.importedFileContent = saved.importedFileContent || "";
    state.activeTab = saved.activeTab || state.activeTab;
  } catch (error) {
    console.warn("failed to restore state", error);
  }
}

function setBanner(message, variant = "info") {
  elements.statusBanner.hidden = false;
  elements.statusBanner.dataset.variant = variant;
  elements.statusBanner.textContent = message;
}

function clearBanner() {
  elements.statusBanner.hidden = true;
  elements.statusBanner.textContent = "";
  delete elements.statusBanner.dataset.variant;
}

function formatBytes(text) {
  const bytes = new TextEncoder().encode(text || "").length;
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function setMode(nextMode) {
  state.mode = nextMode;
  document.querySelectorAll(".mode-chip").forEach((chip) => {
    chip.classList.toggle("is-active", chip.dataset.mode === nextMode);
  });
  document.querySelectorAll(".mode-pane").forEach((pane) => {
    pane.classList.toggle("is-active", pane.id === `${nextMode}-pane`);
  });
  saveState();
}

function setActiveTab(nextTab) {
  state.activeTab = nextTab;
  document.querySelectorAll(".tab").forEach((tab) => {
    tab.classList.toggle("is-active", tab.dataset.tab === nextTab);
  });
  document.querySelectorAll(".tab-panel").forEach((panel) => {
    panel.classList.toggle("is-active", panel.id === `tab-${nextTab}`);
  });
  saveState();
}

function syncInputsFromState() {
  elements.htmlInput.value = state.htmlDraft;
  elements.htmlUrlInput.value = state.htmlUrl;
  elements.urlInput.value = state.fetchUrl;
  elements.fileName.textContent = state.importedFileName || "No file loaded";
  elements.draftBytes.textContent = formatBytes(state.htmlDraft);
  setMode(state.mode);
  setActiveTab(state.activeTab);
}

function updateRuntimeCard() {
  elements.draftBytes.textContent = formatBytes(state.htmlDraft);
}

function readCurrentUrl() {
  return (state.htmlUrl || state.fetchUrl || "https://example.com/article").trim();
}

function sanitizePreviewDocument(contentHtml) {
  return `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
      :root {
        color-scheme: light;
      }
      html {
        background: #fffcf8;
      }
      body {
        margin: 0;
        padding: 24px;
        font-family: "Georgia", "Times New Roman", serif;
        color: #201815;
        line-height: 1.65;
        background: #fffcf8;
      }
      .defuddle-preview,
      .defuddle-preview * {
        box-sizing: border-box;
      }
      .defuddle-preview {
        color: #201815;
      }
      .defuddle-preview :where(
        p,
        li,
        figcaption,
        td,
        th,
        span,
        div,
        article,
        section,
        main,
        aside,
        header,
        footer,
        h1,
        h2,
        h3,
        h4,
        h5,
        h6
      ) {
        color: #201815 !important;
        background: transparent !important;
      }
      .defuddle-preview a {
        color: #8f3518 !important;
      }
      .defuddle-preview img,
      .defuddle-preview video,
      .defuddle-preview table,
      .defuddle-preview svg {
        max-width: 100%;
      }
      .defuddle-preview table {
        border-collapse: collapse;
      }
      .defuddle-preview th,
      .defuddle-preview td {
        padding: 8px 10px;
        border: 1px solid rgba(32, 24, 21, 0.14);
      }
      pre {
        padding: 16px;
        overflow: auto;
        border-radius: 14px;
        background: #241c19;
        color: #fff7ee;
        -webkit-text-fill-color: #fff7ee;
      }
      pre * {
        color: #fff7ee !important;
        background: transparent !important;
        -webkit-text-fill-color: #fff7ee !important;
        opacity: 1 !important;
        text-shadow: none !important;
        mix-blend-mode: normal !important;
      }
      code:not(pre code) {
        color: #8f3518 !important;
        background: rgba(143, 53, 24, 0.08) !important;
        -webkit-text-fill-color: #8f3518 !important;
        padding: 0.12em 0.3em;
        border-radius: 0.35em;
      }
      code {
        font-family: "SFMono-Regular", Consolas, monospace;
      }
      blockquote {
        margin-left: 0;
        padding-left: 16px;
        border-left: 4px solid #d46b43;
        color: #5f5148;
      }
    </style>
  </head>
  <body><div class="defuddle-preview">${contentHtml}</div></body>
</html>`;
}

function renderResult(result) {
  state.result = result;
  elements.summaryEmpty.hidden = true;
  elements.summaryPanel.hidden = false;
  elements.summaryTitle.textContent = result.title || "-";
  elements.summarySite.textContent = result.site || "-";
  elements.summaryAuthor.textContent = result.author || "-";
  elements.summaryPublished.textContent = result.published || "-";
  elements.summaryLanguage.textContent = result.language || "-";
  elements.summaryWordCount.textContent = String(result.word_count ?? "-");
  elements.markdownOutput.textContent = result.content_markdown || "";
  elements.htmlOutput.textContent = result.content_html || "";
  elements.jsonOutput.textContent = JSON.stringify(result, null, 2);
  elements.previewFrame.srcdoc = sanitizePreviewDocument(result.content_html || "");
  elements.lastRun.textContent = new Date().toLocaleTimeString([], {
    hour: "numeric",
    minute: "2-digit",
    second: "2-digit",
  });
}

function resetResult() {
  state.result = null;
  elements.summaryEmpty.hidden = false;
  elements.summaryPanel.hidden = true;
  elements.markdownOutput.textContent = "";
  elements.htmlOutput.textContent = "";
  elements.jsonOutput.textContent = "";
  elements.previewFrame.srcdoc = "";
}

function withBusyState(button, label, task) {
  return async () => {
    if (!state.wasmReady) {
      setBanner("The WASM parser is still loading. Try again in a moment.", "warning");
      return;
    }

    const previous = button.textContent;
    button.disabled = true;
    button.textContent = label;

    try {
      await task();
    } catch (error) {
      setBanner(error.message || String(error), "error");
    } finally {
      button.disabled = false;
      button.textContent = previous;
    }
  };
}

function parseHtml(html, url) {
  const trimmed = (html || "").trim();
  if (!trimmed) {
    throw new Error("No HTML was provided. Paste HTML, import a file, or capture a tab first.");
  }

  state.htmlDraft = trimmed;
  state.htmlUrl = url;
  state.fetchUrl = url;
  syncInputsFromState();

  const result = parse(trimmed, url || "https://example.com/article");
  renderResult(result);
  saveState();
  setBanner("Parse complete.", "success");
}

async function fetchPageHtml(url) {
  const response = await fetch(url, {
    method: "GET",
    headers: {
      Accept: "text/html,application/xhtml+xml",
    },
  });

  if (!response.ok) {
    throw new Error(`Fetch failed with status ${response.status}.`);
  }

  const html = await response.text();
  if (!html.trim()) {
    throw new Error("Fetch returned an empty response body.");
  }

  return html;
}

async function importFile(file) {
  const text = await file.text();
  state.importedFileName = file.name;
  state.importedFileContent = text;
  elements.fileName.textContent = file.name;
  saveState();
  setBanner(`Loaded ${file.name}.`, "success");
}

function activeTabPayload() {
  if (!state.result) {
    throw new Error("There is no parser result to copy or download yet.");
  }

  if (state.activeTab === "markdown") {
    return {
      text: state.result.content_markdown || "",
      filename: "defuddle-output.md",
      type: "text/markdown",
    };
  }

  if (state.activeTab === "html") {
    return {
      text: state.result.content_html || "",
      filename: "defuddle-output.html",
      type: "text/html",
    };
  }

  if (state.activeTab === "preview") {
    return {
      text: state.result.content_html || "",
      filename: "defuddle-preview.html",
      type: "text/html",
    };
  }

  return {
    text: JSON.stringify(state.result, null, 2),
    filename: "defuddle-output.json",
    type: "application/json",
  };
}

function parsedPagePayload() {
  if (!state.result) {
    throw new Error("There is no parsed page to copy or export yet.");
  }

  return {
    text: state.result.content_html || "",
    filename: "defuddle-parsed-page.html",
    type: "text/html",
  };
}

async function copyActiveTab() {
  const payload = activeTabPayload();
  await navigator.clipboard.writeText(payload.text);
  setBanner(`Copied ${payload.filename}.`, "success");
}

function downloadActiveTab() {
  const payload = activeTabPayload();
  const blob = new Blob([payload.text], { type: payload.type });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = payload.filename;
  anchor.click();
  URL.revokeObjectURL(url);
  setBanner(`Downloaded ${payload.filename}.`, "success");
}

async function copyParsedPage() {
  const payload = parsedPagePayload();
  await navigator.clipboard.writeText(payload.text);
  setBanner(`Copied ${payload.filename}.`, "success");
}

function exportParsedPage() {
  const payload = parsedPagePayload();
  const blob = new Blob([payload.text], { type: payload.type });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = payload.filename;
  anchor.click();
  URL.revokeObjectURL(url);
  setBanner(`Downloaded ${payload.filename}.`, "success");
}

function updateCaptureSummary(payload) {
  state.latestCapture = payload;
  elements.captureTitle.textContent = payload?.title || "None";
  elements.captureUrl.textContent = payload?.url || "None";
  elements.captureTime.textContent = payload?.capturedAt
    ? new Date(payload.capturedAt).toLocaleTimeString([], {
        hour: "numeric",
        minute: "2-digit",
        second: "2-digit",
      })
    : "Not yet";
}

async function maybeImportLatestCapture() {
  const stored = await chrome.storage.session.get(CAPTURE_KEY);
  const payload = stored[CAPTURE_KEY] || null;
  updateCaptureSummary(payload);

  if (!payload || !payload.html || !state.wasmReady) {
    return;
  }

  parseHtml(payload.html, payload.url || "https://example.com/article");
  setMode("html");
  setBanner(`Imported ${payload.url || "captured tab"} from the extension.`, "success");
}

async function captureActiveTab() {
  const response = await chrome.runtime.sendMessage({
    type: "DEFUDDLE_CAPTURE_ACTIVE_TAB",
  });

  if (!response?.ok) {
    throw new Error(response?.error || "The extension could not capture the active tab.");
  }

  const payload = response.payload;
  updateCaptureSummary(payload);
  parseHtml(payload.html, payload.url || "https://example.com/article");
  setMode("html");
  setBanner(`Captured ${payload.url || "active tab"} into the side panel.`, "success");
}

function wireEvents() {
  chrome.storage.onChanged.addListener((changes, areaName) => {
    if (areaName !== "session" || !changes[CAPTURE_KEY]) {
      return;
    }

    const payload = changes[CAPTURE_KEY].newValue || null;
    updateCaptureSummary(payload);
    if (!payload || !state.wasmReady) {
      return;
    }

    try {
      parseHtml(payload.html, payload.url || "https://example.com/article");
      setMode("html");
      setBanner(`Imported ${payload.url || "captured tab"} from the extension.`, "success");
    } catch (error) {
      resetResult();
      setBanner(error.message, "error");
    }
  });

  document.querySelectorAll(".mode-chip").forEach((button) => {
    button.addEventListener("click", () => setMode(button.dataset.mode));
  });

  elements.captureActiveTab.addEventListener(
    "click",
    withBusyState(elements.captureActiveTab, "Capturing…", async () => {
      clearBanner();
      await captureActiveTab();
    }),
  );

  elements.githubLink.addEventListener("click", async (event) => {
    event.preventDefault();
    await chrome.tabs.create({
      url: REPO_URL,
      active: true,
    });
  });

  document.querySelectorAll(".tab").forEach((button) => {
    button.addEventListener("click", () => setActiveTab(button.dataset.tab));
  });

  elements.htmlInput.addEventListener("input", (event) => {
    state.htmlDraft = event.target.value;
    updateRuntimeCard();
    saveState();
  });

  elements.htmlUrlInput.addEventListener("input", (event) => {
    state.htmlUrl = event.target.value;
    saveState();
  });

  elements.urlInput.addEventListener("input", (event) => {
    state.fetchUrl = event.target.value;
    saveState();
  });

  document.querySelector("#load-sample").addEventListener("click", () => {
    state.htmlDraft = SAMPLE_HTML;
    state.htmlUrl = SAMPLE_URL;
    state.fetchUrl = SAMPLE_URL;
    syncInputsFromState();
    setMode("html");
    clearBanner();
    setBanner("Sample article loaded into HTML mode.", "info");
    saveState();
  });

  document.querySelector("#clear-html-button").addEventListener("click", () => {
    state.htmlDraft = "";
    syncInputsFromState();
    resetResult();
    saveState();
    clearBanner();
  });

  const fetchParseButton = document.querySelector("#fetch-parse-button");
  fetchParseButton.addEventListener(
    "click",
    withBusyState(fetchParseButton, "Fetching…", async () => {
      clearBanner();
      const url = state.fetchUrl.trim();
      if (!url) {
        throw new Error("Enter a URL before fetching.");
      }

      try {
        const html = await fetchPageHtml(url);
        parseHtml(html, url);
      } catch (error) {
        resetResult();
        setBanner(error.message, "warning");
      }
    }),
  );

  const parseHtmlButton = document.querySelector("#parse-html-button");
  parseHtmlButton.addEventListener(
    "click",
    withBusyState(parseHtmlButton, "Parsing…", async () => {
      clearBanner();
      try {
        parseHtml(state.htmlDraft, readCurrentUrl());
      } catch (error) {
        resetResult();
        setBanner(error.message, "error");
      }
    }),
  );

  const directParseButton = document.querySelector("#direct-parse-button");
  directParseButton.addEventListener(
    "click",
    withBusyState(directParseButton, "Parsing…", async () => {
      clearBanner();
      try {
        parseHtml(state.htmlDraft, readCurrentUrl());
      } catch (error) {
        resetResult();
        setBanner(error.message, "error");
      }
    }),
  );

  const parseFileButton = document.querySelector("#parse-file-button");
  parseFileButton.addEventListener(
    "click",
    withBusyState(parseFileButton, "Parsing…", async () => {
      clearBanner();
      if (!state.importedFileContent) {
        throw new Error("Import an HTML file before parsing.");
      }

      try {
        parseHtml(state.importedFileContent, readCurrentUrl());
      } catch (error) {
        resetResult();
        setBanner(error.message, "error");
      }
    }),
  );

  document.querySelector("#use-file-as-draft-button").addEventListener("click", () => {
    if (!state.importedFileContent) {
      setBanner("Import a file first.", "warning");
      return;
    }

    state.htmlDraft = state.importedFileContent;
    syncInputsFromState();
    setMode("html");
    setBanner("Imported file moved into the HTML draft.", "success");
    saveState();
  });

  document.querySelector("#copy-page-button").addEventListener("click", async () => {
    try {
      await copyParsedPage();
    } catch (error) {
      setBanner(error.message, "error");
    }
  });

  document.querySelector("#export-page-button").addEventListener("click", () => {
    try {
      exportParsedPage();
    } catch (error) {
      setBanner(error.message, "error");
    }
  });

  elements.fileInput.addEventListener("change", async (event) => {
    const [file] = event.target.files || [];
    if (!file) {
      return;
    }

    try {
      await importFile(file);
    } catch (error) {
      setBanner(error.message, "error");
    }
  });

  const dropzone = document.querySelector("#dropzone");
  ["dragenter", "dragover"].forEach((type) => {
    dropzone.addEventListener(type, (event) => {
      event.preventDefault();
      dropzone.classList.add("is-dragging");
    });
  });
  ["dragleave", "drop"].forEach((type) => {
    dropzone.addEventListener(type, (event) => {
      event.preventDefault();
      dropzone.classList.remove("is-dragging");
    });
  });
  dropzone.addEventListener("drop", async (event) => {
    const [file] = event.dataTransfer?.files || [];
    if (!file) {
      return;
    }

    try {
      await importFile(file);
    } catch (error) {
      setBanner(error.message, "error");
    }
  });
}

async function boot() {
  restoreState();
  syncInputsFromState();
  wireEvents();
  resetResult();
  setBanner("Loading the WASM parser…", "info");

  try {
    await initDefuddleWasm();
    state.wasmReady = true;
    elements.wasmState.textContent = "Ready";
    setBanner("WASM parser ready.", "success");
  } catch (error) {
    elements.wasmState.textContent = "Failed";
    setBanner(`Failed to initialize the WASM parser: ${error.message}`, "error");
    return;
  }

  if (state.htmlDraft.trim()) {
    setBanner("Restored your previous HTML draft from local storage.", "info");
  }

  await maybeImportLatestCapture();
}

boot();
