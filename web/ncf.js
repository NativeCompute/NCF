(() => {
  "use strict";

  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
  const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const el = (tag, attrs = {}, ...kids) => {
    const n = document.createElement(tag);
    Object.entries(attrs).forEach(([k, v]) => (k === "class" ? (n.className = v) : n.setAttribute(k, v)));
    kids.flat().forEach((c) => n.append(c));
    return n;
  };

  // header here---
  const header = () => {
    const h = $("[data-header]");
    if (!h) return;
    const onScroll = () => h.classList.toggle("is-scrolled", window.scrollY > 4);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    const menuBtn = $("[data-menu]", h);
    const mobileNav = $("#mobile-nav");
    if (!menuBtn || !mobileNav) return;
    const setMenu = (open) => {
      mobileNav.hidden = !open;
      h.dataset.open = String(open);
      menuBtn.setAttribute("aria-expanded", String(open));
      document.body.style.overflow = open ? "hidden" : "";
    };
    menuBtn.addEventListener("click", () => setMenu(mobileNav.hidden));
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && !mobileNav.hidden) { setMenu(false); menuBtn.focus(); }
    });
  };

  // toast here---
  let toasts = null;
  let toastTimer = 0;
  const toast = (text, action) => {
    if (!toasts) { toasts = el("div", { class: "toasts", role: "status", "aria-live": "polite" }); document.body.append(toasts); }
    clearTimeout(toastTimer);
    toasts.replaceChildren();
    const t = el("div", { class: "toast" }, el("span", {}, text));
    const hide = () => { t.classList.remove("is-in"); setTimeout(() => t.remove(), 150); };
    if (action) {
      const b = el("button", { class: "btn btn--sm", type: "button" }, action.label);
      b.addEventListener("click", () => { hide(); action.run(b); });
      t.append(b);
    }
    toasts.append(t);
    requestAnimationFrame(() => t.classList.add("is-in"));
    let paused = false;
    t.addEventListener("pointerenter", () => (paused = true));
    t.addEventListener("pointerleave", () => { paused = false; toastTimer = setTimeout(hide, 3000); });
    t.addEventListener("focusin", () => (paused = true));
    toastTimer = setTimeout(() => { if (!paused) hide(); }, 6000);
  };

  // copy here---
  const copyButtons = () => $$("[data-copy]").forEach((b) => b.addEventListener("click", async () => {
    const source = $(b.dataset.copy);
    try {
      await navigator.clipboard.writeText(source ? source.textContent : b.dataset.copyText || "");
      toast(b.dataset.copied || "Copied to your clipboard.");
    } catch {
      toast("Copying isn't available here. Select the text and copy it instead.");
    }
  }));

  // tray here---
  const tray = (root) => {
    const panel = $(".tray", root);
    let opener = null;
    const open = () => {
      opener = document.activeElement;
      root.classList.add("is-open");
      root.setAttribute("aria-hidden", "false");
      document.body.style.overflow = "hidden";
      requestAnimationFrame(() => ($("[autofocus], button, a, input", panel) || panel).focus());
    };
    const close = () => {
      root.classList.remove("is-open");
      root.setAttribute("aria-hidden", "true");
      document.body.style.overflow = "";
      opener?.focus?.();
    };
    $$("[data-tray-close], .tray-scrim", root).forEach((b) => b.addEventListener("click", close));
    root.addEventListener("keydown", (e) => {
      if (e.key === "Escape") close();
      if (e.key !== "Tab") return;
      const focusable = $$("a[href], button:not([disabled]), input, select, textarea, [tabindex]:not([tabindex='-1'])", panel);
      if (!focusable.length) return;
      const [first, last] = [focusable[0], focusable[focusable.length - 1]];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    });
    return { open, close };
  };
  const trays = () => $$("[data-tray]").forEach((root) => {
    const t = tray(root);
    root.ncf = t;
    $$(`[data-tray-open="${root.id}"]`).forEach((b) => b.addEventListener("click", t.open));
  });

  // confirm here---
  const confirm = ({ title, body, yes = "OK", no = "Cancel", danger = false }) => new Promise((resolve) => {
    const d = el("dialog", { class: "confirm", "aria-labelledby": "ncf-confirm-title" },
      el("div", { class: "confirm-body" }, el("h2", { class: "h3", id: "ncf-confirm-title" }, title), el("p", { class: "sec" }, body || "")),
      el("div", { class: "confirm-foot" },
        el("button", { class: "btn btn--ghost btn--sm", type: "button", value: "no" }, no),
        el("button", { class: `btn ${danger ? "btn--danger" : "btn--primary"} btn--sm`, type: "button", value: "yes" }, yes)));
    document.body.append(d);
    d.addEventListener("click", (e) => { const v = e.target.closest("button")?.value; if (v) d.close(v); });
    d.addEventListener("close", () => { resolve(d.returnValue === "yes"); d.remove(); });
    d.showModal();
  });

  // fit here---
  const fitAll = () => $$("[data-fit]").forEach((n) => {
    const avail = n.parentElement.clientWidth;
    if (!avail) return;
    n.style.fontSize = "100px";
    const w = n.getBoundingClientRect().width;
    if (w) n.style.fontSize = (100 * avail * 0.985 / w).toFixed(2) + "px";
  });
  const fit = () => {
    fitAll();
    document.fonts?.ready.then(fitAll);
    document.fonts?.addEventListener?.("loadingdone", fitAll);
    let t = 0;
    window.addEventListener("resize", () => { clearTimeout(t); t = setTimeout(fitAll, 100); });
  };

  // clock here---
  const clocks = () => {
    const list = $$("[data-clock]");
    if (!list.length) return;
    const tick = () => {
      const now = new Date();
      const text = now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false });
      list.forEach((c) => { c.textContent = text; c.dateTime = now.toISOString(); });
    };
    tick();
    setInterval(tick, 1000);
  };

  // tabs here---
  const tabs = () => $$("[role=tablist]").forEach((list) => {
    const all = $$("[role=tab]", list);
    const select = (tab, focus = true) => {
      all.forEach((t) => {
        const on = t === tab;
        t.setAttribute("aria-selected", String(on));
        t.tabIndex = on ? 0 : -1;
        const panel = document.getElementById(t.getAttribute("aria-controls"));
        if (panel) panel.hidden = !on;
      });
      if (focus) tab.focus();
      if (tab.dataset.key && list.dataset.hash !== "off") history.replaceState(null, "", "#" + tab.dataset.key);
      list.dispatchEvent(new CustomEvent("ncf:tab", { detail: { key: tab.dataset.key } }));
    };
    all.forEach((t, i) => {
      t.addEventListener("click", () => select(t));
      t.addEventListener("keydown", (e) => {
        const d = { ArrowRight: 1, ArrowLeft: -1 }[e.key];
        if (d) { e.preventDefault(); select(all[(i + d + all.length) % all.length]); }
        if (e.key === "Home") { e.preventDefault(); select(all[0]); }
        if (e.key === "End") { e.preventDefault(); select(all[all.length - 1]); }
      });
    });
    const fromHash = () => {
      const t = all.find((x) => "#" + x.dataset.key === location.hash);
      if (t) select(t, false);
    };
    fromHash();
    window.addEventListener("hashchange", fromHash);
  });

  // ranges here---
  const ranges = () => $$("input[type=range]").forEach((r) => {
    const paint = () => {
      const min = Number(r.min || 0), max = Number(r.max || 100);
      r.style.setProperty("--fill", ((Number(r.value) - min) / (max - min) * 100).toFixed(1) + "%");
    };
    r.addEventListener("input", paint);
    paint();
  });

  // theme here---
  const setTheme = (mode) => {
    if (mode === "light") document.documentElement.dataset.theme = "light";
    else delete document.documentElement.dataset.theme;
    try { localStorage.setItem("ncf:theme", mode); } catch { }
  };
  const themeToggles = () => {
    try { const saved = localStorage.getItem("ncf:theme"); if (saved) setTheme(saved); } catch { }
    $$("[data-theme-toggle]").forEach((b) => b.addEventListener("click", () => {
      setTheme(document.documentElement.dataset.theme === "light" ? "dark" : "light");
    }));
  };

  const start = () => {
    document.documentElement.classList.remove("no-js");
    document.documentElement.classList.add("js");
    header();
    copyButtons();
    trays();
    fit();
    clocks();
    tabs();
    ranges();
    themeToggles();
  };

  const trayApi = {
    open: (id) => document.getElementById(id)?.ncf?.open(),
    close: (id) => document.getElementById(id)?.ncf?.close(),
  };
  window.NCF = { toast, confirm, tray: trayApi, setTheme, fit: fitAll, reduceMotion };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start);
  else start();
})();
