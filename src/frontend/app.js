// IPC Bridge
function sendToRust(command, data) {
  var msg = JSON.stringify(Object.assign({ command: command }, data || {}));
  window.ipc.postMessage(msg);
}

// Rust calls this to send events to JS
window.__fromRust = function(event, data) {
  switch (event) {
    case 'file_opened':
      addRecentFile(data.path);
      TabManager.createTab(data.path, data.content);
      break;
    case 'file_saved':
      TabManager.markClean();
      if (data.path) {
        TabManager.updateTabPath(null, data.path);
      }
      onFileSaved();
      break;
    case 'stdin_opened':
      TabManager.createTab(null, data.content, 'preview', data.title || 'stdin');
      break;
    case 'error':
      showError(data.message);
      break;
  }
};

// State
var currentMode = 'edit';
var splitMode = false;

// Cross-mode selection helpers
function selectInPreview(text, ratio) {
  var preview = document.getElementById('preview');
  var walker = document.createTreeWalker(preview, NodeFilter.SHOW_TEXT);
  var nodes = [], node, fullText = '';
  while (node = walker.nextNode()) {
    nodes.push({ node: node, start: fullText.length });
    fullText += node.textContent;
  }
  if (!nodes.length) return false;
  var textLower = text.toLowerCase(), fullLower = fullText.toLowerCase();
  var occurrences = [], idx = 0;
  while ((idx = fullLower.indexOf(textLower, idx)) !== -1) {
    occurrences.push(idx);
    idx += 1;
  }
  if (!occurrences.length) return false;
  var targetPos = ratio * fullText.length;
  var best = occurrences.reduce(function(a, b) {
    return Math.abs(b - targetPos) < Math.abs(a - targetPos) ? b : a;
  });
  var startPos = best, endPos = best + text.length;
  var startNode, startOffset, endNode, endOffset;
  for (var i = 0; i < nodes.length; i++) {
    var ns = nodes[i].start, ne = ns + nodes[i].node.textContent.length;
    if (!startNode && startPos >= ns && startPos < ne) {
      startNode = nodes[i].node; startOffset = startPos - ns;
    }
    if (endPos >= ns && endPos <= ne) {
      endNode = nodes[i].node; endOffset = endPos - ns;
    }
  }
  if (!startNode || !endNode) return false;
  var range = document.createRange();
  range.setStart(startNode, startOffset);
  range.setEnd(endNode, endOffset);
  var sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(range);
  if (startNode.parentElement) startNode.parentElement.scrollIntoView({ block: 'center' });
  return true;
}

function selectInEditor(text, ratio) {
  var editor = document.getElementById('editor');
  var valueLower = editor.value.toLowerCase(), textLower = text.toLowerCase();
  var occurrences = [], idx = 0;
  while ((idx = valueLower.indexOf(textLower, idx)) !== -1) {
    occurrences.push(idx);
    idx += 1;
  }
  if (!occurrences.length) return false;
  var targetPos = ratio * editor.value.length;
  var best = occurrences.reduce(function(a, b) {
    return Math.abs(b - targetPos) < Math.abs(a - targetPos) ? b : a;
  });
  editor.selectionStart = best;
  editor.selectionEnd = best + text.length;
  var lines = editor.value.substring(0, best).split('\n');
  var approxLine = lines.length - 1;
  var totalLines = editor.value.split('\n').length;
  editor.scrollTop = (approxLine / totalLines) * editor.scrollHeight - editor.clientHeight / 3;
  return true;
}

// Mode Toggle
function toggleMode() {
  if (splitMode) {
    splitMode = false;
    document.body.classList.remove('split-mode');
    document.getElementById('btn-split').classList.remove('active');
  }

  var iconPreview = document.getElementById('icon-preview');
  var iconEdit = document.getElementById('icon-edit');

  if (currentMode === 'edit') {
    var editor = document.getElementById('editor');
    var content = editor.value;
    var selectedText = content.substring(editor.selectionStart, editor.selectionEnd);
    var scrollRatio = content.length > 0 ? editor.selectionStart / content.length : 0;
    document.getElementById('preview').innerHTML = marked.parse(content);
    resolveLocalImages();
    document.getElementById('editor-container').classList.remove('active');
    document.getElementById('preview-container').classList.add('active');
    document.getElementById('btn-toggle').classList.add('active');
    document.getElementById('status-mode').textContent = 'PREVIEW';
    iconPreview.style.display = 'none';
    iconEdit.style.display = '';
    currentMode = 'preview';
    var pc = document.getElementById('preview-container');
    setTimeout(function() {
      if (!selectedText || !selectInPreview(selectedText, scrollRatio)) {
        pc.scrollTop = scrollRatio * (pc.scrollHeight - pc.clientHeight);
      }
    }, 0);
    if (findState.open) doFind(document.getElementById('find-input').value);
  } else {
    var sel = window.getSelection();
    var selectedText = sel.toString();
    var pc = document.getElementById('preview-container');
    var scrollRatio = pc.scrollHeight > pc.clientHeight ? pc.scrollTop / (pc.scrollHeight - pc.clientHeight) : 0;
    document.getElementById('preview-container').classList.remove('active');
    document.getElementById('editor-container').classList.add('active');
    document.getElementById('btn-toggle').classList.remove('active');
    document.getElementById('status-mode').textContent = 'EDIT';
    iconPreview.style.display = '';
    iconEdit.style.display = 'none';
    currentMode = 'edit';
    var editor = document.getElementById('editor');
    editor.focus();
    if (!selectedText || !selectInEditor(selectedText, scrollRatio)) {
      var pos = Math.round(scrollRatio * editor.value.length);
      editor.selectionStart = editor.selectionEnd = pos;
      editor.scrollTop = scrollRatio * (editor.scrollHeight - editor.clientHeight);
    }
    if (findState.open) doFind(document.getElementById('find-input').value);
  }
}

function setTitle(title) {
  document.getElementById('titlebar-title').textContent = title;
}

function onFileSaved() {
  var info = document.getElementById('status-info');
  info.textContent = 'Saved';
  setTimeout(function() { info.textContent = ''; }, 2000);
}

function showError(message) {
  var info = document.getElementById('status-info');
  info.textContent = 'Error: ' + message;
  info.style.color = '#c15050';
  setTimeout(function() { info.textContent = ''; info.style.color = ''; }, 5000);
}

// Split View
function toggleSplit() {
  var iconPreview = document.getElementById('icon-preview');
  var iconEdit = document.getElementById('icon-edit');

  if (splitMode) {
    splitMode = false;
    document.body.classList.remove('split-mode');
    document.getElementById('btn-split').classList.remove('active');
    document.getElementById('preview-container').classList.remove('active');
    currentMode = 'edit';
    document.getElementById('btn-toggle').classList.remove('active');
    document.getElementById('status-mode').textContent = 'EDIT';
    iconPreview.style.display = '';
    iconEdit.style.display = 'none';
    document.getElementById('editor').focus();
  } else {
    splitMode = true;
    document.body.classList.add('split-mode');
    document.getElementById('btn-split').classList.add('active');
    document.getElementById('editor-container').classList.add('active');
    document.getElementById('preview-container').classList.add('active');
    document.getElementById('preview').innerHTML = marked.parse(document.getElementById('editor').value);
    resolveLocalImages();
    currentMode = 'edit';
    document.getElementById('btn-toggle').classList.remove('active');
    document.getElementById('status-mode').textContent = 'SPLIT';
    iconPreview.style.display = '';
    iconEdit.style.display = 'none';
    document.getElementById('editor').focus();
  }
}

var splitPreviewTimer = null;
function updateSplitPreview() {
  if (!splitMode) return;
  clearTimeout(splitPreviewTimer);
  splitPreviewTimer = setTimeout(function() {
    document.getElementById('preview').innerHTML = marked.parse(document.getElementById('editor').value);
    resolveLocalImages();
  }, 150);
}

// Word count
function updateWordCount() {
  var text = document.getElementById('editor').value;
  var words = text.trim() ? text.trim().split(/\s+/).length : 0;
  document.getElementById('status-counts').textContent = words + ' word' + (words !== 1 ? 's' : '');
}

// Recent Files
function getRecentFiles() {
  try { return JSON.parse(localStorage.getItem('peekdown-recent')) || []; } catch(e) { return []; }
}

function addRecentFile(path) {
  if (!path) return;
  var recent = getRecentFiles();
  var filename = path.split(/[/\\]/).pop();
  recent = recent.filter(function(r) { return r.path.replace(/\\/g, '/').toLowerCase() !== path.replace(/\\/g, '/').toLowerCase(); });
  recent.unshift({ path: path, filename: filename });
  if (recent.length > 10) recent = recent.slice(0, 10);
  try { localStorage.setItem('peekdown-recent', JSON.stringify(recent)); } catch(e) {}
}

function showRecentPanel() {
  var panel = document.getElementById('recent-panel');
  var tab = TabManager.getActiveTab();
  if (!tab || tab.path || tab.dirty || tab.content !== '') {
    panel.classList.remove('visible');
    return;
  }
  var recent = getRecentFiles();
  if (recent.length === 0) { panel.classList.remove('visible'); return; }
  panel.innerHTML = '';
  var title = document.createElement('div');
  title.className = 'recent-title';
  title.textContent = 'Recent Files';
  panel.appendChild(title);
  recent.forEach(function(r) {
    var item = document.createElement('div');
    item.className = 'recent-item';
    var name = document.createElement('span');
    name.className = 'recent-name';
    name.textContent = r.filename;
    var path = document.createElement('span');
    path.className = 'recent-path';
    path.textContent = r.path;
    item.appendChild(name);
    item.appendChild(path);
    item.addEventListener('click', function() {
      sendToRust('open_file', { path: r.path });
    });
    panel.appendChild(item);
  });
  panel.classList.add('visible');
}

// Table of Contents
var tocOpen = false;

function parseTOC(text) {
  var lines = text.split('\n');
  var headings = [];
  var inCodeBlock = false;
  for (var i = 0; i < lines.length; i++) {
    if (/^```/.test(lines[i])) { inCodeBlock = !inCodeBlock; continue; }
    if (inCodeBlock) continue;
    var match = lines[i].match(/^(#{1,6})\s+(.+)/);
    if (match) {
      headings.push({ level: match[1].length, text: match[2].replace(/[#*_`\[\]]/g, '').trim(), line: i });
    }
  }
  return headings;
}

function updateTOC() {
  var list = document.getElementById('toc-list');
  var text = document.getElementById('editor').value;
  var headings = parseTOC(text);
  list.innerHTML = '';
  if (headings.length === 0) {
    var empty = document.createElement('div');
    empty.className = 'toc-empty';
    empty.textContent = 'No headings';
    list.appendChild(empty);
    return;
  }
  headings.forEach(function(h, idx) {
    var item = document.createElement('div');
    item.className = 'toc-item toc-h' + h.level;
    item.textContent = h.text;
    item.addEventListener('click', function() {
      if (currentMode === 'edit') {
        scrollEditorToLine(h.line);
      } else {
        scrollPreviewToHeading(idx);
      }
    });
    list.appendChild(item);
  });
}

function scrollEditorToLine(lineNum) {
  var editor = document.getElementById('editor');
  var lines = editor.value.split('\n');
  var pos = 0;
  for (var i = 0; i < lineNum && i < lines.length; i++) {
    pos += lines[i].length + 1;
  }
  editor.focus();
  editor.selectionStart = pos;
  editor.selectionEnd = pos + (lines[lineNum] || '').length;
  var approxLineHeight = editor.scrollHeight / lines.length;
  editor.scrollTop = lineNum * approxLineHeight - editor.clientHeight / 3;
}

function scrollPreviewToHeading(idx) {
  var headings = document.getElementById('preview').querySelectorAll('h1,h2,h3,h4,h5,h6');
  if (headings[idx]) {
    headings[idx].scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
}

function toggleTOC() {
  tocOpen = !tocOpen;
  document.getElementById('toc-panel').classList.toggle('open', tocOpen);
  document.getElementById('btn-toc').classList.toggle('active', tocOpen);
  if (tocOpen) updateTOC();
}

function doSave() {
  var tab = TabManager.getActiveTab();
  var data = { content: document.getElementById('editor').value };
  if (tab && tab.path) data.path = tab.path;
  sendToRust('save_file', data);
}

// Zoom
var zoomLevel = 1;
var ZOOM_STEP = 0.1;
var ZOOM_MIN = 0.5;
var ZOOM_MAX = 3;

function applyZoom(level) {
  zoomLevel = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, level));
  document.documentElement.style.setProperty('--zoom', zoomLevel);
  var toast = document.getElementById('zoom-toast');
  toast.textContent = Math.round(zoomLevel * 100) + '%';
  toast.classList.add('visible');
  clearTimeout(applyZoom._timer);
  applyZoom._timer = setTimeout(function() {
    toast.classList.remove('visible');
  }, 800);
}

document.addEventListener('wheel', function(e) {
  if (e.ctrlKey) {
    e.preventDefault();
    applyZoom(zoomLevel + (e.deltaY < 0 ? ZOOM_STEP : -ZOOM_STEP));
  }
}, { passive: false });

// Preview width resize
(function() {
  var handle = document.getElementById('preview-resize-handle');
  var preview = document.getElementById('preview-wrapper');
  var container = document.getElementById('preview-container');
  var DEFAULT_WIDTH = 720;
  var MIN_WIDTH = 300;

  var saved = null;
  try { saved = localStorage.getItem('peekdown-preview-width'); } catch(e) {}
  if (saved) preview.style.maxWidth = saved + 'px';

  var dragging = false;

  handle.addEventListener('mousedown', function(e) {
    e.preventDefault();
    dragging = true;
    handle.classList.add('dragging');
    document.body.classList.add('preview-resizing');
  });

  document.addEventListener('mousemove', function(e) {
    if (!dragging) return;
    var containerRect = container.getBoundingClientRect();
    var centerX = containerRect.left + containerRect.width / 2;
    var width = Math.max(MIN_WIDTH, (e.clientX - centerX) * 2);
    width = Math.min(width, containerRect.width);
    preview.style.maxWidth = Math.round(width) + 'px';
  });

  document.addEventListener('mouseup', function() {
    if (!dragging) return;
    dragging = false;
    handle.classList.remove('dragging');
    document.body.classList.remove('preview-resizing');
    try { localStorage.setItem('peekdown-preview-width', parseInt(preview.style.maxWidth)); } catch(e) {}
  });

  handle.addEventListener('dblclick', function() {
    preview.style.maxWidth = DEFAULT_WIDTH + 'px';
    try { localStorage.setItem('peekdown-preview-width', DEFAULT_WIDTH); } catch(e) {}
  });
})();

// Find
var findState = { open: false, matches: [], current: -1, marks: [] };

function openFind() {
  document.getElementById('find-bar').classList.add('open');
  findState.open = true;
  var input = document.getElementById('find-input');
  input.focus();
  input.select();
  if (input.value) doFind(input.value);
}

function closeFind() {
  document.getElementById('find-bar').classList.remove('open');
  findState.open = false;
  findState.matches = [];
  findState.current = -1;
  clearPreviewHighlights();
  document.getElementById('find-count').textContent = '';
  if (currentMode === 'edit') document.getElementById('editor').focus();
}

function doFind(term) {
  findState.matches = [];
  findState.current = -1;
  clearPreviewHighlights();
  if (!term) {
    document.getElementById('find-count').textContent = '';
    return;
  }
  if (currentMode === 'edit') {
    var text = document.getElementById('editor').value.toLowerCase();
    var termLower = term.toLowerCase();
    var idx = 0;
    while ((idx = text.indexOf(termLower, idx)) !== -1) {
      findState.matches.push({ start: idx, end: idx + term.length });
      idx += term.length;
    }
  } else {
    var preview = document.getElementById('preview');
    var walker = document.createTreeWalker(preview, NodeFilter.SHOW_TEXT);
    var node, ranges = [], termLower = term.toLowerCase();
    while (node = walker.nextNode()) {
      var nodeText = node.textContent.toLowerCase();
      var idx = 0;
      while ((idx = nodeText.indexOf(termLower, idx)) !== -1) {
        var range = document.createRange();
        range.setStart(node, idx);
        range.setEnd(node, idx + term.length);
        ranges.push(range);
        idx += term.length;
      }
    }
    for (var i = ranges.length - 1; i >= 0; i--) {
      var mark = document.createElement('mark');
      mark.className = 'find-match';
      ranges[i].surroundContents(mark);
      findState.marks.unshift(mark);
    }
    findState.matches = findState.marks.map(function(_, i) { return i; });
  }
  if (findState.matches.length > 0) {
    findState.current = 0;
    goToMatch(0);
  }
  updateFindCount();
}

function clearPreviewHighlights() {
  findState.marks.forEach(function(mark) {
    var parent = mark.parentNode;
    if (!parent) return;
    while (mark.firstChild) parent.insertBefore(mark.firstChild, mark);
    parent.removeChild(mark);
    parent.normalize();
  });
  findState.marks = [];
}

function goToMatch(idx) {
  findState.current = idx;
  if (currentMode === 'edit') {
    var match = findState.matches[idx];
    var editor = document.getElementById('editor');
    editor.focus();
    editor.selectionStart = match.start;
    editor.selectionEnd = match.end;
  } else {
    findState.marks.forEach(function(m) { m.classList.remove('find-active'); });
    var mark = findState.marks[idx];
    mark.classList.add('find-active');
    mark.scrollIntoView({ block: 'center' });
  }
  updateFindCount();
}

function findNext() {
  if (findState.matches.length === 0) return;
  goToMatch((findState.current + 1) % findState.matches.length);
}

function findPrev() {
  if (findState.matches.length === 0) return;
  goToMatch((findState.current - 1 + findState.matches.length) % findState.matches.length);
}

function updateFindCount() {
  var el = document.getElementById('find-count');
  if (findState.matches.length === 0) {
    el.textContent = document.getElementById('find-input').value ? 'No results' : '';
  } else {
    el.textContent = (findState.current + 1) + ' of ' + findState.matches.length;
  }
}

document.getElementById('find-input').addEventListener('input', function() {
  doFind(this.value);
});
document.getElementById('find-input').addEventListener('keydown', function(e) {
  if (e.key === 'Escape') { closeFind(); e.preventDefault(); }
  else if (e.key === 'Enter' && !e.shiftKey) { findNext(); e.preventDefault(); }
  else if (e.key === 'Enter' && e.shiftKey) { findPrev(); e.preventDefault(); }
});
document.getElementById('find-close').addEventListener('click', closeFind);
document.getElementById('find-next').addEventListener('click', findNext);
document.getElementById('find-prev').addEventListener('click', findPrev);

// Keyboard Shortcuts
document.addEventListener('keydown', function(e) {
  if (e.ctrlKey && e.key === 'f') {
    e.preventDefault();
    openFind();
  } else if (e.key === 'Escape' && findState.open) {
    e.preventDefault();
    closeFind();
  } else if (e.ctrlKey && e.key === 'o') {
    e.preventDefault();
    sendToRust('open_file');
  } else if (e.ctrlKey && !e.shiftKey && e.key === 's') {
    e.preventDefault();
    doSave();
  } else if (e.ctrlKey && e.shiftKey && (e.key === 'S' || e.key === 's')) {
    e.preventDefault();
    sendToRust('save_as', { content: document.getElementById('editor').value });
  } else if (e.ctrlKey && e.key === 'e') {
    e.preventDefault();
    toggleMode();
  } else if (e.ctrlKey && e.key === 'n') {
    e.preventDefault();
    TabManager.createTab(null, '');
  } else if (e.ctrlKey && e.key === 'w') {
    e.preventDefault();
    var active = TabManager.getActiveTab();
    if (active) TabManager.closeTab(active.id);
  } else if (e.ctrlKey && !e.shiftKey && e.key === 'Tab') {
    e.preventDefault();
    TabManager.nextTab();
  } else if (e.ctrlKey && e.shiftKey && e.key === 'Tab') {
    e.preventDefault();
    TabManager.prevTab();
  } else if (e.ctrlKey && (e.key === '=' || e.key === '+')) {
    e.preventDefault();
    applyZoom(zoomLevel + ZOOM_STEP);
  } else if (e.ctrlKey && e.key === '-') {
    e.preventDefault();
    applyZoom(zoomLevel - ZOOM_STEP);
  } else if (e.ctrlKey && e.key === '0') {
    e.preventDefault();
    applyZoom(1);
  } else if (e.ctrlKey && e.key === '\\') {
    e.preventDefault();
    toggleSplit();
  } else if (e.ctrlKey && e.shiftKey && (e.key === 'O' || e.key === 'o')) {
    e.preventDefault();
    toggleTOC();
  }
});

// Window Controls
document.getElementById('btn-minimize').addEventListener('click', function() { sendToRust('window_minimize'); });
document.getElementById('btn-maximize').addEventListener('click', function() { sendToRust('window_maximize'); });
document.getElementById('btn-close').addEventListener('click', function() {
  if (TabManager.hasAnyDirty()) {
    if (!confirm('You have unsaved changes. Close anyway?')) return;
  }
  sendToRust('window_close');
});

// Toolbar Buttons
document.getElementById('btn-new').addEventListener('click', function() { TabManager.createTab(null, ''); });
document.getElementById('btn-open').addEventListener('click', function() { sendToRust('open_file'); });
document.getElementById('btn-save').addEventListener('click', doSave);
document.getElementById('btn-toggle').addEventListener('click', toggleMode);
document.getElementById('btn-split').addEventListener('click', toggleSplit);
document.getElementById('btn-toc').addEventListener('click', toggleTOC);

// Theme Toggle
function setTheme(theme) {
  document.documentElement.setAttribute('data-theme', theme);
  document.getElementById('icon-sun').style.display = theme === 'light' ? '' : 'none';
  document.getElementById('icon-moon').style.display = theme === 'light' ? 'none' : '';
  try { localStorage.setItem('peekdown-theme', theme); } catch(e) {}
}

document.getElementById('btn-theme').addEventListener('click', function() {
  var current = document.documentElement.getAttribute('data-theme') || 'dark';
  setTheme(current === 'dark' ? 'light' : 'dark');
});

// Init
document.addEventListener('DOMContentLoaded', function() {
  var saved = null;
  try { saved = localStorage.getItem('peekdown-theme'); } catch(e) {}
  if (saved) setTheme(saved);
  TabManager.createTab(null, '');
  updateWordCount();
  showRecentPanel();
  sendToRust('ready');
});
