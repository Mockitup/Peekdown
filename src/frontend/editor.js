(function() {
  var editor = document.getElementById('editor');

  var changeTimer = null;
  editor.addEventListener('input', function() {
    var activeTab = TabManager.getActiveTab();
    if (activeTab) activeTab.parsedHtml = null;
    clearTimeout(changeTimer);
    changeTimer = setTimeout(function() {
      TabManager.markDirty();
      updateWordCount();
      if (typeof showRecentPanel === 'function') showRecentPanel();
      if (typeof tocOpen !== 'undefined' && tocOpen) updateTOC();
    }, 300);
    if (typeof splitMode !== 'undefined' && splitMode) {
      updateSplitPreview();
    }
  });

  // Tab key inserts spaces (but not Ctrl+Tab which switches tabs)
  editor.addEventListener('keydown', function(e) {
    if (e.key === 'Tab' && !e.ctrlKey) {
      e.preventDefault();
      var start = editor.selectionStart;
      var end = editor.selectionEnd;
      editor.value = editor.value.substring(0, start) + '    ' + editor.value.substring(end);
      editor.selectionStart = editor.selectionEnd = start + 4;
      editor.dispatchEvent(new Event('input'));
      TabManager.markDirty();
      if (typeof splitMode !== 'undefined' && splitMode) updateSplitPreview();
    }
  });
})();
