// Configure marked.js with highlight.js via custom renderer
var renderer = new marked.Renderer();
renderer.code = function(token) {
  var lang = (token.lang || '').trim();
  var code = token.text;
  var highlighted;
  if (lang && hljs.getLanguage(lang)) {
    highlighted = hljs.highlight(code, { language: lang }).value;
  } else {
    highlighted = hljs.highlightAuto(code).value;
  }
  var cls = lang ? ' class="language-' + lang + '"' : '';
  return '<pre><code' + cls + '>' + highlighted + '</code></pre>';
};

marked.setOptions({
  breaks: true,
  gfm: true,
  renderer: renderer,
});

// Post-process: resolve local images via IPC
function resolveLocalImages() {
  var tab = typeof TabManager !== 'undefined' ? TabManager.getActiveTab() : null;
  if (!tab || !tab.path) return;
  var dir = tab.path.replace(/[/\\][^/\\]*$/, '');
  var imgs = document.getElementById('preview').querySelectorAll('img');
  for (var i = 0; i < imgs.length; i++) {
    var src = imgs[i].getAttribute('src');
    if (!src || /^https?:\/\/|^data:/i.test(src)) continue;
    if (imgs[i].getAttribute('data-local-src')) continue;
    // Decode URL-encoded characters (%20 → space, %5C → backslash, etc.)
    var decoded = decodeURIComponent(src);
    // Strip file:// prefix if present
    decoded = decoded.replace(/^file:\/\/\/?/, '');
    // Detect absolute paths (D:/... or /...) vs relative
    var absPath;
    if (/^[a-zA-Z]:[\\/]/.test(decoded) || decoded.startsWith('/')) {
      absPath = decoded;
    } else {
      absPath = dir + '/' + decoded.replace(/^\.\//, '');
    }
    // Normalize to forward slashes
    absPath = absPath.replace(/\\/g, '/');
    imgs[i].setAttribute('data-local-src', absPath);
    imgs[i].removeAttribute('src');
    sendToRust('read_image', { path: absPath });
  }
}

// Rust calls this with base64 data
window.__setImage = function(path, dataUri) {
  var imgs = document.querySelectorAll('img[data-local-src]');
  for (var i = 0; i < imgs.length; i++) {
    if (imgs[i].getAttribute('data-local-src') === path) {
      imgs[i].src = dataUri;
    }
  }
};
