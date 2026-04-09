const PANEL_PATH = "panel.html";

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
    defuddle_latest_capture: payload,
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

async function handleCaptureFromTab(tab) {
  if (!tab.id || !tab.url || isExtensionPage(tab.url)) {
    return;
  }

  const payload = await captureTab(tab.id);
  if (!payload) {
    return;
  }

  await storeCapture(payload);
  await getOrCreatePanelTab();
}

chrome.action.onClicked.addListener(async (tab) => {
  await handleCaptureFromTab(tab);
});
