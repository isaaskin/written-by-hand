// Badge picker. Everything comes from window.WBH_BADGES (site/badges.js), which is
// generated next to the SVGs, so only badges that exist can be offered.
(function () {
  "use strict";
  var data = window.WBH_BADGES;
  var form = document.getElementById("picker");
  if (!data || !form) return;
  form.hidden = false;

  // Links are built from wherever this page is served, so there is no
  // hard-coded host to forget to update.
  var base = new URL("./", document.baseURI).href;
  var state = { style: data.styles[0].key, label: data.labels[0].slug };

  function badgeUrl(style, label) {
    return base + "badges/" + style + "/" + label + ".svg";
  }

  function labelText(slug) {
    for (var i = 0; i < data.labels.length; i++) {
      if (data.labels[i].slug === slug) return data.labels[i].text;
    }
    return "";
  }

  // Builds a radio button wrapped in its label: native keyboard and screen-reader
  // behaviour for free. Text goes in via textContent, never innerHTML.
  function radio(group, value, checked, className, fill) {
    var label = document.createElement("label");
    label.className = className;
    var input = document.createElement("input");
    input.type = "radio";
    input.name = group;
    input.value = value;
    input.checked = checked;
    var body = document.createElement("span");
    body.className = className + "-body";
    fill(body);
    label.appendChild(input);
    label.appendChild(body);
    return label;
  }

  var styles = document.getElementById("styles");
  data.styles.forEach(function (s, i) {
    styles.appendChild(radio("style", s.key, i === 0, "tile", function (body) {
      var img = document.createElement("img");
      img.dataset.style = s.key;
      img.alt = "";
      var name = document.createElement("span");
      name.className = "tile-name";
      name.textContent = s.name;
      var blurb = document.createElement("span");
      blurb.className = "tile-blurb";
      blurb.textContent = s.blurb;
      body.appendChild(img);
      body.appendChild(name);
      body.appendChild(blurb);
    }));
  });

  var labels = document.getElementById("labels");
  data.labels.forEach(function (l, i) {
    labels.appendChild(radio("label", l.slug, i === 0, "chip", function (body) {
      body.textContent = l.text;
    }));
  });

  var markdown = document.getElementById("markdown");
  var status = document.getElementById("status");

  function escapeMarkdown(text) {
    return text.replace(/[\\\[\]]/g, "\\$&");
  }

  function render() {
    var text = labelText(state.label);
    var url = badgeUrl(state.style, state.label);
    styles.querySelectorAll("img").forEach(function (img) {
      img.src = badgeUrl(img.dataset.style, state.label);
    });
    ["preview-light", "preview-dark"].forEach(function (id) {
      var img = document.getElementById(id);
      img.src = url;
      img.alt = text + " badge, " + state.style + " style";
    });
    markdown.value = "[![" + escapeMarkdown(text) + "](" + url + ")](" + base + ")";
    var download = document.getElementById("download");
    download.href = url;
    download.setAttribute("download", "written-by-hand-" + state.style + ".svg");
    status.textContent = "";
  }

  form.addEventListener("change", function (e) {
    if (e.target.name === "style") state.style = e.target.value;
    if (e.target.name === "label") state.label = e.target.value;
    render();
  });

  document.getElementById("copy").addEventListener("click", function () {
    function fallback() {
      markdown.focus();
      markdown.select();
      var ok = false;
      try { ok = document.execCommand("copy"); } catch (e) { ok = false; }
      status.textContent = ok ? "Copied!" : "Press Ctrl+C (or ⌘C) to copy the selected text.";
    }
    if (navigator.clipboard && window.isSecureContext) {
      navigator.clipboard.writeText(markdown.value).then(function () {
        status.textContent = "Copied! Paste it near the top of your README.";
      }, fallback);
    } else {
      fallback();
    }
  });

  if (location.protocol !== "https:") {
    document.getElementById("local-note").hidden = false;
  }

  // On GitHub Pages (owner.github.io/repo/) the source repository is implied by the URL.
  var host = location.hostname;
  var repo = location.pathname.split("/")[1];
  if (/\.github\.io$/.test(host) && repo) {
    var link = document.getElementById("repo-link");
    link.href = "https://github.com/" + host.split(".")[0] + "/" + repo;
    link.hidden = false;
  }

  render();
})();
