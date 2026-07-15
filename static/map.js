// Slipstick system map — canvas renderer for the solar system view.
// Pure drawing + coordinate helpers; all orbital mechanics happens server-side.
"use strict";

const SysMap = (() => {
  const BODY_COLORS = {
    sol: "#e8d05f", mercury: "#9c8f83", venus: "#e0c58f", earth: "#5fb4e8",
    luna: "#aab4be", mars: "#e86a4a", jupiter: "#d9a066", saturn: "#e8d0a0",
    uranus: "#8fd4d4", neptune: "#6a8fe8",
  };
  const FALLBACK = ["#c9a0f0", "#7ed491", "#f08fbe", "#6ae8d4", "#e8d05f"];

  function bodyColor(id, i) {
    return BODY_COLORS[id] || FALLBACK[i % FALLBACK.length];
  }

  // view: { cx, cy (world m at canvas center), mpp (metres per pixel) }
  const toPx = (view, w, h, x, y) =>
    [w / 2 + (x - view.cx) / view.mpp, h / 2 - (y - view.cy) / view.mpp];
  const toWorld = (view, w, h, px, py) =>
    [view.cx + (px - w / 2) * view.mpp, view.cy - (py - h / 2) * view.mpp];

  // state: { bodies: [{id,name,x,y,radius,orbit_a,parent_xy,sel}],
  //          ships: [{id,name,x,y,vx,vy,sel,landed,burning,path:[[x,y]..]}] }
  function draw(canvas, state, view) {
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth || 800;
    const h = canvas.clientHeight || 560;
    canvas.width = Math.round(w * dpr);
    canvas.height = Math.round(h * dpr);
    const ctx = canvas.getContext("2d");
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const css = getComputedStyle(document.documentElement);
    const cText = css.getPropertyValue("--plot-text").trim() || "#8fa3b8";
    const cGrid = css.getPropertyValue("--plot-grid").trim() || "#1c2530";
    ctx.fillStyle = css.getPropertyValue("--bg-inset").trim() || "#0a0f14";
    ctx.fillRect(0, 0, w, h);
    ctx.font = "11px 'IBM Plex Mono', ui-monospace, monospace";
    const layers = state.layers || {};

    // orbit rails (circles around each body's parent)
    ctx.strokeStyle = cGrid;
    ctx.lineWidth = 1;
    for (const b of layers.rails === false ? [] : state.bodies) {
      if (!b.orbit_a || !b.parent_xy) continue;
      const [px, py] = toPx(view, w, h, b.parent_xy[0], b.parent_xy[1]);
      const r = b.orbit_a / view.mpp;
      if (r < 4 || r > 40000) continue;
      ctx.beginPath();
      ctx.arc(px, py, r, 0, 2 * Math.PI);
      ctx.stroke();
    }

    // projected ship paths ([t, x, y] samples)
    const strokePath = (path, color, dash) => {
      if (!path || path.length < 2) return;
      ctx.strokeStyle = color;
      ctx.setLineDash(dash || []);
      ctx.beginPath();
      let started = false;
      for (const p of path) {
        const [px, py] = toPx(view, w, h, p[1], p[2]);
        if (!started) { ctx.moveTo(px, py); started = true; } else ctx.lineTo(px, py);
      }
      ctx.stroke();
      ctx.setLineDash([]);
    };
    if (layers.paths !== false) {
      for (const s of state.ships)
        strokePath(s.path, s.sel ? "#7ed491" : "rgba(126, 212, 145, 0.35)", [4, 4]);
      for (const ep of state.extraPaths || []) strokePath(ep.path, ep.color, ep.dash);
    }

    if (layers.weapons !== false) for (const wr of state.weaponRanges || []) {
      const [px, py] = toPx(view, w, h, wr.x, wr.y);
      const r = wr.radius / view.mpp;
      if (r > 2 && r < 40000) {
        ctx.strokeStyle = wr.color || "rgba(240,160,80,.5)"; ctx.setLineDash([3, 4]);
        ctx.beginPath(); ctx.arc(px, py, r, 0, Math.PI * 2); ctx.stroke(); ctx.setLineDash([]);
      }
    }

    // maneuver node: ring + thrust-direction arrow
    if (state.node) {
      const n = state.node;
      const [px, py] = toPx(view, w, h, n.x, n.y);
      ctx.strokeStyle = "#f0a050";
      ctx.lineWidth = 1.6;
      ctx.beginPath();
      ctx.arc(px, py, 7, 0, 2 * Math.PI);
      ctx.stroke();
      const th = (n.angle_deg || 0) * Math.PI / 180;
      const len = 26;
      const ax = px + Math.cos(th) * len, ay = py - Math.sin(th) * len;
      ctx.beginPath();
      ctx.moveTo(px + Math.cos(th) * 8, py - Math.sin(th) * 8);
      ctx.lineTo(ax, ay);
      ctx.stroke();
      ctx.fillStyle = "#f0a050";
      ctx.beginPath();
      ctx.arc(ax, ay, 3, 0, 2 * Math.PI);
      ctx.fill();
      if (n.label) {
        ctx.textAlign = "left";
        ctx.fillText(n.label, px + 10, py - 10);
      }
    }

    // bodies
    state.bodies.forEach((b, i) => {
      const [px, py] = toPx(view, w, h, b.x, b.y);
      if (px < -60 || px > w + 60 || py < -60 || py > h + 60) return;
      const col = bodyColor(b.id, i);
      const r = Math.max(b.id === "sol" ? 4 : 2.2, b.radius / view.mpp);
      ctx.fillStyle = col;
      ctx.beginPath();
      ctx.arc(px, py, Math.min(r, 400), 0, 2 * Math.PI);
      ctx.fill();
      if (b.sel || b.target) {
        ctx.strokeStyle = b.target ? "#f0a050" : "#e8d05f";
        ctx.beginPath();
        ctx.arc(px, py, Math.min(r, 400) + 5, 0, 2 * Math.PI);
        ctx.stroke();
      }
      if (layers.labels !== false) {
        ctx.fillStyle = cText; ctx.textAlign = "left";
        ctx.fillText(b.name + (b.target ? " ◎" : ""), px + Math.min(r, 12) + 4, py + 3);
      }
    });

    // ships: diamond marker + velocity arrow
    for (const s of state.ships) {
      const [px, py] = toPx(view, w, h, s.x, s.y);
      if (px < -40 || px > w + 40 || py < -40 || py > h + 40) continue;
      const col = s.target ? "#f0a050" : s.source ? "#7ed491" : s.sel ? "#e8d05f" : s.landed ? "#8fa3b8" : "#5fb4e8";
      ctx.strokeStyle = col;
      ctx.fillStyle = col;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(px, py - 6); ctx.lineTo(px + 5, py); ctx.lineTo(px, py + 6);
      ctx.lineTo(px - 5, py); ctx.closePath();
      if (s.sel) ctx.fill(); else ctx.stroke();
      // velocity arrow: where the ship drifts in the next hour, min 12 px cap 60
      const sp = Math.hypot(s.vx, s.vy);
      if (layers.vectors !== false && sp > 1 && !s.landed) {
        const len = Math.max(12, Math.min(60, sp * 3600 / view.mpp));
        const ux = s.vx / sp, uy = -s.vy / sp;
        ctx.beginPath();
        ctx.moveTo(px, py);
        ctx.lineTo(px + ux * len, py + uy * len);
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(px + ux * len, py + uy * len, 1.8, 0, 2 * Math.PI);
        ctx.fill();
      }
      if (layers.labels !== false) {
        ctx.fillStyle = cText; ctx.textAlign = "left";
        ctx.fillText(s.name + (s.source ? " ◉" : "") + (s.target ? " ◎" : "") +
                     (s.burning ? " ▶" : "") + (s.landed ? " ⏚" : ""), px + 9, py - 8);
      }
    }

    // scale bar: a nice round distance ~ a quarter of the canvas width
    const target = view.mpp * w * 0.25;
    const pow = Math.pow(10, Math.floor(Math.log10(target)));
    const nice = [1, 2, 5, 10].map(m => m * pow)
      .reduce((a, b) => Math.abs(b - target) < Math.abs(a - target) ? b : a);
    const barPx = nice / view.mpp;
    ctx.strokeStyle = cText;
    ctx.fillStyle = cText;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(14, h - 14); ctx.lineTo(14 + barPx, h - 14);
    ctx.moveTo(14, h - 18); ctx.lineTo(14, h - 10);
    ctx.moveTo(14 + barPx, h - 18); ctx.lineTo(14 + barPx, h - 10);
    ctx.stroke();
    ctx.textAlign = "left";
    ctx.fillText(fmtSI(nice, "m"), 20 + barPx, h - 10);
  }

  // Nearest object within `threshold` px. objects: [{kind, id, x, y}]
  function pick(canvas, view, ev, objects, threshold = 14) {
    const rect = canvas.getBoundingClientRect();
    const px = ev.clientX - rect.left, py = ev.clientY - rect.top;
    let best = null, bd = threshold;
    for (const o of objects) {
      const [ox, oy] = toPx(view, rect.width, rect.height, o.x, o.y);
      const d = Math.hypot(ox - px, oy - py);
      if (d < bd) { bd = d; best = o; }
    }
    return best;
  }

  function eventWorld(canvas, view, ev) {
    const rect = canvas.getBoundingClientRect();
    return toWorld(view, rect.width, rect.height,
                   ev.clientX - rect.left, ev.clientY - rect.top);
  }

  // Nearest point on a [t, x, y] polyline within `threshold` px of the event.
  function pathHit(canvas, view, ev, path, threshold = 12) {
    if (!path || path.length < 2) return null;
    const rect = canvas.getBoundingClientRect();
    const px = ev.clientX - rect.left, py = ev.clientY - rect.top;
    let best = null, bd = threshold;
    for (let i = 0; i < path.length - 1; i++) {
      const [ax, ay] = toPx(view, rect.width, rect.height, path[i][1], path[i][2]);
      const [bx, by] = toPx(view, rect.width, rect.height, path[i + 1][1], path[i + 1][2]);
      const dx = bx - ax, dy = by - ay;
      const den = dx * dx + dy * dy;
      const f = den > 0 ? Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / den)) : 0;
      const d = Math.hypot(px - (ax + f * dx), py - (ay + f * dy));
      if (d < bd) { bd = d; best = { i, f }; }
    }
    if (!best) return null;
    const a = path[best.i], b = path[best.i + 1], f = best.f;
    return { index: f < 0.5 ? best.i : best.i + 1,
      t: a[0] + f * (b[0] - a[0]), x: a[1] + f * (b[1] - a[1]),
      y: a[2] + f * (b[2] - a[2]) };
  }

  return { draw, pick, eventWorld, pathHit, bodyColor };
})();
