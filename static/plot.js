// Slipstick plot module — hand-rolled canvas line plots.
// Line series, log/linear axes, cutoff markers, optional second y-axis.
// No dependencies; works offline.
"use strict";

const Plot = (() => {
  const PALETTE = ["#5fb4e8", "#f0a050", "#7ed491", "#e86a6a", "#c9a0f0",
                   "#e8d05f", "#6ae8d4", "#f08fbe"];

  function niceTicks(lo, hi, n) {
    if (!(hi > lo)) hi = lo + 1;
    const span = hi - lo;
    const step0 = Math.pow(10, Math.floor(Math.log10(span / n)));
    let step = step0;
    for (const m of [1, 2, 5, 10]) {
      if (span / (step0 * m) <= n) { step = step0 * m; break; }
    }
    const ticks = [];
    for (let v = Math.ceil(lo / step) * step; v <= hi + step * 1e-9; v += step)
      ticks.push(Math.abs(v) < step * 1e-9 ? 0 : v);
    return ticks;
  }

  function logTicks(lo, hi) {
    const ticks = [];
    const d0 = Math.floor(Math.log10(lo)), d1 = Math.ceil(Math.log10(hi));
    for (let d = d0; d <= d1; d++) {
      const v = Math.pow(10, d);
      if (v >= lo * 0.999 && v <= hi * 1.001) ticks.push(v);
    }
    // If too few decades, add 2 and 5 subticks.
    if (ticks.length < 3) {
      for (let d = d0 - 1; d <= d1; d++)
        for (const m of [2, 5]) {
          const v = m * Math.pow(10, d);
          if (v >= lo * 0.999 && v <= hi * 1.001) ticks.push(v);
        }
      ticks.sort((a, b) => a - b);
    }
    return ticks;
  }

  function fmtTick(v) {
    if (v === 0) return "0";
    const a = Math.abs(v);
    if (a >= 1e6 || a < 1e-3) {
      const e = Math.floor(Math.log10(a));
      const m = v / Math.pow(10, e);
      const ms = Math.abs(m - 1) < 1e-6 ? "" : (Math.round(m * 100) / 100) + "×";
      return ms + "10" + sup(e);
    }
    if (a >= 1000) return (v / 1000) + "k";
    return String(Math.round(v * 1000) / 1000);
  }

  function sup(n) {
    const map = { "-": "⁻", "0": "⁰", "1": "¹", "2": "²", "3": "³", "4": "⁴",
                  "5": "⁵", "6": "⁶", "7": "⁷", "8": "⁸", "9": "⁹" };
    return String(n).split("").map(c => map[c] || c).join("");
  }

  // opts: { series: [{x, y, label, color, axis:'y'|'y2', dash}],
  //         xlabel, ylabel, y2label, xlog, ylog, hlines:[{y,label,color,axis}],
  //         vlines:[{x,label,color}], height }
  function draw(canvas, opts) {
    const dpr = window.devicePixelRatio || 1;
    const cssW = canvas.clientWidth || canvas.parentElement.clientWidth || 600;
    const cssH = opts.height || 260;
    canvas.style.height = cssH + "px";
    canvas.width = Math.round(cssW * dpr);
    canvas.height = Math.round(cssH * dpr);
    const ctx = canvas.getContext("2d");
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, cssW, cssH);

    const series = (opts.series || []).filter(s => s.x && s.x.length > 1);
    const hasY2 = series.some(s => s.axis === "y2") ||
                  (opts.hlines || []).some(h => h.axis === "y2");
    const pad = { l: 58, r: hasY2 ? 58 : 14, t: 10, b: 34 };
    const W = cssW - pad.l - pad.r, H = cssH - pad.t - pad.b;
    if (W < 40 || H < 40) return;

    // ---- ranges
    const finiteX = v => Number.isFinite(v) && (!opts.xlog || v > 0);
    let xmin = Infinity, xmax = -Infinity;
    for (const s of series) for (const v of s.x) if (finiteX(v)) {
      if (v < xmin) xmin = v; if (v > xmax) xmax = v;
    }
    for (const vl of opts.vlines || []) {
      if (vl.x < xmin) xmin = vl.x; if (vl.x > xmax) xmax = vl.x;
    }
    if (!Number.isFinite(xmin)) { xmin = 0; xmax = 1; }
    if (xmin === xmax) xmax = xmin + 1;

    function yRange(axisName) {
      let lo = Infinity, hi = -Infinity;
      for (const s of series) {
        if ((s.axis === "y2") !== (axisName === "y2")) continue;
        for (const v of s.y) {
          if (!Number.isFinite(v)) continue;
          if (opts.ylog && v <= 0) continue;
          if (v < lo) lo = v; if (v > hi) hi = v;
        }
      }
      for (const h of opts.hlines || []) {
        if (((h.axis || "y") === "y2") !== (axisName === "y2")) continue;
        if (h.y < lo) lo = h.y; if (h.y > hi) hi = h.y;
      }
      if (!Number.isFinite(lo)) { lo = 0; hi = 1; }
      if (lo === hi) { hi = lo + 1; }
      if (!opts.ylog) {
        const m = (hi - lo) * 0.06;
        if (lo > 0 && lo - m < 0 && lo < (hi - lo)) lo = 0; else lo -= m;
        hi += m;
      } else { lo *= 0.7; hi *= 1.4; }
      return [lo, hi];
    }
    const [ymin, ymax] = yRange("y");
    const [y2min, y2max] = hasY2 ? yRange("y2") : [0, 1];

    const xmap = v => {
      if (opts.xlog) return pad.l + W * (Math.log(v) - Math.log(xmin)) /
                             (Math.log(xmax) - Math.log(xmin));
      return pad.l + W * (v - xmin) / (xmax - xmin);
    };
    // Inverse x-mapping stashed on the canvas so click handlers can read
    // "what range did I click" back out of a rendered plot.
    canvas._plotX = { xlog: !!opts.xlog, xmin, xmax, padl: pad.l, w: W };
    const ymapFor = (lo, hi) => v => {
      if (opts.ylog) return pad.t + H * (1 - (Math.log(v) - Math.log(lo)) /
                                              (Math.log(hi) - Math.log(lo)));
      return pad.t + H * (1 - (v - lo) / (hi - lo));
    };
    const ymap = ymapFor(ymin, ymax);
    const y2map = ymapFor(y2min, y2max);

    const css = getComputedStyle(document.documentElement);
    const cGrid = css.getPropertyValue("--plot-grid").trim() || "#26313d";
    const cText = css.getPropertyValue("--plot-text").trim() || "#8fa3b8";
    const cAxis = css.getPropertyValue("--plot-axis").trim() || "#3a4a5c";

    ctx.font = "11px 'IBM Plex Mono', ui-monospace, monospace";

    // ---- grid + ticks
    const xticks = opts.xlog ? logTicks(xmin, xmax) : niceTicks(xmin, xmax, 7);
    const yticks = opts.ylog ? logTicks(ymin, ymax) : niceTicks(ymin, ymax, 5);
    ctx.strokeStyle = cGrid; ctx.fillStyle = cText; ctx.lineWidth = 1;
    for (const t of xticks) {
      const px = xmap(t);
      if (px < pad.l - 1 || px > pad.l + W + 1) continue;
      ctx.beginPath(); ctx.moveTo(px, pad.t); ctx.lineTo(px, pad.t + H); ctx.stroke();
      ctx.textAlign = "center";
      ctx.fillText(fmtTick(t), px, pad.t + H + 15);
    }
    for (const t of yticks) {
      const py = ymap(t);
      if (py < pad.t - 1 || py > pad.t + H + 1) continue;
      ctx.beginPath(); ctx.moveTo(pad.l, py); ctx.lineTo(pad.l + W, py); ctx.stroke();
      ctx.textAlign = "right";
      ctx.fillText(fmtTick(t), pad.l - 6, py + 3);
    }
    if (hasY2) {
      const y2ticks = niceTicks(y2min, y2max, 5);
      ctx.textAlign = "left";
      for (const t of y2ticks) {
        const py = y2map(t);
        if (py < pad.t - 1 || py > pad.t + H + 1) continue;
        ctx.fillText(fmtTick(t), pad.l + W + 6, py + 3);
      }
    }

    // ---- axes frame
    ctx.strokeStyle = cAxis;
    ctx.strokeRect(pad.l, pad.t, W, H);

    // ---- axis labels
    ctx.fillStyle = cText; ctx.textAlign = "center";
    if (opts.xlabel) ctx.fillText(opts.xlabel, pad.l + W / 2, cssH - 4);
    if (opts.ylabel) {
      ctx.save(); ctx.translate(11, pad.t + H / 2); ctx.rotate(-Math.PI / 2);
      ctx.fillText(opts.ylabel, 0, 0); ctx.restore();
    }
    if (hasY2 && opts.y2label) {
      ctx.save(); ctx.translate(cssW - 5, pad.t + H / 2); ctx.rotate(Math.PI / 2);
      ctx.fillText(opts.y2label, 0, 0); ctx.restore();
    }

    // ---- cutoff markers
    ctx.save();
    ctx.setLineDash([5, 4]);
    for (const h of opts.hlines || []) {
      const m = (h.axis === "y2") ? y2map : ymap;
      const py = m(h.y);
      if (py < pad.t || py > pad.t + H) continue;
      ctx.strokeStyle = h.color || "#e86a6a";
      ctx.beginPath(); ctx.moveTo(pad.l, py); ctx.lineTo(pad.l + W, py); ctx.stroke();
      if (h.label) {
        ctx.fillStyle = h.color || "#e86a6a"; ctx.textAlign = "left";
        ctx.fillText(h.label, pad.l + 5, py - 4);
      }
    }
    for (const v of opts.vlines || []) {
      const px = xmap(v.x);
      if (px < pad.l || px > pad.l + W) continue;
      ctx.strokeStyle = v.color || "#e8d05f";
      ctx.beginPath(); ctx.moveTo(px, pad.t); ctx.lineTo(px, pad.t + H); ctx.stroke();
      if (v.label) {
        ctx.fillStyle = v.color || "#e8d05f"; ctx.textAlign = "left";
        ctx.save(); ctx.translate(px + 4, pad.t + 12); ctx.fillText(v.label, 0, 0); ctx.restore();
      }
    }
    ctx.restore();

    // ---- series
    ctx.save();
    ctx.beginPath(); ctx.rect(pad.l, pad.t, W, H); ctx.clip();
    series.forEach((s, si) => {
      const col = s.color || PALETTE[si % PALETTE.length];
      const m = (s.axis === "y2") ? y2map : ymap;
      ctx.strokeStyle = col; ctx.lineWidth = 1.6;
      ctx.setLineDash(s.dash ? [6, 4] : []);
      ctx.beginPath();
      let started = false;
      for (let i = 0; i < s.x.length; i++) {
        const xv = s.x[i], yv = s.y[i];
        if (!Number.isFinite(xv) || !Number.isFinite(yv) ||
            (opts.xlog && xv <= 0) || (opts.ylog && yv <= 0)) { started = false; continue; }
        const px = xmap(xv), py = m(yv);
        if (!started) { ctx.moveTo(px, py); started = true; } else ctx.lineTo(px, py);
      }
      ctx.stroke();
    });
    ctx.restore();

    // ---- legend
    const labeled = series.filter(s => s.label);
    if (labeled.length) {
      ctx.textAlign = "left"; ctx.font = "11px 'IBM Plex Mono', ui-monospace, monospace";
      let ly = pad.t + 14;
      const lx = pad.l + W - 8 -
        Math.max(...labeled.map(s => ctx.measureText(s.label).width)) - 18;
      for (let si = 0; si < series.length; si++) {
        const s = series[si];
        if (!s.label) continue;
        const col = s.color || PALETTE[si % PALETTE.length];
        ctx.strokeStyle = col; ctx.lineWidth = 2;
        ctx.setLineDash(s.dash ? [6, 4] : []);
        ctx.beginPath(); ctx.moveTo(lx, ly - 3); ctx.lineTo(lx + 14, ly - 3); ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillStyle = cText;
        ctx.fillText(s.label, lx + 18, ly);
        ly += 15;
      }
    }
  }

  // Map a click event on a drawn plot back to an x-axis value (null if the
  // click fell outside the plot area or nothing has been drawn yet).
  function xFromEvent(canvas, ev) {
    const m = canvas._plotX;
    if (!m) return null;
    const rect = canvas.getBoundingClientRect();
    const px = ev.clientX - rect.left;
    const f = (px - m.padl) / m.w;
    if (f < 0 || f > 1) return null;
    if (m.xlog) return Math.exp(Math.log(m.xmin) + f * (Math.log(m.xmax) - Math.log(m.xmin)));
    return m.xmin + f * (m.xmax - m.xmin);
  }

  return { draw, xFromEvent, PALETTE };
})();
