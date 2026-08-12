(function () {
  function boot(mermaid) {
    document.querySelectorAll("code.language-mermaid").forEach(function (el) {
      var wrap = document.createElement("div");
      wrap.className = "mermaid";
      wrap.textContent = el.textContent;
      el.parentElement.replaceWith(wrap);
    });
    var root = document.documentElement;
    var dark = ["coal", "navy", "ayu"].some(function (t) {
      return root.classList.contains(t);
    });
    mermaid.initialize({
      startOnLoad: false,
      theme: dark ? "dark" : "neutral",
      flowchart: { htmlLabels: false, curve: "basis" },
    });
    mermaid.run({ querySelector: ".mermaid" });
  }

  var s = document.createElement("script");
  s.src = "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js";
  s.onload = function () {
    boot(window.mermaid);
  };
  document.head.appendChild(s);
})();
