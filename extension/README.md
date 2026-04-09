# Defuddle RS Browser Extension

This extension captures the full HTML of the current browser tab and opens the Defuddle RS extension UI.

## Intended flow

1. Open any page you want to parse.
2. Click the `Defuddle RS Capture` extension action.
3. The extension opens or focuses `panel.html`.
4. The extension UI imports the captured HTML and parses it automatically.

## Local development

Load this folder as an unpacked extension in Chromium-based browsers:

1. Open `chrome://extensions`
2. Enable `Developer mode`
3. Click `Load unpacked`
4. Select the `extension/` directory from this repo

After rebuilding the Rust WASM package, refresh the extension bundle with:

```bash
npm run build:extension
```
