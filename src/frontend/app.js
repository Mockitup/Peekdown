// IPC Bridge
function sendToRust(command, data) {
  var msg = JSON.stringify(Object.assign({ command: command }, data || {}));
  window.ipc.postMessage(msg);
}

// Rust calls this to send events to JS
window.__fromRust = function(event, data) {
  switch (event) {
    case 'file_opened':
      TabManager.createTab(data.path, data.content);
      break;
    case 'file_saved':
      TabManager.markClean();
      if (data.path) {
        TabManager.updateTabPath(null, data.path);
      }
      onFileSaved();
      break;
    case 'error':
      showError(data.message);
      break;
  }
};

// State
var currentMode = 'edit';

// Mode Toggle
function toggleMode() {
  var iconPreview = document.getElementById('icon-preview');
  var iconEdit = document.getElementById('icon-edit');

  if (currentMode === 'edit') {
    var content = document.getElementById('editor').value;
    document.getElementById('preview').innerHTML = marked.parse(content);
    document.getElementById('editor-container').classList.remove('active');
    document.getElementById('preview-container').classList.add('active');
    document.getElementById('btn-toggle').classList.add('active');
    document.getElementById('status-mode').textContent = 'PREVIEW';
    iconPreview.style.display = 'none';
    iconEdit.style.display = '';
    currentMode = 'preview';
  } else {
    document.getElementById('preview-container').classList.remove('active');
    document.getElementById('editor-container').classList.add('active');
    document.getElementById('btn-toggle').classList.remove('active');
    document.getElementById('status-mode').textContent = 'EDIT';
    iconPreview.style.display = '';
    iconEdit.style.display = 'none';
    currentMode = 'edit';
    document.getElementById('editor').focus();
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

function doSave() {
  var tab = TabManager.getActiveTab();
  var data = { content: document.getElementById('editor').value };
  if (tab && tab.path) data.path = tab.path;
  sendToRust('save_file', data);
}

// Keyboard Shortcuts
document.addEventListener('keydown', function(e) {
  if (e.ctrlKey && e.key === 'o') {
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

// Init
document.addEventListener('DOMContentLoaded', function() {
  TabManager.createTab(null, '');
  sendToRust('ready');
});
