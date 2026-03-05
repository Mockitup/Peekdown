var TabManager = (function() {
  var tabs = [];
  var activeTabId = null;
  var tabIdCounter = 0;

  function normalizePath(p) {
    return p.replace(/\\/g, '/');
  }

  function createTab(path, content) {
    if (path) {
      var existing = findTabByPath(path);
      if (existing) {
        switchTab(existing.id);
        return existing;
      }
    }

    var active = getActiveTab();
    if (active && !active.path && !active.dirty && active.content === '' && path) {
      active.path = normalizePath(path);
      active.filename = path.split(/[/\\]/).pop();
      active.content = content != null ? content : '';
      active.dirty = false;
      active.mode = 'preview';
      restoreTabState(active);
      renderTabBar();
      updateWindowTitle();
      return active;
    }

    var id = ++tabIdCounter;
    var tab = {
      id: id,
      path: path ? normalizePath(path) : null,
      filename: path ? path.split(/[/\\]/).pop() : 'Untitled',
      content: content != null ? content : '',
      dirty: false,
      mode: path ? 'preview' : 'edit',
      scrollTop: 0,
      cursorStart: 0,
      cursorEnd: 0
    };
    tabs.push(tab);
    switchTab(id);
    return tab;
  }

  function closeTab(id) {
    var idx = tabs.findIndex(function(t) { return t.id === id; });
    if (idx === -1) return;
    var tab = tabs[idx];
    if (tab.dirty) {
      if (!confirm('Unsaved changes in "' + tab.filename + '". Close anyway?')) return;
    }
    tabs.splice(idx, 1);
    if (tabs.length === 0) {
      createTab(null, '');
      return;
    }
    if (activeTabId === id) {
      var newIdx = Math.min(idx, tabs.length - 1);
      switchTab(tabs[newIdx].id);
    } else {
      renderTabBar();
    }
  }

  function switchTab(id) {
    var outgoing = getActiveTab();
    if (outgoing && outgoing.id === id) {
      renderTabBar();
      return;
    }
    if (outgoing) {
      saveTabState(outgoing);
    }
    activeTabId = id;
    var tab = getActiveTab();
    if (!tab) return;
    restoreTabState(tab);
    renderTabBar();
    updateWindowTitle();
  }

  function saveTabState(tab) {
    var editor = document.getElementById('editor');
    tab.content = editor.value;
    tab.cursorStart = editor.selectionStart;
    tab.cursorEnd = editor.selectionEnd;
    tab.mode = currentMode;
    if (currentMode === 'edit') {
      tab.scrollTop = editor.scrollTop;
    } else {
      tab.scrollTop = document.getElementById('preview-container').scrollTop;
    }
  }

  function restoreTabState(tab) {
    var editor = document.getElementById('editor');
    editor.value = tab.content;

    if (tab.mode !== currentMode) {
      toggleMode();
    }

    if (currentMode === 'edit') {
      editor.scrollTop = tab.scrollTop;
      editor.selectionStart = tab.cursorStart;
      editor.selectionEnd = tab.cursorEnd;
      editor.focus();
    } else {
      document.getElementById('preview').innerHTML = marked.parse(tab.content);
      setTimeout(function() {
        document.getElementById('preview-container').scrollTop = tab.scrollTop;
      }, 0);
    }

    document.getElementById('status-file').textContent = tab.filename;
  }

  function markDirty(id) {
    var tab = tabs.find(function(t) { return t.id === (id || activeTabId); });
    if (tab && !tab.dirty) {
      tab.dirty = true;
      renderTabBar();
      updateWindowTitle();
    }
  }

  function markClean(id) {
    var tab = tabs.find(function(t) { return t.id === (id || activeTabId); });
    if (tab) {
      tab.dirty = false;
      renderTabBar();
      updateWindowTitle();
    }
  }

  function updateWindowTitle() {
    var tab = getActiveTab();
    if (!tab) return;
    var title = 'mdview - ' + tab.filename;
    if (tab.dirty) title += ' *';
    sendToRust('set_title', { title: title });
    setTitle(tab.filename + (tab.dirty ? ' *' : ''));
  }

  function renderTabBar() {
    var bar = document.getElementById('tab-bar');
    var show = tabs.length > 1;
    bar.style.display = show ? '' : 'none';
    document.body.classList.toggle('has-tabs', show);
    bar.innerHTML = '';
    tabs.forEach(function(tab) {
      var el = document.createElement('div');
      el.className = 'tab' + (tab.id === activeTabId ? ' active' : '');

      var label = document.createElement('span');
      label.className = 'tab-label';
      label.textContent = tab.filename;
      el.appendChild(label);

      if (tab.dirty) {
        var dot = document.createElement('span');
        dot.className = 'tab-dirty';
        dot.textContent = '\u2022';
        el.appendChild(dot);
      }

      var close = document.createElement('span');
      close.className = 'tab-close';
      close.innerHTML = '&times;';
      close.addEventListener('click', function(e) {
        e.stopPropagation();
        closeTab(tab.id);
      });
      el.appendChild(close);

      el.addEventListener('click', function() {
        switchTab(tab.id);
      });
      el.addEventListener('mousedown', function(e) {
        if (e.button === 1) {
          e.preventDefault();
          closeTab(tab.id);
        }
      });
      bar.appendChild(el);
    });
  }

  function nextTab() {
    if (tabs.length < 2) return;
    var idx = tabs.findIndex(function(t) { return t.id === activeTabId; });
    switchTab(tabs[(idx + 1) % tabs.length].id);
  }

  function prevTab() {
    if (tabs.length < 2) return;
    var idx = tabs.findIndex(function(t) { return t.id === activeTabId; });
    switchTab(tabs[(idx - 1 + tabs.length) % tabs.length].id);
  }

  function findTabByPath(path) {
    if (!path) return null;
    var norm = normalizePath(path).toLowerCase();
    return tabs.find(function(t) { return t.path && t.path.toLowerCase() === norm; }) || null;
  }

  function getActiveTab() {
    return tabs.find(function(t) { return t.id === activeTabId; }) || null;
  }

  function hasAnyDirty() {
    return tabs.some(function(t) { return t.dirty; });
  }

  function updateTabPath(id, path) {
    var tab = tabs.find(function(t) { return t.id === (id || activeTabId); });
    if (tab) {
      tab.path = path ? normalizePath(path) : null;
      tab.filename = path ? path.split(/[/\\]/).pop() : 'Untitled';
      renderTabBar();
      updateWindowTitle();
      document.getElementById('status-file').textContent = tab.filename;
    }
  }

  return {
    createTab: createTab,
    closeTab: closeTab,
    switchTab: switchTab,
    markDirty: markDirty,
    markClean: markClean,
    nextTab: nextTab,
    prevTab: prevTab,
    findTabByPath: findTabByPath,
    getActiveTab: getActiveTab,
    hasAnyDirty: hasAnyDirty,
    updateTabPath: updateTabPath
  };
})();
