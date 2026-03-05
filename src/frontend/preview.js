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
