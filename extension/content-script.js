chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (!message || message.type !== "DEFUDDLE_CAPTURE_PAGE") {
    return undefined;
  }

  sendResponse({
    ok: true,
    payload: {
      html: document.documentElement ? document.documentElement.outerHTML : "",
      url: window.location.href,
      title: document.title || "",
      capturedAt: new Date().toISOString(),
    },
  });

  return true;
});
