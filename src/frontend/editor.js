(function() {
  var editor = document.getElementById('editor');

  var changeTimer = null;
  editor.addEventListener('input', function() {
    clearTimeout(changeTimer);
    changeTimer = setTimeout(function() {
      sendToRust('content_changed');
    }, 300);
  });

  // Tab key inserts spaces
  editor.addEventListener('keydown', function(e) {
    if (e.key === 'Tab') {
      e.preventDefault();
      var start = editor.selectionStart;
      var end = editor.selectionEnd;
      editor.value = editor.value.substring(0, start) + '    ' + editor.value.substring(end);
      editor.selectionStart = editor.selectionEnd = start + 4;
      sendToRust('content_changed');
    }
  });
})();
