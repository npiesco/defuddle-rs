const PANEL_PATH = "panel.html";
const CAPTURE_KEY = "defuddle_latest_capture";

function extensionPagePrefix() {
  return chrome.runtime.getURL("");
}

function panelUrl() {
  return chrome.runtime.getURL(PANEL_PATH);
}

function isExtensionPage(url) {
  return typeof url === "string" && url.startsWith(extensionPagePrefix());
}

async function getOrCreatePanelTab() {
  const tabs = await chrome.tabs.query({});
  const existing = tabs.find((tab) => tab.url && tab.url.startsWith(panelUrl()));
  if (existing && existing.id != null) {
    await chrome.tabs.update(existing.id, { active: true });
    if (existing.windowId != null) {
      await chrome.windows.update(existing.windowId, { focused: true });
    }
    return existing.id;
  }

  const created = await chrome.tabs.create({ url: panelUrl(), active: true });
  return created.id;
}

async function storeCapture(payload) {
  await chrome.storage.session.set({
    [CAPTURE_KEY]: payload,
  });
}

async function captureTab(tabId) {
  if (tabId == null) {
    return null;
  }

  try {
    const response = await chrome.tabs.sendMessage(tabId, {
      type: "DEFUDDLE_CAPTURE_PAGE",
    });
    return response && response.ok ? response.payload : null;
  } catch (_error) {
    return null;
  }
}

async function getCurrentPageTab() {
  const tabs = await chrome.tabs.query({
    active: true,
    lastFocusedWindow: true,
  });
  return tabs.find((tab) => tab.id && tab.url && !isExtensionPage(tab.url)) || null;
}

async function handleCaptureFromTab(tab, { openPanel = true } = {}) {
  if (!tab?.id || !tab.url || isExtensionPage(tab.url)) {
    return;
  }

  const payload = await captureTab(tab.id);
  if (!payload) {
    return;
  }

  await storeCapture(payload);
  if (openPanel) {
    await chrome.sidePanel.open({ windowId: tab.windowId });
  }
}

async function enableSidePanelAction() {
  try {
    await chrome.sidePanel.setPanelBehavior({ openPanelOnActionClick: true });
  } catch (_error) {
    // Ignore when the API is unavailable in older Chromium builds.
  }
}

chrome.runtime.onInstalled.addListener(enableSidePanelAction);
chrome.runtime.onStartup.addListener(enableSidePanelAction);
enableSidePanelAction();

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (!message || typeof message !== "object") {
    return undefined;
  }

  if (message.type === "DEFUDDLE_CAPTURE_ACTIVE_TAB") {
    (async () => {
      const tab = await getCurrentPageTab();
      if (!tab) {
        sendResponse({ ok: false, error: "No active page tab was available to capture." });
        return;
      }

      const payload = await captureTab(tab.id);
      if (!payload) {
        sendResponse({ ok: false, error: "The extension could not capture the current tab." });
        return;
      }

      await storeCapture(payload);
      sendResponse({ ok: true, payload });
    })();

    return true;
  }

  if (message.type === "DEFUDDLE_OPEN_PANEL_FOR_ACTIVE_TAB") {
    (async () => {
      const tab = await getCurrentPageTab();
      if (!tab) {
        sendResponse({ ok: false, error: "No active page tab was available." });
        return;
      }

      await chrome.sidePanel.open({ windowId: tab.windowId });
      sendResponse({ ok: true });
    })();

    return true;
  }

  return undefined;
});
