// Slipstick frontend. Owns schema and presentation; most physical numbers come
// from /api/calc/*. Straight sums, unit formatting, option lookups, and the
// synchronous radiator mass derivation happen here.
"use strict";

/* ============================== API ==================================== */

async function calc(name, body) {
  const res = await fetch("/api/calc/" + name, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    let msg = res.statusText;
    try { msg = (await res.json()).error || msg; } catch (e) {}
    throw new Error(msg);
  }
  return res.json();
}

let DB = null;
let saveTimer = null;
let saveRevision = 0;
let savedRevision = 0;
let saveInFlight = false;
let lastSaveError = null;

function invalidNumberPath(value, path = "fleet", seen = new WeakSet()) {
  if (typeof value === "number") return Number.isFinite(value) ? null : path;
  if (!value || typeof value !== "object") return null;
  if (seen.has(value)) return path + " (circular reference)";
  seen.add(value);
  for (const [key, child] of Object.entries(value)) {
    const childPath = Array.isArray(value) ? `${path}[${key}]` : `${path}.${key}`;
    const invalid = invalidNumberPath(child, childPath, seen);
    if (invalid) return invalid;
  }
  seen.delete(value);
  return null;
}

function fleetSnapshot() {
  const invalid = invalidNumberPath(DB);
  if (invalid) throw new Error(`${invalid}: enter a valid number`);
  return JSON.stringify(DB);
}

async function saveResponseError(res) {
  const fallback = `HTTP ${res.status}${res.statusText ? " " + res.statusText : ""}`;
  try {
    const payload = await res.json();
    return payload?.error ? `${payload.error} (${fallback})` : fallback;
  } catch (e) {
    return fallback;
  }
}

function setSaveStatus(state, detail = "") {
  const el = $("save-status");
  if (!el) return;
  const labels = { loading: "Loading…", dirty: "Unsaved", saving: "Saving…",
                   saved: "Saved", failed: "Save failed — Retry" };
  el.textContent = labels[state] || state;
  el.className = "save-status " + state;
  el.title = detail || "Fleet data save status";
  el.disabled = state !== "failed";
}

async function flushSave() {
  clearTimeout(saveTimer);
  if (saveInFlight || !DB || savedRevision >= saveRevision) return;
  const revision = saveRevision;
  const previousError = lastSaveError?.message;
  saveInFlight = true;
  setSaveStatus("saving");
  try {
    const snapshot = fleetSnapshot();
    const res = await fetch("/api/data", {
      method: "PUT", headers: { "Content-Type": "application/json" }, body: snapshot,
    });
    if (!res.ok) throw new Error(await saveResponseError(res));
    savedRevision = revision;
    lastSaveError = null;
  } catch (e) {
    lastSaveError = e;
  } finally {
    saveInFlight = false;
  }
  if (lastSaveError) {
    setSaveStatus("failed", lastSaveError.message);
    if (lastSaveError.message !== previousError) toast("Save failed: " + lastSaveError.message);
  }
  else if (savedRevision < saveRevision) {
    setSaveStatus("dirty");
    saveTimer = setTimeout(flushSave, 0);
  } else setSaveStatus("saved");
}

function touch(rerender = true) {
  clearTimeout(saveTimer);
  saveRevision++;
  lastSaveError = null;
  setSaveStatus("dirty");
  saveTimer = setTimeout(flushSave, 400);
  if (rerender) render();
}

/* ============================ helpers =================================== */

const $ = id => document.getElementById(id);
const S = () => DB.settings;
const uid = () => Math.random().toString(36).slice(2, 10);
const esc = s => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;")
                          .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const num = id => parseFloat($(id).value);

// Nearest engineering prefix, globally.
function fmtSI(v, unit) {
  if (!Number.isFinite(v)) return "—";
  const neg = v < 0 ? "−" : "";
  v = Math.abs(v);
  const steps = [[1e18, "E"], [1e15, "P"], [1e12, "T"], [1e9, "G"], [1e6, "M"], [1e3, "k"]];
  for (const [f, p] of steps)
    if (v >= f) return neg + trim3(v / f) + " " + p + unit;
  if (v >= 1 || v === 0) return neg + trim3(v) + " " + unit;
  const small = [[1e-3, "m"], [1e-6, "µ"], [1e-9, "n"]];
  for (const [f, p] of small)
    if (v >= f) return neg + trim3(v / f) + " " + p + unit;
  return neg + v.toExponential(2) + " " + unit;
}
function trim3(v) {
  if (v >= 100) return v.toFixed(0);
  if (v >= 10) return v.toFixed(1);
  return v.toFixed(2);
}
const fmtDist = m => fmtSI(m, "m");
const fmtVel = ms => fmtSI(ms, "m/s");
function fmtT(t) { // tonnes
  if (!Number.isFinite(t)) return "—";
  if (Math.abs(t) >= 10000) return (t / 1000).toLocaleString(undefined, { maximumFractionDigits: 1 }) + " kt";
  if (Math.abs(t) >= 10) return t.toLocaleString(undefined, { maximumFractionDigits: 0 }) + " t";
  if (Math.abs(t) >= 0.01) return t.toLocaleString(undefined, { maximumFractionDigits: 2 }) + " t";
  return (t * 1000).toFixed(1) + " kg";
}
function fmtDur(s) {
  if (!Number.isFinite(s)) return "indefinite";
  if (s < 0) return "—";
  if (s < 120) return s.toFixed(s < 10 ? 2 : 0) + " s";
  if (s < 7200) return (s / 60).toFixed(1) + " min";
  if (s < 172800) return (s / 3600).toFixed(1) + " h";
  return (s / 86400).toFixed(1) + " d";
}
function fmtMg(a) { // m/s^2 -> milligee
  return (a / (S().g * 1e-3)).toLocaleString(undefined, { maximumFractionDigits: 2 }) + " mg";
}
const fmtMJ = mj => fmtSI(mj * 1e6, "J");
const DIST_UNITS = { Gm: 1e9, Mm: 1e6, AU: 1.495978707e11, km: 1e3 };
function fmtMwKg(v) {
  if (!Number.isFinite(v)) return "—";
  return Math.abs(v) >= 0.01 ? trim3(v) : v.toPrecision(3);
}

function radiatorFlux_w_m2(t_k, eps) {
  const sigma = S().sigma;
  if (![t_k, eps, sigma].every(Number.isFinite) || t_k <= 0 || eps <= 0 || sigma <= 0) return 0;
  return eps * sigma * Math.pow(t_k, 4);
}

function radiatorPower_w(c) {
  const area = c.area_m2 || 0;
  if (!Number.isFinite(area) || area <= 0) return 0;
  if (Number.isFinite(c.mw_per_m2) && c.mw_per_m2 > 0) return area * c.mw_per_m2 * 1e6;
  return radiatorFlux_w_m2(c.t_k || 0, c.eps || 0) * area;
}

function specificPowerFromAreaDensityMwKg(t_k, eps, kg_m2) {
  if (!Number.isFinite(kg_m2) || kg_m2 <= 0) return 0;
  return radiatorFlux_w_m2(t_k, eps) / (kg_m2 * 1e6);
}

function radiatorDefaultMwKg(kind) {
  const st = S();
  const hot = kind === "radiator_hot";
  const key = hot ? "as_hot_mw_per_kg" : "as_low_mw_per_kg";
  if (Number.isFinite(st[key]) && st[key] > 0) return st[key];
  const legacyKey = hot ? "as_hot_kg_m2" : "as_low_kg_m2";
  const t = hot ? st.as_hot_t_k : st.as_low_t_k;
  const eps = hot ? st.as_hot_eps : st.as_low_eps;
  return specificPowerFromAreaDensityMwKg(t, eps, st[legacyKey]) || (hot ? 0.326592 : 0.0015946875);
}

function radiatorMwKg(c) {
  if (Number.isFinite(c.mw_per_kg) && c.mw_per_kg > 0) return c.mw_per_kg;
  const q = radiatorPower_w(c);
  if (q > 0 && Number.isFinite(c.mass_t) && c.mass_t > 0) return q / (c.mass_t * 1e9);
  return radiatorDefaultMwKg(c.kind);
}

function componentMass_t(c) {
  const each = Math.max(0, c.mass_t || 0);
  return each * (c.kind === "laser" ? laserCount(c) : 1);
}

function toast(msg, ok = false) {
  const root = $("toast-root");
  root.innerHTML = `<div class="toast ${ok ? "ok" : ""}">${esc(msg)}</div>`;
  clearTimeout(toast._t);
  toast._t = setTimeout(() => { root.innerHTML = ""; }, 5000);
}

// Guard for async panel updates: only the latest call may write the DOM.
function makeLatest() {
  let seq = 0;
  return async fn => {
    const my = ++seq;
    try { await fn(() => my === seq); }
    catch (e) { if (my === seq) toast(e.message); }
  };
}

/* ---- choice fields: a select of sane options with a custom escape hatch */

function choiceField(id, value, opts) {
  const match = opts.find(o => Math.abs(o.v - value) <= Math.abs(o.v) * 1e-9);
  const sel = `<select id="${id}-sel">` + opts.map(o =>
    `<option value="${o.v}" ${match && o.v === match.v ? "selected" : ""}>${esc(o.label)}</option>`).join("") +
    `<option value="__custom" ${match ? "" : "selected"}>custom…</option></select>`;
  const inp = `<input type="number" step="any" id="${id}" value="${value}"
    style="display:${match ? "none" : "inline-block"};width:90px">`;
  return sel + " " + inp;
}
function bindChoice(id, onchange) {
  const sel = $(id + "-sel"), inp = $(id);
  sel.onchange = () => {
    if (sel.value === "__custom") inp.style.display = "inline-block";
    else { inp.style.display = "none"; inp.value = sel.value; if (onchange) onchange(); }
  };
  if (onchange) inp.onchange = onchange;
}

/* ========================== design sums ================================= */

const designById = id => DB.designs.find(d => d.id === id);
const shipById = id => DB.states.find(s => s.id === id);
const missileById = id => DB.missiles.find(m => m.id === id);

const PROPULSION = {
  mh: "Metallic hydrogen",
  am: "Antimatter thermal",
  fusion: "Fusion bus",
};
// Exhaust velocity by propulsion choice; defaults live in settings.
function stageVe(s) {
  if (s.propulsion === "am") return (s.isp_s || S().prop_am_isp_s) * S().g;
  if (s.propulsion === "fusion") return (s.isp_s || S().prop_fusion_isp_s) * S().g;
  if (s.propulsion === "mh" || !s.ve_m_s) return S().prop_mh_ve_m_s;
  return s.ve_m_s;
}
const missileVe = m => stageVe(m.stages?.[0] || m); // compatibility/readout helper
const missileWetKg = m => (m.payload_kg || 0) + (m.stages || []).reduce(
  (n, s) => n + (s.dry_mass_kg || 0) + (s.propellant_kg || 0), 0);
const missileDryKg = m => (m.payload_kg || 0) + (m.stages || []).reduce(
  (n, s) => n + (s.dry_mass_kg || 0), 0);
const missilePayload = m => ({
  payload_kg: m.payload_kg || 0,
  stages: (m.stages || []).map(s => ({
    id: s.id, name: s.name || s.id, dry_mass_kg: s.dry_mass_kg || 0,
    propellant_kg: s.propellant_kg || 0, ve: stageVe(s), a0_g: s.a0_g,
    jettison: !!s.jettison,
  })),
  g: S().g,
});
const laserCount = c => Math.max(1, Math.round(c.count || 1));

function sums(design) {
  const st = S();
  const out = {
    compMass_t: 0, ordnance_t: 0, tankCap_t: 0,
    p_fusion: 0, rad_load_frac: 0, f_cap: 0,
    sinkCap_mj: 0, flyCap_mj: 0,
    radHot: [], radLow: [], lasers: [], magazines: [],
  };
  for (const c of design.components) {
    out.compMass_t += componentMass_t(c);
    switch (c.kind) {
      case "reactor":
        out.p_fusion += c.p_fusion_w || 0;
        out.rad_load_frac = c.rad_load_frac ?? out.rad_load_frac;
        break;
      case "nozzle": out.f_cap += c.f_max_n || 0; break;
      case "radiator_hot": out.radHot.push(c); break;
      case "radiator_low": out.radLow.push(c); break;
      case "heat_sink": out.sinkCap_mj += (c.li_t || 0) * 1000 *
        (c.energy_mj_per_kg || st.li_sink_mj_per_kg); break;
      case "flywheel": out.flyCap_mj += (c.mass_t || 0) * 1000 *
        (c.energy_mj_per_kg || st.flywheel_mj_per_t / 1000); break;
      case "laser": out.lasers.push(c); break;
      case "magazine": {
        const m = missileById(c.missile_id);
        const each_t = m ? missileWetKg(m) / 1000 : 0;
        out.ordnance_t += (c.capacity || 0) * each_t;
        out.magazines.push(c);
        break;
      }
      case "tank": out.tankCap_t += (c.mass_t || 0) /
        Math.max(c.tank_structure_frac || 1 / st.tank_prop_per_mass, 1e-12); break;
    }
  }
  out.dry_t = out.compMass_t + out.ordnance_t + (design.structure_t || 0);
  out.wet_t = out.dry_t * (design.mr || 1);
  out.propNeeded_t = out.wet_t - out.dry_t;
  return out;
}

// One entry per installed unit, for waste-heat and suite sums.
const expandLasers = lasers => lasers.flatMap(l =>
  Array(laserCount(l)).fill({ p_beam: l.p_beam_w, eta_wall: l.eta_wall, t_pulse: l.t_pulse_s }));

// Live dry mass: design dry minus expended ordnance.
function shipDry_t(ship) {
  const design = designById(ship.design_id);
  const s = sums(design);
  let expended = 0;
  for (const mag of s.magazines) {
    const m = missileById(mag.missile_id);
    const have = ship.magazines?.[mag.id] ?? mag.capacity;
    expended += (mag.capacity - have) * (m ? missileWetKg(m) / 1000 : 0);
  }
  return s.dry_t - expended;
}
const shipMass_t = ship => shipDry_t(ship) + (ship.propellant_t || 0);
const shipV = ship => (ship.velocity_kms || 0) * 1000; // m/s
const shipNavSpeed = ship => {
  const n = DB?.system?.nav?.[ship.id];
  return n ? Math.hypot(n.vx || 0, n.vy || 0) : shipV(ship);
};

function materialByName(name) {
  return DB.materials.find(m => m.name === name) || DB.materials[0];
}

/* ======================= events & state deltas ========================== */

function applyDeltas(ship, deltas, sign = 1) {
  for (const [k, v] of Object.entries(deltas || {})) {
    if (k.startsWith("mag:")) {
      const id = k.slice(4);
      ship.magazines = ship.magazines || {};
      ship.magazines[id] = (ship.magazines[id] || 0) + sign * v;
    } else {
      ship[k] = (ship[k] || 0) + sign * v;
    }
  }
}

function addEvent(ship, date, kind, note, deltas) {
  DB.events.push({ id: uid(), ship_id: ship.id, date, kind, note, deltas });
  applyDeltas(ship, deltas, 1);
  touch();
}

function deleteEvent(evId) {
  const i = DB.events.findIndex(e => e.id === evId);
  if (i < 0) return;
  const ev = DB.events[i];
  const ship = shipById(ev.ship_id);
  if (ship) applyDeltas(ship, ev.deltas, -1); // roll the deltas back
  DB.events.splice(i, 1);
  touch();
}

/* ============================== modal =================================== */

function modal(title, bodyHtml, { submitLabel = "Apply", onSubmit, wide } = {}) {
  const root = $("modal-root");
  modal._returnFocus = document.activeElement;
  root.innerHTML = `
    <div class="modal-back" id="modal-back">
      <div class="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title" tabindex="-1"
        ${wide ? 'style="max-width:920px;min-width:640px"' : ""}>
        <h2 id="modal-title">${esc(title)}</h2>
        <div id="modal-body">${bodyHtml}</div>
        <div class="modal-actions">
          <button id="m-cancel">Cancel</button>
          ${onSubmit ? `<button id="m-ok" class="primary">${esc(submitLabel)}</button>` : ""}
        </div>
      </div>
    </div>`;
  $("m-cancel").onclick = closeModal;
  $("modal-back").onclick = e => { if (e.target.id === "modal-back") closeModal(); };
  $("modal-back").onkeydown = e => { if (e.key === "Escape") closeModal(); };
  if (onSubmit) $("m-ok").onclick = async () => {
    try {
      const keep = await onSubmit();
      if (keep !== false) closeModal();
    } catch (e) { toast(e.message); }
  };
  const first = root.querySelector("input:not([disabled]), select:not([disabled]), button:not([disabled])");
  (first || root.querySelector(".modal")).focus();
}
function closeModal() {
  $("modal-root").innerHTML = "";
  if (modal._returnFocus?.isConnected) modal._returnFocus.focus();
}

/* =============================== tabs =================================== */

const TABS = [
  ["fleet", "Fleet"], ["designer", "Designer"], ["drive", "Drive & Travel"],
  ["laser", "Laser Lab"], ["lidar", "Lidar & PD"], ["missile", "Missile Lab"], ["map", "System Map"],
];
const UI = {
  tab: "fleet",
  shipId: null, designId: null,
  autosize: null,
  plan: { source: null, target: null, slider: 1000, reserve_kms: 5,
          depart_d: 30, arrive_d: 400, capture_mm: null },
  map: { view: null, sel: null, tick_s: null, nticks: 1, bodyOut: null,
         proj: { ships: {}, bodies: {} }, projKey: null, frame: "", node: null,
         intercept: null, pickTarget: false, playing: false, busy: false,
         layers: { rails: true, paths: true, vectors: true, labels: true, weapons: true } },
  drive: { source: null, slider: 1000, reserve_kms: 5, mode: "flip",
           dist: 1, unit: "AU", dir: 1, hours: 6, travel: null, burn: null, sprint: null },
  laser: { weapon: null, p_gw: 10, ap: 30, lambda_nm: 200, wall: 0.4, drill: null,
           profiles: null, clickR: null, pulse: 0.5, fly: 1800000, sink: 2300000,
           qlow: 0, deployed: false, graphMode: "damage", dirty: false },
  lidarPd: null,
  missile: { sel: null, range_mm: 100, vclose_kms: 20, phases: null, result: null,
             optimizer: null, optimizerResult: null },
};

function renderTabs() {
  const nav = $("tabs");
  nav.innerHTML = TABS.map(([id, label]) =>
    `<button data-tab="${id}" class="${UI.tab === id ? "active" : ""}">${label}</button>`
  ).join("") + `<button class="gear-btn" id="btn-settings" title="Canon constants & scaling">⚙</button>`;
  nav.querySelectorAll("[data-tab]").forEach(b =>
    b.onclick = () => { UI.tab = b.dataset.tab; render(); });
  $("btn-settings").onclick = settingsModal;
}

function render() {
  renderTabs();
  const main = $("main");
  main.classList.toggle("map-main", UI.tab === "map");
  if (!DB) { main.innerHTML = "<p class=note>Loading…</p>"; return; }
  switch (UI.tab) {
    case "fleet": return renderFleet(main);
    case "designer": return renderDesigner(main);
    case "drive": return renderDrive(main);
    case "laser": return renderLaser(main);
    case "lidar": return renderLidarPd(main);
    case "missile": return renderMissile(main);
    case "map": return renderMap(main);
  }
}

/* ============================ settings ================================== */

const SETTING_LABELS = {
  f_exh: "Exhaust power fraction f_exh",
  eta_noz: "Nozzle efficiency η_noz",
  ve_max_m_s: "Ve max (m/s)",
  ve_gear_min_m_s: "Ve gear minimum (m/s)",
  g: "g (m/s²)",
  sigma: "Stefan–Boltzmann σ",
  li_sink_mj_per_kg: "Li sink capacity (MJ/kg)",
  li_vent_mj_per_kg: "Li vent heat dump (MJ/kg)",
  flywheel_mj_per_t: "Flywheel capacity (MJ/t)",
  tank_prop_per_mass: "Tank capacity (t prop / t tank)",
  laser_cutoff_mm_s: "Laser effective cutoff (mm/s)",
  eta_drill: "Drilling efficiency η_drill",
  laser_eta_wall: "Laser wall-plug η default",
  pulse_missile_s: "Missile-kill pulse (s)",
  pulse_ship_s: "Ship-attack pulse (s)",
  kill_threshold_mm: "Kill threshold (mm)",
  open_fire_factor: "Open-fire doctrine ×kill range",
  prop_mh_ve_m_s: "Metallic hydrogen Ve (m/s)",
  prop_am_isp_s: "Antimatter thermal ISP default (s)",
  prop_fusion_isp_s: "Fusion bus ISP default (s)",
  as_target_accel_mg: "Auto-size: target accel (mg)",
  as_reactor_t_per_tw: "Auto-size: reactor t per TW",
  as_nozzle_cap_factor: "Auto-size: nozzle cap ÷ cruise thrust",
  as_nozzle_t_per_mn: "Auto-size: nozzle t per MN cap",
  as_rad_load_frac: "Auto-size: radiator load fraction",
  as_hot_t_k: "Auto-size: hot loop T (K)",
  as_hot_eps: "Auto-size: hot loop ε",
  as_hot_mw_per_kg: "Auto-size: hot loop MW/kg",
  as_low_area_frac: "Auto-size: low loop area ÷ hot",
  as_low_t_k: "Auto-size: low loop T (K)",
  as_low_eps: "Auto-size: low loop ε",
  as_low_mw_per_kg: "Auto-size: low loop MW/kg",
  as_structure_frac: "Auto-size: structure ÷ dry mass",
  as_sink_endurance_min: "Auto-size: sink endurance (min of laser fire)",
  as_sink_extra_mass_factor: "Auto-size: sink mass ÷ Li mass",
  as_flywheel_fire_s: "Auto-size: flywheel s of full-suite fire",
  g_const: "Gravitational constant G",
  c_m_s: "Lightspeed (m/s)",
  map_tick_s: "Map: default tick (s)",
  map_substep_s: "Map: integrator substep (s)",
  map_project_d: "Map: course projection (days)",
};

function settingsModal() {
  const rows = Object.entries(S()).map(([k, v]) => `
    <label>${esc(SETTING_LABELS[k] || k)}</label>
    <input type="number" step="any" id="set-${k}" value="${v}">`).join("");
  modal("Canon constants & scaling parameters", `
    <p class="note">All physics and auto-sizing reads these values — nothing is hardcoded.</p>
    <div class="grid2">${rows}</div>`, {
    submitLabel: "Save",
    async onSubmit() {
      for (const k of Object.keys(S())) {
        const v = num("set-" + k);
        if (Number.isFinite(v)) S()[k] = v;
      }
      await syncAllDesignerMasses();
      touch();
    },
  });
}

/* ============================== FLEET =================================== */

function bar(label, value, max, colorClass, text) {
  const pct = max > 0 ? Math.max(0, Math.min(100, 100 * value / max)) : 0;
  return `<div class="bar">
    <div class="bar-label"><span>${esc(label)}</span><span>${esc(text)}</span></div>
    <div class="bar-track"><div class="bar-fill ${colorClass}" style="width:${pct}%"></div></div>
  </div>`;
}
const pctColor = p => p >= 70 ? "good" : p >= 35 ? "warn" : "bad";

function renderFleet(main) {
  if (!UI.shipId && DB.states.length) UI.shipId = DB.states[0].id;
  const ship = shipById(UI.shipId);

  const shipRows = DB.states.map(s => {
    const d = designById(s.design_id);
    return `<tr class="clickable ${s.id === UI.shipId ? "sel" : ""}" data-ship="${s.id}">
      <td>${esc(s.name)}</td><td>${esc(d ? d.class : "?")}</td></tr>`;
  }).join("");

  const designOpts = DB.designs.map(d =>
    `<option value="${d.id}">${esc(d.name)}</option>`).join("");

  main.innerHTML = `<div class="cols">
    <div class="col-narrow">
      <div class="panel">
        <h2>Ships</h2>
        <table><tr><th>Name</th><th>Class</th></tr>${shipRows ||
          `<tr><td colspan=2 class=note>none — commission one below</td></tr>`}</table>
        <h3>Commission</h3>
        <div class="field"><select id="comm-design">${designOpts}</select></div>
        <div class="field"><input type="text" id="comm-name" class="wide" placeholder="Ship name"></div>
        <button id="btn-commission">Commission at full load</button>
      </div>
    </div>
    <div class="col" id="fleet-detail"></div>
  </div>`;

  main.querySelectorAll("[data-ship]").forEach(r =>
    r.onclick = () => { UI.shipId = r.dataset.ship; render(); });
  $("btn-commission").onclick = () => {
    const d = designById($("comm-design").value);
    const name = $("comm-name").value.trim() || "Unnamed " + d.class;
    const s = sums(d);
    const mags = {};
    for (const m of s.magazines) mags[m.id] = m.capacity;
    const ship = {
      id: uid(), name, design_id: d.id,
      propellant_t: Math.min(s.propNeeded_t, s.tankCap_t), velocity_kms: 0,
      sink_mj: 0, sink_capacity_mj: s.sinkCap_mj, flywheel_mj: s.flyCap_mj,
      radiator_hot_pct: 100, radiator_low_pct: 100, magazines: mags,
    };
    DB.states.push(ship);
    DB.events.push({ id: uid(), ship_id: ship.id, date: "", kind: "note",
                     note: "Commissioned at full load.", deltas: {} });
    UI.shipId = ship.id;
    touch();
  };

  if (ship) renderShipDetail($("fleet-detail"), ship);
}

const latestSummary = makeLatest();

function renderShipDetail(root, ship) {
  const design = designById(ship.design_id);
  const s = sums(design);

  const magBars = s.magazines.map(m => {
    const have = ship.magazines?.[m.id] ?? 0;
    const mi = missileById(m.missile_id);
    return bar(`${m.name} (${mi ? mi.name : m.missile_id})`, have, m.capacity,
               pctColor(100 * have / (m.capacity || 1)), `${have} / ${m.capacity}`);
  }).join("");

  const sinkFillPct = ship.sink_capacity_mj > 0 ? 100 * ship.sink_mj / ship.sink_capacity_mj : 0;
  const sinkColor = sinkFillPct < 50 ? "good" : sinkFillPct < 80 ? "warn" : "bad";
  const scarPct = s.sinkCap_mj > 0 ? 100 * ship.sink_capacity_mj / s.sinkCap_mj : 100;

  const events = DB.events.filter(e => e.ship_id === ship.id).slice().reverse();
  const evHtml = events.map(e => {
    const deltas = Object.entries(e.deltas || {})
      .map(([k, v]) => `${k} ${v >= 0 ? "+" : ""}${Math.abs(v) >= 100 ? Math.round(v).toLocaleString() : +v.toFixed(2)}`)
      .join(", ");
    return `<div class="event">
      <div class="ev-head">
        <span class="ev-date">${esc(e.date || "—")}</span>
        <span class="ev-kind">${esc(e.kind)}</span>
        <span class="ev-del"><button class="small danger" data-del-ev="${e.id}">✕</button></span>
      </div>
      <div class="ev-note">${esc(e.note || "")}</div>
      ${deltas ? `<div class="ev-deltas">${esc(deltas)}</div>` : ""}
    </div>`;
  }).join("");

  root.innerHTML = `
    <div class="panel" id="ship-summary">
      <h2>${esc(ship.name)} <span class="note">${esc(design.name)} — ${fmtT(shipMass_t(ship))} current</span></h2>
      <div class="readout" id="ship-summary-readout"><div class="r"><span class="note">Computing…</span></div></div>
      <div id="ship-summary-extra"></div>
    </div>
    <div class="panel">
      ${bar("Propellant", ship.propellant_t, s.tankCap_t, "",
            `${fmtT(ship.propellant_t)} / ${fmtT(s.tankCap_t)}`)}
      ${bar("Heat sink fill (full = bad)", ship.sink_mj, ship.sink_capacity_mj, sinkColor,
            `${fmtMJ(ship.sink_mj)} / ${fmtMJ(ship.sink_capacity_mj)}`)}
      ${scarPct < 99.9 ? bar("Sink capacity (vent scars)", ship.sink_capacity_mj, s.sinkCap_mj,
            "warn", `${scarPct.toFixed(1)}% of design`) : ""}
      ${bar("Flywheels", ship.flywheel_mj, s.flyCap_mj, "",
            `${fmtMJ(ship.flywheel_mj)} / ${fmtMJ(s.flyCap_mj)}`)}
      ${bar("Hot radiator integrity", ship.radiator_hot_pct, 100,
            pctColor(ship.radiator_hot_pct), ship.radiator_hot_pct.toFixed(0) + "%")}
      ${bar("Low-temp radiator integrity", ship.radiator_low_pct, 100,
            pctColor(ship.radiator_low_pct), ship.radiator_low_pct.toFixed(0) + "%")}
      ${magBars}
      <div class="actions">
        <button data-act="fire">Fire laser</button>
        <button data-act="launch">Launch missiles</button>
        <button data-act="radiate">Radiate</button>
        <button data-act="recharge">Recharge flywheels</button>
        <button data-act="vent">Vent sink</button>
        <button data-act="damage">Damage / Repair</button>
        <button data-act="resupply">Resupply</button>
        <button data-act="note">Note</button>
        <span style="flex:1"></span>
        <button class="danger small" id="btn-del-ship">Strike from list</button>
      </div>
      <p class="note">Burns live on the Drive &amp; Travel tab — they draw propellant and move the velocity ledger.</p>
    </div>
    <div class="panel inset">
      <h2>Event log</h2>
      ${evHtml || `<p class="note">No events.</p>`}
    </div>`;

  root.querySelectorAll("[data-act]").forEach(b =>
    b.onclick = () => ACTIONS[b.dataset.act](ship));
  root.querySelectorAll("[data-del-ev]").forEach(b =>
    b.onclick = () => { if (confirm("Delete event and roll back its deltas?")) deleteEvent(b.dataset.delEv); });
  $("btn-del-ship").onclick = () => {
    if (!confirm(`Strike ${ship.name} and her event log?`)) return;
    DB.events = DB.events.filter(e => e.ship_id !== ship.id);
    DB.states = DB.states.filter(x => x.id !== ship.id);
    UI.shipId = null;
    touch();
  };

  updateShipSummary(ship);
}

// Live totals from the Designer plus as-she-floats numbers: velocity, accel at
// both ends of the gear range, saved laser ranges, missile Δv per magazine.
async function updateShipSummary(ship) {
  await latestSummary(async fresh => {
    const st = S();
    const design = designById(ship.design_id);
    const s = sums(design);
    const m_kg = shipMass_t(ship) * 1000;

    const gearAt = ve => calc("gear", {
      p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
      ve, ve_max: st.ve_max_m_s, f_cap: s.f_cap || null, mass_kg: m_kg,
    });
    const [rep, gmax, gmin, dv, ...laserRes] = await Promise.all([
      calc("design_report", {
        p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
        ve_max: st.ve_max_m_s, f_cap: s.f_cap || null,
        m_dry: s.dry_t * 1000, m_wet: s.wet_t * 1000, dv_reserve: 0,
        rad_load_frac: s.rad_load_frac, sigma: st.sigma,
        rad_hot: s.radHot.map(r => ({ area: r.area_m2, t_k: r.t_k, eps: r.eps })),
        rad_low: s.radLow.map(r => ({ area: r.area_m2, t_k: r.t_k, eps: r.eps })),
        sink_mj: s.sinkCap_mj, flywheel_mj: s.flyCap_mj,
        lasers: expandLasers(s.lasers),
      }),
      gearAt(st.ve_max_m_s),
      gearAt(st.ve_gear_min_m_s),
      calc("deltav", { ve: st.ve_max_m_s, m_wet: m_kg, m_dry: shipDry_t(ship) * 1000 }),
      ...s.lasers.filter(l => (l.profiles || []).length).map(l =>
        calc("laser_profiles", {
          p_beam: l.p_beam_w, aperture: l.aperture_m, lambda: l.lambda_m,
          eta_drill: st.eta_drill, open_fire_factor: st.open_fire_factor,
          profiles: l.profiles.map(p => {
            const mat = materialByName(p.material);
            return { name: p.name, rho: mat.rho, e_vap_mj: mat.e_vap_mj,
                     t_pulse_s: p.t_pulse_s, threshold_mm: p.threshold_mm };
          }),
          n: 8,
        })),
    ]);
    if (!fresh() || !$("ship-summary-readout")) return;

    const nav = DB.system?.nav?.[ship.id];
    const mapRow = nav ? `
      <div class="r"><span class="k">System map</span>
        <span class="v accent">${nav.landed_on
          ? "landed — " + esc(DB.system.bodies.find(b => b.id === nav.landed_on)?.name || nav.landed_on)
          : fmtVel(Math.hypot(nav.vx, nav.vy))}</span>
        <span class="u">${nav.burn?.t_remaining_s > 0
          ? "burning, " + fmtDur(nav.burn.t_remaining_s) + " left"
          : "at " + epochStr()}</span></div>` : "";
    $("ship-summary-readout").innerHTML = `${mapRow}
      <div class="r"><span class="k">Velocity</span><span class="v accent">${fmtVel(shipV(ship))}</span>
        <span class="u">drive tab to burn</span></div>
      <div class="r"><span class="k">Accel @ max gear</span><span class="v">${fmtMg(gmax.accel)}</span>
        <span class="u">${fmtSI(gmax.thrust, "N")}</span></div>
      <div class="r"><span class="k">Accel @ min gear</span><span class="v">${fmtMg(gmin.accel)}</span>
        <span class="u">${fmtSI(gmin.thrust, "N")}${gmin.capped ? " capped" : ""}</span></div>
      <div class="r"><span class="k">Δv remaining</span><span class="v accent">${fmtVel(Math.max(0, dv.dv))}</span>
        <span class="u">pure plasma, no reserve</span></div>
      <div class="r"><span class="k">Hot loop vs waste</span>
        <span class="v ${rep.hot_margin_w < 0 ? "bad" : "good"}">${fmtSI(rep.hot_reject_w, "W")} / ${fmtSI(rep.waste_heat_w, "W")}</span></div>
      <div class="r"><span class="k">Sink vs own lasers</span>
        <span class="v">${fmtDur(rep.sink_endurance_retracted_s)}</span>
        <span class="u">low loop retracted</span></div>`;

    const lasersWithProfiles = s.lasers.filter(l => (l.profiles || []).length);
    const laserLines = lasersWithProfiles.map((l, i) => {
      const res = laserRes[i];
      const parts = res.profiles.map((p, j) => {
        const chosen = l.profiles[j].chosen_range_m;
        return `${esc(p.name)} <span class="v accent">${fmtDist(p.r_kill)}</span>` +
               ` <span class="u">(open ${fmtDist(p.r_open)}${chosen ? ", chosen " + fmtDist(chosen) : ""})</span>`;
      }).join(" · ");
      return `<tr><td>${esc(l.name)}${laserCount(l) > 1 ? " ×" + laserCount(l) : ""}</td><td>${parts}</td></tr>`;
    }).join("");
    const missileLines = (await Promise.all(s.magazines.map(async mag => {
      const mi = missileById(mag.missile_id);
      if (!mi) return "";
      const r = await calc("missile", missilePayload(mi));
      const have = ship.magazines?.[mag.id] ?? 0;
      return `<tr><td>${esc(mag.name)}</td><td>${esc(mi.name)} — Δv <span class="v accent">${fmtVel(r.dv)}</span>
        <span class="u">${esc(PROPULSION[mi.propulsion] || "custom")}, ${have} aboard</span></td></tr>`;
    }))).join("");
    if (!fresh() || !$("ship-summary-extra")) return;
    $("ship-summary-extra").innerHTML =
      (laserLines ? `<h3>Laser engagement ranges</h3><table>${laserLines}</table>` : "") +
      (missileLines ? `<h3>Ordnance</h3><table>${missileLines}</table>` : "");
  });
}

const dateField = `<div class="field"><label>Date</label>
  <input type="text" id="act-date" class="wide" placeholder="your calendar, your format"></div>`;
const noteField = `<div class="field"><label>Note</label>
  <input type="text" id="act-note" class="wide" placeholder="optional"></div>`;
const userNote = () => { const n = $("act-note")?.value.trim(); return n ? " — " + n : ""; };

const ACTIONS = {
  fire(ship) {
    const design = designById(ship.design_id), s = sums(design);
    if (!s.lasers.length) return toast("No lasers mounted on " + design.name);
    const opts = s.lasers.map(l => `<option value="${l.id}">${esc(l.name)}</option>`).join("");
    modal("Fire laser", `
      ${dateField}
      <div class="field"><label>Weapon</label><select id="act-laser">${opts}</select></div>
      <div class="field"><label>Shots</label><input type="number" id="act-n" value="1" min="1"></div>
      ${noteField}
      <p class="note">Per-shot bookkeeping: electrical = P·t/η_wall from flywheels; waste heat to the sink.
      Shots are per emitter at the weapon's own pulse length.</p>`, {
      async onSubmit() {
        const st = S();
        const l = s.lasers.find(x => x.id === $("act-laser").value);
        const n = Math.max(1, Math.round(num("act-n")));
        const r = await calc("laser", {
          p_beam: l.p_beam_w, aperture: l.aperture_m, lambda: l.lambda_m,
          eta_drill: st.eta_drill, cutoff_mm_s: st.laser_cutoff_mm_s,
          materials: [], eta_wall: l.eta_wall, t_pulse: l.t_pulse_s,
        });
        const elec = r.shot.electrical_mj * n, waste = r.shot.waste_mj * n;
        if (elec > ship.flywheel_mj + 1e-9)
          throw new Error(`Needs ${fmtMJ(elec)} from flywheels; only ${fmtMJ(ship.flywheel_mj)} charged.`);
        if (ship.sink_mj + waste > ship.sink_capacity_mj + 1e-9)
          throw new Error(`Sink would saturate (+${fmtMJ(waste)} onto ${fmtMJ(ship.sink_mj)} of ${fmtMJ(ship.sink_capacity_mj)}). Radiate or vent first.`);
        addEvent(ship, $("act-date").value, "laser",
          `${l.name}: ${n} shot${n > 1 ? "s" : ""} — ${fmtMJ(elec)} electrical, ${fmtMJ(waste)} waste heat` + userNote(),
          { flywheel_mj: -elec, sink_mj: waste });
      },
    });
  },

  launch(ship) {
    const design = designById(ship.design_id), s = sums(design);
    if (!s.magazines.length) return toast("No magazines on " + design.name);
    const opts = s.magazines.map(m => {
      const mi = missileById(m.missile_id);
      const have = ship.magazines?.[m.id] ?? 0;
      return `<option value="${m.id}">${esc(m.name)} — ${esc(mi ? mi.name : "?")} (${have} left)</option>`;
    }).join("");
    modal("Launch missiles", `
      ${dateField}
      <div class="field"><label>Magazine</label><select id="act-mag">${opts}</select></div>
      <div class="field"><label>Count</label><input type="number" id="act-n" value="1" min="1"></div>
      ${noteField}`, {
      onSubmit() {
        const magId = $("act-mag").value;
        const mag = s.magazines.find(m => m.id === magId);
        const n = Math.max(1, Math.round(num("act-n")));
        const have = ship.magazines?.[magId] ?? 0;
        if (n > have) throw new Error(`Only ${have} rounds left in ${mag.name}.`);
        const mi = missileById(mag.missile_id);
        addEvent(ship, $("act-date").value, "launch",
          `Launched ${n} × ${mi ? mi.name : mag.missile_id} from ${mag.name}` + userNote(),
          { ["mag:" + magId]: -n });
      },
    });
  },

  radiate(ship) {
    modal("Radiate (low-temp loop)", `
      ${dateField}
      <div class="field"><label>Duration (h)</label><input type="number" step="any" id="act-h" value="1"></div>
      ${noteField}
      <p class="note">Drains the sink at the low loop's current capability (area × T⁴ × integrity).</p>`, {
      async onSubmit() {
        const st = S();
        const design = designById(ship.design_id), s = sums(design);
        const hrs = num("act-h");
        if (!(hrs > 0)) throw new Error("Duration must be positive.");
        let q = 0;
        for (const r of s.radLow) {
          const res = await calc("radiator", {
            area: r.area_m2, t_k: r.t_k, eps: r.eps, sigma: st.sigma,
            integrity_pct: ship.radiator_low_pct,
          });
          q += res.q_w;
        }
        if (q <= 0) throw new Error("No functioning low-temp radiators.");
        const drain = Math.min(q * hrs * 3600 / 1e6, ship.sink_mj);
        addEvent(ship, $("act-date").value, "radiate",
          `Radiated ${hrs} h at ${fmtSI(q, "W")} — sink −${fmtMJ(drain)}` + userNote(),
          { sink_mj: -drain });
      },
    });
  },

  recharge(ship) {
    const design = designById(ship.design_id), s = sums(design);
    const room = Math.max(0, s.flyCap_mj - ship.flywheel_mj);
    modal("Recharge flywheels", `
      ${dateField}
      <div class="field"><label>Charge (MJ)</label>
        <input type="number" step="any" id="act-mj" value="${Math.round(room)}"></div>
      ${noteField}
      <p class="note">${fmtMJ(room)} of headroom in the banks.</p>`, {
      onSubmit() {
        const mj = Math.min(num("act-mj"), room);
        if (!(mj > 0)) throw new Error("Banks are already full.");
        addEvent(ship, $("act-date").value, "recharge",
          `Recharged flywheels +${fmtMJ(mj)}` + userNote(), { flywheel_mj: mj });
      },
    });
  },

  vent(ship) {
    modal("Vent sink", `
      ${dateField}
      <div class="field"><label>Heat to dump (MJ)</label>
        <input type="number" step="any" id="act-mj" value="${Math.round(ship.sink_mj)}"></div>
      ${noteField}
      <p class="note warn">Venting expels lithium: heat leaves at ${S().li_vent_mj_per_kg} MJ/kg,
      and sink capacity is permanently reduced by the mass vented. Emergency measure with a scar.</p>`, {
      async onSubmit() {
        const heat = Math.min(num("act-mj"), ship.sink_mj);
        if (!(heat > 0)) throw new Error("Nothing in the sink to vent.");
        const st = S();
        const v = await calc("vent", {
          heat_mj: heat, vent_mj_per_kg: st.li_vent_mj_per_kg,
          sink_mj_per_kg: st.li_sink_mj_per_kg,
        });
        addEvent(ship, $("act-date").value, "vent",
          `Vented ${fmtMJ(heat)} — ${v.li_kg.toFixed(0)} kg lithium expelled, capacity scarred −${fmtMJ(v.capacity_lost_mj)}` + userNote(),
          { sink_mj: -heat, sink_capacity_mj: -v.capacity_lost_mj });
      },
    });
  },

  damage(ship) {
    modal("Damage / Repair", `
      ${dateField}
      <div class="field"><label>System</label>
        <select id="act-sys">
          <option value="radiator_hot_pct">Hot radiator integrity</option>
          <option value="radiator_low_pct">Low-temp radiator integrity</option>
        </select></div>
      <div class="field"><label>Change (%, − damage / + repair)</label>
        <input type="number" step="any" id="act-d" value="-10"></div>
      ${noteField}`, {
      onSubmit() {
        const sys = $("act-sys").value;
        let d = num("act-d");
        if (!Number.isFinite(d) || d === 0) throw new Error("Enter a non-zero change.");
        d = Math.max(-(ship[sys] || 0), Math.min(100 - (ship[sys] || 0), d));
        addEvent(ship, $("act-date").value, d < 0 ? "damage" : "repair",
          `${sys.startsWith("radiator_hot") ? "Hot" : "Low-temp"} loop ${d < 0 ? "" : "+"}${d.toFixed(0)}%` + userNote(),
          { [sys]: d });
      },
    });
  },

  resupply(ship) {
    const design = designById(ship.design_id), s = sums(design);
    modal("Resupply", `
      ${dateField}${noteField}
      <p class="note">Tops up propellant, flywheels, and magazines; drains the sink and restores
      vented lithium to design capacity.</p>`, {
      onSubmit() {
        const deltas = {
          propellant_t: Math.min(s.propNeeded_t, s.tankCap_t) - ship.propellant_t,
          flywheel_mj: s.flyCap_mj - ship.flywheel_mj,
          sink_mj: -ship.sink_mj,
          sink_capacity_mj: s.sinkCap_mj - ship.sink_capacity_mj,
        };
        for (const m of s.magazines)
          deltas["mag:" + m.id] = m.capacity - (ship.magazines?.[m.id] ?? 0);
        for (const k of Object.keys(deltas)) if (Math.abs(deltas[k]) < 1e-9) delete deltas[k];
        addEvent(ship, $("act-date").value, "resupply", "Resupplied to full load" + userNote(), deltas);
      },
    });
  },

  note(ship) {
    modal("Log note", `${dateField}
      <div class="field"><label>Note</label><input type="text" id="act-note" class="wide"></div>`, {
      onSubmit() {
        const n = $("act-note").value.trim();
        if (!n) throw new Error("Empty note.");
        addEvent(ship, $("act-date").value, "note", n, {});
      },
    });
  },
};

/* ============================= DESIGNER ================================= */

const AUTO_MASS_KINDS = ["reactor", "nozzle", "radiator_hot", "radiator_low", "heat_sink",
  "flywheel", "laser", "magazine", "tank", "crew", "structure"];

const FLYWHEEL_MATERIALS = {
  "Carbon-fiber composite": 9,
  "CNT composite": 25,
  "Maraging steel": 0.5,
  "Custom": null,
};
const DESIGNER_PERCENT_FIELDS = new Set(["rad_load_frac", "laser_waste_frac", "tank_structure_frac", "structure_frac"]);
const DESIGNER_INTEGER_FIELDS = new Set(["capacity", "count", "crew_count"]);

const KIND_FIELDS = {
  reactor: [["p_fusion_w", "Fusion power (W)"], ["rad_load_frac", "Waste heat (%)"],
            ["mw_per_kg", "Specific power (MW/kg)"], ["target_accel_mg", "Target wet accel (mg)"]],
  nozzle: [["f_max_n", "Maximum thrust (N)"], ["mw_per_kg", "Specific power (MW/kg)"]],
  radiator_hot: [["area_m2", "Rejection area (m²)"], ["radiator_mode", "Mass model"],
                 ["mw_per_kg", "Specific rejection (MW/kg)"], ["kg_per_m2", "Areal density (kg/m²)"],
                 ["mw_per_m2", "Surface rejection (MW/m²)"]],
  radiator_low: [["area_m2", "Rejection area (m²)"], ["laser_waste_frac", "Laser waste heat (%)"],
                 ["radiator_mode", "Mass model"], ["mw_per_kg", "Specific rejection (MW/kg)"],
                 ["kg_per_m2", "Areal density (kg/m²)"], ["mw_per_m2", "Surface rejection (MW/m²)"]],
  heat_sink: [["li_t", "Heat-storage material (t)"], ["energy_mj_per_kg", "Heat capacity (MJ/kg)"],
              ["installed_mass_factor", "Installed / material mass"], ["endurance_s", "Laser operation (s)"]],
  flywheel: [["material", "Flywheel material"], ["energy_mj_per_kg", "Storage (MJ/kg)"],
             ["endurance_s", "Laser operation (s)"]],
  laser: [["p_beam_w", "Beam power (W)"], ["mw_per_kg", "Specific power (MW/kg)"],
          ["aperture_m", "Aperture (m)"], ["lambda_m", "Wavelength (m)"],
          ["eta_wall", "Wall-plug η"], ["t_pulse_s", "Pulse (s)"], ["count", "Count"]],
  magazine: [["missile_id", "Missile design"], ["capacity", "Capacity (rounds)"],
             ["missile_mass_ratio", "Missile mass / structure mass"]],
  tank: [["tank_structure_frac", "Tank structure (% of propellant)"]],
  crew: [["crew_count", "Crew members"], ["tonnes_per_crew", "Tonnes / crew member"]],
  structure: [["structure_frac", "Mass (% of rest of ship)"]],
  other: [["note", "Note"]],
};

// Sane-option choices per kind.field; everything keeps a custom escape hatch.
const FIELD_CHOICES = {
  "laser.lambda_m": [{ v: 2e-7, label: "200 nm near-UV (vacuum)" },
                     { v: 5.32e-7, label: "532 nm green (atmo)" }],
  "laser.t_pulse_s": [{ v: 0.01, label: "0.01 s missile kill" },
                      { v: 0.5, label: "0.5 s ship attack" }],
  "radiator_hot.t_k": [{ v: 1800, label: "1800 K refractory" },
                       { v: 2000, label: "2000 K refractory" },
                       { v: 2500, label: "2500 K CNT-reinforced" }],
  "radiator_low.t_k": [{ v: 400, label: "400 K" }, { v: 500, label: "500 K" }, { v: 600, label: "600 K" }],
};

function compSummary(c) {
  switch (c.kind) {
    case "reactor": return fmtSI(c.p_fusion_w, "W") + ", load " + (100 * (c.rad_load_frac || 0)).toFixed(0) + "%";
    case "nozzle": return "cap " + fmtSI(c.f_max_n, "N");
    case "radiator_hot": case "radiator_low":
      return fmtSI(c.area_m2, "m²") + " @ " + c.t_k + " K, ε " + c.eps +
             " → " + fmtSI(radiatorPower_w(c), "W") + ", " + fmtMwKg(radiatorMwKg(c)) + " MW/kg";
    case "heat_sink": return c.li_t + " t storage → " +
      fmtMJ(c.li_t * (c.energy_mj_per_kg || S().li_sink_mj_per_kg) * 1000);
    case "flywheel": return `${c.material || "Custom"} → ` + fmtMJ((c.mass_t || 0) * 1000 * (c.energy_mj_per_kg || 0));
    case "laser": return (laserCount(c) > 1 ? "×" + laserCount(c) + " " : "") +
                         fmtSI(c.p_beam_w, "W") + ", " + c.aperture_m + " m, " +
                         fmtSI(c.lambda_m, "m") + `, ${(c.profiles || []).length} profiles`;
    case "magazine": {
      const m = missileById(c.missile_id);
      return (m ? m.name : c.missile_id) + " × " + c.capacity +
             " (" + fmtT(c.capacity * (m ? missileWetKg(m) / 1000 : 0)) + " ordnance)";
    }
    case "tank": return "→ " + fmtT((c.mass_t || 0) /
      Math.max(c.tank_structure_frac || 1 / S().tank_prop_per_mass, 1e-12)) + " propellant";
    case "crew": return `${c.crew_count || 0} crew @ ${trim3(c.tonnes_per_crew || 0)} t/person`;
    case "structure": return `${((c.structure_frac || 0) * 100).toFixed(1)}% of the rest of the dry ship`;
    default: return c.note || "";
  }
}

const latestDesigner = makeLatest();
const designerSyncQueues = new WeakMap();
const COMPONENT_GROUPS = [
  ["Drive", ["reactor", "nozzle"]],
  ["Thermal", ["radiator_hot", "radiator_low", "heat_sink", "flywheel"]],
  ["Weapons", ["laser"]],
  ["Stores & crew", ["magazine", "tank", "crew"]],
  ["Structure & other", ["structure", "other"]],
];

async function syncDesignerMasses(design, action = null) {
  const previous = designerSyncQueues.get(design) || Promise.resolve();
  const request = previous.catch(() => {}).then(() => calc("designer", {
    settings: S(), missiles: DB.missiles, design, action,
  }));
  designerSyncQueues.set(design, request);
  const result = await request;
  Object.assign(design, result.design);
  return result;
}

async function syncAllDesignerMasses() {
  await Promise.all(DB.designs.map(design => syncDesignerMasses(design)));
}

function inlineComponentFields(c) {
  return KIND_FIELDS[c.kind].map(([k, label]) => {
    let value = k === "mw_per_kg" ? radiatorMwKg(c) : c[k];
    if (DESIGNER_PERCENT_FIELDS.has(k) && Number.isFinite(value)) value *= 100;
    if (k === "missile_id") return `<label>${esc(label)}</label><select data-ci="${c.id}" data-key="${k}">
      ${DB.missiles.map(m => `<option value="${m.id}" ${m.id === c.missile_id ? "selected" : ""}>${esc(m.name)}</option>`).join("")}</select>`;
    if (k === "radiator_mode") return `<label>${esc(label)}</label><select data-ci="${c.id}" data-key="${k}">
      <option value="specific_power" ${c.radiator_mode !== "areal" ? "selected" : ""}>MW/kg</option>
      <option value="areal" ${c.radiator_mode === "areal" ? "selected" : ""}>kg/m² + MW/m²</option></select>`;
    if (k === "material") return `<label>${esc(label)}</label><select data-ci="${c.id}" data-key="${k}">
      ${Object.keys(FLYWHEEL_MATERIALS).map(name => `<option ${name === c.material ? "selected" : ""}>${esc(name)}</option>`).join("")}</select>`;
    const type = k === "note" ? "text" : "number";
    return `<label>${esc(label)}</label><input type="${type}" ${type === "number" ? `step="${DESIGNER_INTEGER_FIELDS.has(k) ? 1 : "any"}"` : ""}
      ${DESIGNER_PERCENT_FIELDS.has(k) ? 'min="0" max="100"' : ""}
      ${DESIGNER_INTEGER_FIELDS.has(k) ? 'min="0"' : ""}
      data-ci="${c.id}" data-key="${k}" value="${esc(value ?? "")}">`;
  }).join("");
}

function componentSizingActions(c) {
  switch (c.kind) {
    case "reactor": return `<button class="small" data-size-component="${c.id}" data-size-mode="reactor-min">Auto-size power at min gear</button>
      <button class="small" data-size-component="${c.id}" data-size-mode="reactor-max">Auto-size power at max gear</button>`;
    case "nozzle": return `<button class="small" data-size-component="${c.id}" data-size-mode="nozzle">Auto-size for min exhaust velocity</button>`;
    case "radiator_hot": return `<button class="small" data-size-component="${c.id}" data-size-mode="radiator-hot">Reject reactor waste heat</button>`;
    case "radiator_low": return `<button class="small" data-size-component="${c.id}" data-size-mode="radiator-low">Reject selected laser waste heat</button>`;
    case "heat_sink": return `<button class="small" data-size-component="${c.id}" data-size-mode="heat-sink">Accept laser heat for set time</button>`;
    case "flywheel": return `<button class="small" data-size-component="${c.id}" data-size-mode="flywheel">Power lasers for set time</button>`;
    case "tank": return `<button class="small" data-size-component="${c.id}" data-size-mode="tank">Size tank structure for mass ratio</button>`;
    case "magazine": return `<button class="small" data-size-component="${c.id}" data-size-mode="magazine">Size structure from missile load</button>`;
    case "structure": return `<button class="small" data-size-component="${c.id}" data-size-mode="structure">Calculate from rest of ship</button>`;
    default: return "";
  }
}

function componentCard(c) {
  const lab = c.kind === "laser" ? `<button class="small" data-open-laser="${c.id}">Open Laser Lab</button>`
    : c.kind === "magazine" ? `<button class="small" data-open-missile="${c.missile_id}">Open Missile Lab</button>` : "";
  return `<details class="component-card" open data-component="${c.id}">
    <summary><span class="component-kind">${esc(c.kind)}</span>
      <strong>${esc(c.name)}</strong><span class="component-summary" data-comp-summary="${c.id}">${esc(compSummary(c))}</span>
      <span class="component-mass" data-comp-mass="${c.id}">${fmtT(componentMass_t(c))}</span></summary>
    <div class="component-editor">
      <div class="component-fields">
        <label>Name</label><input type="text" data-ci="${c.id}" data-key="name" value="${esc(c.name)}">
        ${inlineComponentFields(c)}
        ${AUTO_MASS_KINDS.includes(c.kind) ? `<label>Mass</label><label class="matcheck"><input type="checkbox"
          data-mass-override="${c.id}" ${c.mass_override ? "checked" : ""}> manual override</label>` : ""}
        ${c.mass_override || !AUTO_MASS_KINDS.includes(c.kind) ? `<label>${c.kind === "laser" ? "Manual mass / unit (t)" : "Manual mass (t)"}</label>
          <input type="number" step="any" data-ci="${c.id}" data-key="mass_t" value="${c.mass_t || 0}">` : ""}
      </div>
      <div class="actions">${componentSizingActions(c)}${lab}<button class="small" data-dup="${c.id}">Duplicate</button>
        <button class="small danger" data-del="${c.id}">Delete</button></div>
    </div></details>`;
}

function renderDesigner(main) {
  if (!UI.designId && DB.designs.length) UI.designId = DB.designs[0].id;
  const design = designById(UI.designId);

  const rows = DB.designs.map(d =>
    `<tr class="clickable ${d.id === UI.designId ? "sel" : ""}" data-des="${d.id}">
      <td>${esc(d.name)}</td><td>${esc(d.class)}</td></tr>`).join("");

  main.innerHTML = `<div class="cols">
    <div class="col-narrow">
      <div class="panel">
        <h2>Designs</h2>
        <table><tr><th>Name</th><th>Class</th></tr>${rows}</table>
        <div class="actions">
          <button id="btn-new-design">New</button>
          <button id="btn-dup-design">Duplicate</button>
          <button id="btn-del-design" class="danger">Delete</button>
        </div>
      </div>
    </div>
    <div class="col" id="designer-detail"></div>
  </div>`;

  main.querySelectorAll("[data-des]").forEach(r =>
    r.onclick = () => { UI.designId = r.dataset.des; UI.autosize = null; render(); });
  $("btn-new-design").onclick = async () => {
    const d = { id: uid(), name: "New design", class: "custom", mr: 4,
                structure_t: 0, structure_auto: false, components: [
                  { id: uid(), kind: "structure", name: "Primary structure", structure_frac: S().as_structure_frac,
                    mass_t: 0, mass_override: false }
                ] };
    DB.designs.push(d); UI.designId = d.id; UI.autosize = null;
    try { await syncDesignerMasses(d); touch(); }
    catch (e) { toast(e.message); }
  };
  $("btn-dup-design").onclick = () => {
    if (!design) return;
    const copy = JSON.parse(JSON.stringify(design));
    copy.id = uid(); copy.name = design.name + " (copy)";
    copy.components.forEach(c => c.id = uid());
    DB.designs.push(copy); UI.designId = copy.id; UI.autosize = null; touch();
  };
  $("btn-del-design").onclick = () => {
    if (!design) return;
    if (DB.states.some(s => s.design_id === design.id))
      return toast("Ships are flying this design — strike them first.");
    if (!confirm(`Delete ${design.name}?`)) return;
    DB.designs = DB.designs.filter(d => d.id !== design.id);
    UI.designId = null; touch();
  };

  if (design) renderDesignDetail($("designer-detail"), design);
}

function renderDesignDetail(root, design) {
  const grouped = COMPONENT_GROUPS.map(([label, kinds]) => {
    const comps = design.components.filter(c => kinds.includes(c.kind));
    return `<section class="component-group"><h3>${label} <span class="pill">${comps.length}</span></h3>
      ${comps.map(componentCard).join("") || `<p class="note">No ${label.toLowerCase()} components.</p>`}</section>`;
  }).join("");

  const kindOpts = Object.keys(KIND_FIELDS).map(k => `<option>${k}</option>`).join("");

  root.innerHTML = `
    <div class="panel">
      <h2>Design</h2>
      <div class="field"><label>Name</label><input type="text" id="des-name" class="wide" value="${esc(design.name)}"></div>
      <div class="field"><label>Class</label><input type="text" id="des-class" value="${esc(design.class)}"></div>
      <div class="field"><label>Mass ratio</label><input type="number" step="any" id="des-mr" value="${design.mr}"></div>
      ${design.note ? `<div class="flag">${esc(design.note)}</div>` : ""}
      <h3>Components</h3>
      <div class="design-groups">${grouped}</div>
      <div class="actions">
        <select id="add-kind">${kindOpts}</select>
        <button id="btn-add-comp">Add component</button>
      </div>
    </div>
    <div class="panel inset sticky-report" id="design-report"><p class="note">Computing…</p></div>`;

  const commit = () => {
    design.name = $("des-name").value;
    design.class = $("des-class").value;
    const mr = num("des-mr");
    if (Number.isFinite(mr) && mr > 1) design.mr = mr;
    scheduleDesignerSync();
  };
  ["des-name", "des-class", "des-mr"].forEach(id => $(id).oninput = commit);

  const updateCard = c => {
    const summary = root.querySelector(`[data-comp-summary="${c.id}"]`);
    const mass = root.querySelector(`[data-comp-mass="${c.id}"]`);
    const title = root.querySelector(`[data-component="${c.id}"] > summary strong`);
    if (title) title.textContent = c.name;
    if (summary) summary.textContent = compSummary(c);
    if (mass) mass.textContent = fmtT(componentMass_t(c));
    clearTimeout(renderDesignDetail._t);
    renderDesignDetail._t = setTimeout(() => updateDesignReport(design), 120);
  };
  const scheduleDesignerSync = () => {
    clearTimeout(renderDesignDetail._syncT);
    renderDesignDetail._syncT = setTimeout(async () => {
      try {
        await syncDesignerMasses(design);
        touch(false);
        design.components.forEach(updateCard);
      } catch (e) { toast(e.message); }
    }, 120);
  };
  root.querySelectorAll("[data-ci]").forEach(inp => {
    const apply = () => {
      const c = design.components.find(x => x.id === inp.dataset.ci);
      const k = inp.dataset.key;
      if (!c) return;
      if (k === "name" || k === "note" || k === "missile_id" || k === "radiator_mode" || k === "material") c[k] = inp.value;
      else {
        const v = parseFloat(inp.value);
        if (Number.isFinite(v)) c[k] = DESIGNER_PERCENT_FIELDS.has(k) ? v / 100
          : DESIGNER_INTEGER_FIELDS.has(k) ? Math.max(0, Math.round(v)) : v;
      }
      if (k === "material" && FLYWHEEL_MATERIALS[inp.value] != null) {
        c.energy_mj_per_kg = FLYWHEEL_MATERIALS[inp.value];
        const energy = root.querySelector(`[data-ci="${c.id}"][data-key="energy_mj_per_kg"]`);
        if (energy) energy.value = c.energy_mj_per_kg;
      }
      scheduleDesignerSync();
    };
    inp.oninput = inp.tagName === "INPUT" ? apply : null;
    inp.onchange = apply;
  });
  root.querySelectorAll("[data-mass-override]").forEach(inp => inp.onchange = async () => {
    const c = design.components.find(x => x.id === inp.dataset.massOverride);
    c.mass_override = inp.checked;
    try { await syncDesignerMasses(design); touch(); }
    catch (e) { toast(e.message); }
  });
  root.querySelectorAll("[data-del]").forEach(b =>
    b.onclick = async () => {
      design.components = design.components.filter(c => c.id !== b.dataset.del);
      try { await syncDesignerMasses(design); touch(); }
      catch (e) { toast(e.message); }
    });
  root.querySelectorAll("[data-dup]").forEach(b => b.onclick = async () => {
    const c = design.components.find(x => x.id === b.dataset.dup);
    const copy = JSON.parse(JSON.stringify(c)); copy.id = uid(); copy.name += " copy";
    design.components.splice(design.components.indexOf(c) + 1, 0, copy);
    try { await syncDesignerMasses(design); touch(); }
    catch (e) { toast(e.message); }
  });
  root.querySelectorAll("[data-open-laser]").forEach(b => b.onclick = () => {
    UI.laser.weapon = design.id + ":" + b.dataset.openLaser; UI.tab = "laser"; render();
  });
  root.querySelectorAll("[data-open-missile]").forEach(b => b.onclick = () => {
    UI.missile.sel = b.dataset.openMissile; UI.missile.phases = defaultPhases(missileById(b.dataset.openMissile));
    UI.tab = "missile"; render();
  });
  root.querySelectorAll("[data-size-component]").forEach(b => b.onclick = async () => {
    const c = design.components.find(x => x.id === b.dataset.sizeComponent);
    if (!c) return;
    try { await sizeDesignerComponent(design, c, b.dataset.sizeMode); touch(); }
    catch (e) { toast(e.message); }
  });
  $("btn-add-comp").onclick = async () => {
    const kind = $("add-kind").value;
    const st = S();
    const c = { id: uid(), kind, name: kind, mass_t: 100, mass_override: kind === "other" };
    if (kind === "magazine") { c.missile_id = DB.missiles[0]?.id; c.capacity = 10; c.missile_mass_ratio = 10; }
    if (kind === "reactor") {
      c.p_fusion_w = 1e12; c.rad_load_frac = st.as_rad_load_frac;
      c.mw_per_kg = 1.333333; c.target_accel_mg = st.as_target_accel_mg;
    }
    if (kind === "nozzle") { c.f_max_n = 1e7; c.mw_per_kg = 4; }
    if (kind.startsWith("radiator")) {
      c.area_m2 = 1e5;
      c.t_k = kind === "radiator_hot" ? st.as_hot_t_k : st.as_low_t_k;
      c.eps = kind === "radiator_hot" ? st.as_hot_eps : st.as_low_eps;
      c.mw_per_kg = radiatorDefaultMwKg(kind);
      c.mw_per_m2 = radiatorFlux_w_m2(c.t_k, c.eps) / 1e6;
      c.kg_per_m2 = c.mw_per_m2 / c.mw_per_kg;
      c.radiator_mode = "specific_power";
      if (kind === "radiator_low") c.laser_waste_frac = 1;
    }
    if (kind === "heat_sink") {
      c.li_t = 50; c.energy_mj_per_kg = st.li_sink_mj_per_kg;
      c.installed_mass_factor = st.as_sink_extra_mass_factor; c.endurance_s = st.as_sink_endurance_min * 60;
    }
    if (kind === "flywheel") {
      c.material = "Carbon-fiber composite"; c.energy_mj_per_kg = FLYWHEEL_MATERIALS[c.material];
      c.endurance_s = st.as_flywheel_fire_s;
    }
    if (kind === "laser") {
      c.p_beam_w = 1.5e9; c.aperture_m = 3; c.lambda_m = 2e-7;
      c.mw_per_kg = 0.006;
      c.eta_wall = st.laser_eta_wall; c.t_pulse_s = st.pulse_missile_s; c.count = 1;
      c.profiles = [{ id: uid(), name: "Missile kill",
                      material: "Ti-C hybrid (missile hull)",
                      t_pulse_s: st.pulse_missile_s, threshold_mm: st.kill_threshold_mm }];
    }
    if (kind === "tank") c.tank_structure_frac = 1 / st.tank_prop_per_mass;
    if (kind === "crew") { c.crew_count = 20; c.tonnes_per_crew = 2; }
    if (kind === "structure") c.structure_frac = st.as_structure_frac;
    design.components.push(c);
    try { await syncDesignerMasses(design); touch(); }
    catch (e) { toast(e.message); }
  };
  updateDesignReport(design);
}

async function sizeDesignerComponent(design, c, mode) {
  const result = await syncDesignerMasses(design, { component_id: c.id, mode });
  const resized = design.components.find(component => component.id === c.id);
  const detail = result.action?.message || `auto-sized to ${fmtT(componentMass_t(resized))}`;
  toast(`${resized?.name || c.name}: ${detail}.`, result.action?.feasible !== false);
}

async function updateDesignReport(design) {
  await latestDesigner(async fresh => {
    const st = S(), s = sums(design);
    const el = () => $("design-report");
    if (!s.p_fusion) {
      if (fresh() && el()) el().innerHTML = `<p class="note warn">No reactor — add one to see drive numbers.</p>`;
      return;
    }
    const rep = await calc("design_report", {
      p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
      ve_max: st.ve_max_m_s, f_cap: s.f_cap || null,
      m_dry: s.dry_t * 1000, m_wet: s.wet_t * 1000, dv_reserve: 0,
      rad_load_frac: s.rad_load_frac, sigma: st.sigma,
      rad_hot: s.radHot.map(r => ({ area: r.area_m2, t_k: r.t_k, eps: r.eps })),
      rad_low: s.radLow.map(r => ({ area: r.area_m2, t_k: r.t_k, eps: r.eps })),
      sink_mj: s.sinkCap_mj, flywheel_mj: s.flyCap_mj,
      lasers: expandLasers(s.lasers),
    });
    if (!fresh() || !el()) return;

    const warns = [];
    const tankShort = s.propNeeded_t - s.tankCap_t;
    if (Math.abs(tankShort) > 0.005 * Math.max(s.propNeeded_t, 1))
      warns.push(tankShort > 0
        ? `Tankage short: MR ${design.mr} needs ${fmtT(s.propNeeded_t)} but tanks hold ${fmtT(s.tankCap_t)} (−${fmtT(tankShort)}).`
        : `Excess tankage: ${fmtT(-tankShort)} beyond the MR ${design.mr} load.`);
    if (rep.hot_margin_w < 0)
      warns.push(`Hot radiators under-built: reject ${fmtSI(rep.hot_reject_w, "W")} vs ${fmtSI(rep.waste_heat_w, "W")} reactor waste load.`);
    if (rep.capped_at_ve_max)
      warns.push("Nozzle cap binds even at Ve_max — the drive can never use full jet power.");

    // Suite rows: one per weapon type, per-unit shot figures.
    let li = 0;
    const shotRows = s.lasers.map(l => {
      const sh = rep.laser_shots[li];
      li += laserCount(l);
      return `<tr><td>${esc(l.name)}${laserCount(l) > 1 ? " ×" + laserCount(l) : ""}</td>
        <td class="num">${fmtMJ(sh.electrical_mj)}</td>
        <td class="num">${fmtMJ(sh.waste_mj)}</td>
        <td class="num">${sh.shots_per_bank.toFixed(0)}</td></tr>`;
    }).join("");

    el().innerHTML = `
      <h2>Live totals</h2>
      <div class="readout">
        <div class="r"><span class="k">Dry mass</span><span class="v">${fmtT(s.dry_t)}</span>
          <span class="u">${fmtT(s.compMass_t)} installed components + ${fmtT(s.ordnance_t)} loaded ordnance</span></div>
        <div class="r"><span class="k">Wet mass</span><span class="v">${fmtT(s.wet_t)}</span><span class="u">MR ${design.mr}</span></div>
        <div class="r"><span class="k">Propellant / tankage</span>
          <span class="v ${Math.abs(tankShort) > 0.005 * Math.max(s.propNeeded_t, 1) ? "warn" : "good"}">${fmtT(s.propNeeded_t)} / ${fmtT(s.tankCap_t)}</span></div>
        <div class="r"><span class="k">Thrust @ Ve_max</span><span class="v accent">${fmtSI(rep.thrust_max, "N")}</span></div>
        <div class="r"><span class="k">Accel wet → dry</span>
          <span class="v">${fmtMg(rep.accel_wet)} → ${fmtMg(rep.accel_dry)}</span></div>
        <div class="r"><span class="k">Pure-plasma Δv</span><span class="v accent">${fmtVel(rep.dv_plasma)}</span><span class="u">no reserve</span></div>
        <div class="r"><span class="k">Hot loop vs waste</span>
          <span class="v ${rep.hot_margin_w < 0 ? "bad" : "good"}">${fmtSI(rep.hot_reject_w, "W")} vs ${fmtSI(rep.waste_heat_w, "W")}</span></div>
        <div class="r"><span class="k">Low loop</span><span class="v">${fmtSI(rep.low_reject_w, "W")}</span></div>
        <div class="r"><span class="k">Sink vs own lasers</span>
          <span class="v">${fmtDur(rep.sink_endurance_retracted_s)}</span>
          <span class="u">low loop retracted; deployed: ${fmtDur(rep.sink_endurance_s)}</span></div>
      </div>
      ${warns.map(w => `<p class="note warn">⚠ ${esc(w)}</p>`).join("")}
      <div class="actions"><button id="design-open-drive">Open in Drive &amp; Travel</button></div>
      ${s.lasers.length ? `<h3>Laser suite vs flywheels (${fmtMJ(s.flyCap_mj)}) — per emitter</h3>
      <table><tr><th>Weapon</th><th class="num">Electrical/shot</th><th class="num">Waste/shot</th><th class="num">Shots per bank</th></tr>
      ${shotRows}</table>` : ""}`;
    $("design-open-drive").onclick = () => {
      UI.plan.source = "design:" + design.id; UI.tab = "drive"; render();
    };
  });
}

/* ========================== DRIVE & TRAVEL ============================== */

const latestGear = makeLatest();
const latestCurve = makeLatest();

function massSources() {
  const out = [];
  for (const d of DB.designs) {
    const s = sums(d);
    out.push({ id: "design:" + d.id, label: `${d.name} (paper, ${fmtT(s.wet_t)} wet)`,
               m0_t: s.wet_t, dry_t: s.dry_t, design: d, ship: null });
  }
  for (const sh of DB.states) {
    const d = designById(sh.design_id);
    if (!d) continue;
    out.push({ id: "ship:" + sh.id,
               label: `${sh.name} (as she floats, ${fmtT(shipMass_t(sh))}, ${fmtVel(shipNavSpeed(sh))}${DB.system?.nav?.[sh.id] ? " map" : " ledger"})`,
               m0_t: shipMass_t(sh), dry_t: shipDry_t(sh), design: d, ship: sh });
  }
  return out;
}

const sliderToVe = k => {
  const lo = S().ve_gear_min_m_s, hi = S().ve_max_m_s;
  return lo * Math.pow(hi / lo, k / 1000);
};

function renderDrive(main) {
  const sources = massSources();
  if (!sources.length) { main.innerHTML = `<p class="note">No designs.</p>`; return; }
  if (!UI.plan.source || !sources.find(s => s.id === UI.plan.source))
    UI.plan.source = sources[0].id;
  const src = sources.find(s => s.id === UI.plan.source);
  const geo = src.ship && DB.system?.nav?.[src.ship.id] ? missionGeometry(src.ship) : null;

  const srcOpts = sources.map(s =>
    `<option value="${s.id}" ${s.id === UI.plan.source ? "selected" : ""}>${esc(s.label)}</option>`).join("");

  const modes = [["flip", "Flip-and-burn (arrive at rest)"],
                 ["burn", "Timed burn (duration → Δv, distance)"],
                 ["sprint", "Sprint intercept (max velocity, no decel)"]];

  main.innerHTML = `
    <div class="panel">
      <h2>Drive gearing</h2>
      <div class="field"><label>Mass from</label><select id="drv-source">${srcOpts}</select></div>
      <div class="field"><label>Δv reserve (km/s)</label>
        <input type="number" step="any" id="drv-reserve" value="${UI.plan.reserve_kms}"></div>
      ${src.ship ? `<div class="field"><label>Current velocity</label>
        <span class="v accent">${fmtVel(shipNavSpeed(src.ship))}</span>
        <span class="note">${DB.system?.nav?.[src.ship.id] ? "live map vector" : "velocity ledger"}</span></div>` : ""}
      <div class="gear-slider">
        <input type="range" id="drv-slider" min="0" max="1000" value="${UI.plan.slider}">
        <div class="gear-ends"><span>afterburner ${fmtVel(S().ve_gear_min_m_s)}</span>
          <span>pure plasma ${fmtVel(S().ve_max_m_s)}</span></div>
      </div>
      <div class="readout" id="drv-readout"><div class="r"><span class="note">…</span></div></div>
      <p class="note" id="drv-capnote"></p>
      <canvas class="plot" id="drv-curve" height="260"></canvas>
    </div>
    <div class="panel">
      <h2>Travel solvers</h2>
      ${geo ? `<div class="plan-context"><span class="chip active"><span>Map target</span><b>${esc(geo.name)}</b></span>
        <span>${fmtDist(geo.distance)} · ${fmtVel(geo.radialClosing)} radial closing</span>
        <button id="trv-use-map" class="small">Use map separation</button>
        <button id="trv-open-map" class="small">Open map plan</button></div>` : ""}
      <div class="field"><label>Solver</label>
        <select id="trv-mode">${modes.map(([v, l]) =>
          `<option value="${v}" ${UI.drive.mode === v ? "selected" : ""}>${l}</option>`).join("")}</select></div>
      <div id="trv-inputs"></div>
      <div id="trv-out"></div>
    </div>`;

  $("drv-source").onchange = () => { UI.plan.source = $("drv-source").value; renderDrive(main); };
  $("drv-reserve").onchange = () => {
    UI.plan.reserve_kms = num("drv-reserve") || 0;
    updateGearReadout(); updateDriveCurve();
  };
  $("drv-slider").oninput = () => {
    UI.plan.slider = +$("drv-slider").value;
    clearTimeout(renderDrive._t);
    renderDrive._t = setTimeout(updateGearReadout, 60);
  };
  $("trv-mode").onchange = () => { UI.drive.mode = $("trv-mode").value; renderSolverInputs(); };
  if ($("trv-use-map")) $("trv-use-map").onclick = () => {
    UI.drive.dist = geo.distance / 1e6; UI.drive.unit = "Mm"; renderSolverInputs();
  };
  if ($("trv-open-map")) $("trv-open-map").onclick = () => { UI.map.sel = "ship:" + src.ship.id; UI.tab = "map"; render(); };

  renderSolverInputs();
  updateGearReadout();
  updateDriveCurve();
}

const driveSource = () => massSources().find(s => s.id === UI.plan.source);

function renderSolverInputs() {
  const el = $("trv-inputs");
  if (!el) return;
  const src = driveSource();
  const distField = `
    <div class="field"><label>Distance</label>
      <input type="number" step="any" id="trv-dist" value="${UI.drive.dist}">
      <select id="trv-unit">${Object.keys(DIST_UNITS).map(u =>
        `<option ${u === UI.drive.unit ? "selected" : ""}>${u}</option>`).join("")}</select></div>`;
  if (UI.drive.mode === "flip") {
    el.innerHTML = `${distField}
      <button id="trv-solve" class="primary">Solve</button>
      <p class="note">Rest-to-rest brachistochrone at this gear; current velocity is ignored.</p>`;
  } else if (UI.drive.mode === "burn") {
    el.innerHTML = `
      <div class="field"><label>Duration (h)</label>
        <input type="number" step="any" id="trv-hours" value="${UI.drive.hours}"></div>
      <div class="field"><label>Direction</label>
        <select id="trv-dir">
          <option value="1" ${UI.drive.dir > 0 ? "selected" : ""}>prograde (accelerate)</option>
          <option value="-1" ${UI.drive.dir < 0 ? "selected" : ""}>retrograde (decelerate)</option>
        </select></div>
      <button id="trv-solve" class="primary">Solve</button>
      ${src.ship ? `<span class="note">starts from ${esc(src.ship.name)}'s ${fmtVel(shipNavSpeed(src.ship))}</span>` : ""}`;
  } else {
    el.innerHTML = `${distField}
      <button id="trv-solve" class="primary">Solve</button>
      <p class="note">Burns everything above the reserve floor accelerating, then coasts.
      You arrive fast and you do not stop.</p>`;
  }
  $("trv-solve").onclick = solveTravel;
  $("trv-out").innerHTML = "";
}

async function updateGearReadout() {
  await latestGear(async fresh => {
    const st = S(), src = driveSource();
    if (!src) return;
    const s = sums(src.design);
    const ve = sliderToVe(UI.plan.slider);
    if (!s.p_fusion) {
      if (fresh() && $("drv-readout")) $("drv-readout").innerHTML =
        `<div class="r"><span class="note warn">No reactor in this design.</span></div>`;
      return;
    }
    const m0_kg = src.m0_t * 1000;
    const [g, dv] = await Promise.all([
      calc("gear", { p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
                     ve, ve_max: st.ve_max_m_s, f_cap: s.f_cap || null, mass_kg: m0_kg }),
      calc("deltav", { ve, m_wet: m0_kg, m_dry: src.dry_t * 1000,
                       dv_reserve: UI.plan.reserve_kms * 1000 }),
    ]);
    if (!fresh() || !$("drv-readout")) return;
    const abFrac = g.mdot > 0 ? g.mdot_afterburner / g.mdot : 0;
    $("drv-readout").innerHTML = `
      <div class="r"><span class="k">Gear Ve</span><span class="v accent">${fmtVel(ve)}</span></div>
      <div class="r"><span class="k">Thrust</span><span class="v ${g.capped ? "warn" : ""}">${fmtSI(g.thrust, "N")}</span>
        <span class="u">${g.capped ? "nozzle cap" : "jet-limited"}</span></div>
      <div class="r"><span class="k">Accel</span><span class="v">${fmtMg(g.accel)}</span>
        <span class="u">${fmtSI(g.accel, "m/s²")}</span></div>
      <div class="r"><span class="k">Fuel flow</span><span class="v">${fmtSI(g.mdot_fuel * 1000, "g/s")}</span></div>
      <div class="r"><span class="k">Afterburner</span><span class="v ${abFrac > 0.5 ? "warn" : ""}">${fmtSI(g.mdot_afterburner * 1000, "g/s")}</span>
        <span class="u">${(100 * abFrac).toFixed(0)}% of flow</span></div>
      <div class="r"><span class="k">Δv at this gear</span>
        <span class="v ${dv.dv <= 0 ? "bad" : "accent"}">${fmtVel(Math.max(0, dv.dv))}</span>
        <span class="u">${UI.plan.reserve_kms} km/s reserve held</span></div>`;
    $("drv-capnote").textContent = g.capped
      ? `Nozzle cap binds below Ve ≈ ${fmtVel(g.ve_cap)} — the gear slider flattens here: more afterburner buys no more thrust.`
      : (g.ve_cap ? `Nozzle cap would bind below Ve ≈ ${fmtVel(g.ve_cap)}.` : "");
  });
}

async function updateDriveCurve() {
  await latestCurve(async fresh => {
    const st = S(), src = driveSource();
    if (!src) return;
    const s = sums(src.design);
    if (!s.p_fusion) return;
    const c = await calc("drive_curve", {
      p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
      ve_min: st.ve_gear_min_m_s, ve_max: st.ve_max_m_s, f_cap: s.f_cap || null,
      m_wet: src.m0_t * 1000, m_dry: src.dry_t * 1000,
      dv_reserve: UI.plan.reserve_kms * 1000,
    });
    if (!fresh() || !$("drv-curve")) return;
    const kms = c.ve.map(v => v / 1000);
    Plot.draw($("drv-curve"), {
      xlog: true,
      xlabel: "Ve (km/s)", ylabel: "thrust (N)", y2label: "Δv (km/s)",
      series: [
        { x: kms, y: c.thrust, label: "thrust", color: "#5fb4e8" },
        c.dv ? { x: kms, y: c.dv.map(v => v / 1000), label: "Δv", color: "#f0a050", axis: "y2" } : null,
      ].filter(Boolean),
      vlines: [
        c.ve_cap && c.ve_cap > st.ve_gear_min_m_s
          ? { x: c.ve_cap / 1000, label: "nozzle cap", color: "#e8d05f" } : null,
        { x: sliderToVe(UI.plan.slider) / 1000, label: "gear", color: "#7ed491" },
      ].filter(Boolean),
      height: 260,
    });
  });
}

async function solveTravel() {
  try {
    const st = S(), src = driveSource();
    const s = sums(src.design);
    const ve = sliderToVe(UI.plan.slider);
    const g = await calc("gear", {
      p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
      ve, ve_max: st.ve_max_m_s, f_cap: s.f_cap || null,
    });
    const m0 = src.m0_t * 1000;
    const dvr = await calc("deltav", { ve, m_wet: m0, m_dry: src.dry_t * 1000,
                                       dv_reserve: UI.plan.reserve_kms * 1000 });
    const m_floor = dvr.m_floor;

    if (UI.drive.mode === "flip") {
      UI.drive.dist = num("trv-dist");
      UI.drive.unit = $("trv-unit").value;
      const dist_m = UI.drive.dist * DIST_UNITS[UI.drive.unit];
      if (!(dist_m > 0)) throw new Error("Distance must be positive.");
      const t = await calc("travel", {
        distance: dist_m, ve, thrust: g.thrust, mdot: g.mdot,
        m0, m_dry: src.dry_t * 1000, dv_reserve: UI.plan.reserve_kms * 1000,
      });
      renderFlipResult(t);
    } else if (UI.drive.mode === "burn") {
      UI.drive.hours = num("trv-hours");
      UI.drive.dir = +$("trv-dir").value;
      if (!(UI.drive.hours > 0)) throw new Error("Duration must be positive.");
      const b = await calc("burn", {
        v0: src.ship ? shipNavSpeed(src.ship) : 0,
        duration_s: UI.drive.hours * 3600, ve, thrust: g.thrust, mdot: g.mdot,
        m0, m_floor, direction: UI.drive.dir,
      });
      renderBurnResult(b, src, ve, g);
    } else {
      UI.drive.dist = num("trv-dist");
      UI.drive.unit = $("trv-unit").value;
      const dist_m = UI.drive.dist * DIST_UNITS[UI.drive.unit];
      if (!(dist_m > 0)) throw new Error("Distance must be positive.");
      const sp = await calc("sprint", {
        distance: dist_m, v0: src.ship ? shipNavSpeed(src.ship) : 0,
        ve, thrust: g.thrust, mdot: g.mdot, m0, m_floor,
      });
      renderSprintResult(sp);
    }
  } catch (e) { toast(e.message); }
}

function renderFlipResult(t) {
  const out = $("trv-out");
  out.innerHTML = `
    ${t.feasible ? "" : `<p class="note bad">Infeasible at this gear and reserve — max reachable
      distance is ${fmtDist(t.max_distance)} (${(t.max_distance / DIST_UNITS.AU).toFixed(2)} AU).
      Numbers below are that maximum burn.</p>`}
    <div class="readout">
      <div class="r"><span class="k">Transit</span><span class="v accent">${fmtDur(t.t_total)}</span></div>
      <div class="r"><span class="k">Flip at</span><span class="v">${fmtDur(t.t_flip)}</span></div>
      <div class="r"><span class="k">Peak velocity</span><span class="v">${fmtVel(t.peak_v)}</span></div>
      <div class="r"><span class="k">Propellant</span><span class="v">${fmtT(t.prop_used_kg / 1000)}</span></div>
      <div class="r"><span class="k">Δv spent</span><span class="v">${fmtVel(t.dv_spent)}</span></div>
      <div class="r"><span class="k">Arrival mass</span><span class="v">${fmtT(t.m_arrival / 1000)}</span>
        <span class="u">floor ${fmtT(t.m_floor / 1000)}</span></div>
    </div>
    <canvas class="plot" id="trv-plot" height="240"></canvas>`;
  const days = t.profile.map(p => p[0] / 86400);
  Plot.draw($("trv-plot"), {
    xlabel: "time (days)", ylabel: "velocity (km/s)", y2label: "distance (Gm)",
    series: [
      { x: days, y: t.profile.map(p => p[2] / 1000), label: "velocity", color: "#5fb4e8" },
      { x: days, y: t.profile.map(p => p[1] / 1e9), label: "distance", color: "#7ed491", axis: "y2" },
    ],
    vlines: [{ x: t.t_flip / 86400, label: "flip", color: "#e8d05f" }],
    height: 240,
  });
}

function renderBurnResult(b, src, ve, gear) {
  const out = $("trv-out");
  out.innerHTML = `
    ${b.clamped ? `<p class="note warn">Clamped at the reserve floor after ${fmtDur(b.t)}.</p>` : ""}
    <div class="readout">
      <div class="r"><span class="k">Δv</span><span class="v accent">${UI.drive.dir > 0 ? "+" : "−"}${fmtVel(b.dv)}</span></div>
      <div class="r"><span class="k">End velocity</span><span class="v">${fmtVel(b.v_end)}</span></div>
      <div class="r"><span class="k">Distance covered</span><span class="v">${fmtDist(b.distance)}</span>
        <span class="u">during the burn</span></div>
      <div class="r"><span class="k">Propellant</span><span class="v">${fmtT(b.prop_used_kg / 1000)}</span></div>
      <div class="r"><span class="k">Coast rate after</span><span class="v">${fmtDist(Math.abs(b.v_end) * 86400)}/day</span></div>
    </div>
    ${src.ship ? `
      <div class="field"><label>Date</label><input type="text" id="burn-date" class="wide"
        placeholder="your calendar, your format"></div>
      <button id="btn-log-burn" class="primary">Log burn to ${esc(src.ship.name)}</button>` : ""}
    <canvas class="plot" id="trv-plot" height="240"></canvas>`;
  if (src.ship) $("btn-log-burn").onclick = () => {
    const ship = src.ship;
    const mapped = DB.system?.nav?.[ship.id];
    if (mapped) {
      if (mapped.burn) return toast("This ship already has a programmed map burn.");
      mapped.burn = {
        thrust_n: gear.thrust, mdot_kg_s: gear.mdot, ve_m_s: ve,
        mode: UI.drive.dir > 0 ? "prograde" : "retrograde", angle_deg: 0,
        target_body: null, t_start_s: 0, t_remaining_s: b.t,
        prop_drawn_t: 0, dv_gained: 0,
      };
      toast("Programmed on the System Map; propellant will be drawn as time advances.", true);
      touch(false);
      return;
    }
    const prop_t = b.prop_used_kg / 1000;
    if (prop_t > ship.propellant_t + 1e-9)
      return toast(`Needs ${fmtT(prop_t)} propellant; only ${fmtT(ship.propellant_t)} aboard.`);
    addEvent(ship, $("burn-date").value, "burn",
      `Burn ${fmtDur(b.t)} @ ${fmtVel(ve)} ${UI.drive.dir > 0 ? "prograde" : "retrograde"} — Δv ${UI.drive.dir > 0 ? "+" : "−"}${fmtVel(b.dv)}`,
      { propellant_t: -prop_t, velocity_kms: (UI.drive.dir > 0 ? 1 : -1) * b.dv / 1000 });
    toast("Logged — velocity and propellant updated.", true);
  };
  const hrs = b.profile.map(p => p[0] / 3600);
  Plot.draw($("trv-plot"), {
    xlabel: "time (h)", ylabel: "velocity (km/s)", y2label: "distance (Mm)",
    series: [
      { x: hrs, y: b.profile.map(p => p[2] / 1000), label: "velocity", color: "#5fb4e8" },
      { x: hrs, y: b.profile.map(p => p[1] / 1e6), label: "distance", color: "#7ed491", axis: "y2" },
    ],
    height: 240,
  });
}

function renderSprintResult(sp) {
  const out = $("trv-out");
  out.innerHTML = `
    ${sp.hit ? "" : `<p class="note bad">${esc(sp.miss_reason || "No intercept.")}</p>`}
    <div class="readout">
      ${sp.hit ? `<div class="r"><span class="k">Time to target</span><span class="v accent">${fmtDur(sp.t_total)}</span></div>
      <div class="r"><span class="k">Velocity at target</span><span class="v accent">${fmtVel(sp.v_arrival)}</span>
        <span class="u">and you do not stop</span></div>` : ""}
      <div class="r"><span class="k">Burnout</span><span class="v">${fmtDur(sp.t_burn)}</span></div>
      <div class="r"><span class="k">Δv spent</span><span class="v">${fmtVel(sp.dv_spent)}</span></div>
      <div class="r"><span class="k">Propellant</span><span class="v">${fmtT(sp.prop_used_kg / 1000)}</span>
        <span class="u">arrives at ${fmtT(sp.m_end / 1000)}</span></div>
    </div>
    <canvas class="plot" id="trv-plot" height="240"></canvas>`;
  const days = sp.profile.map(p => p[0] / 86400);
  Plot.draw($("trv-plot"), {
    xlabel: "time (days)", ylabel: "velocity (km/s)", y2label: "distance (Gm)",
    series: [
      { x: days, y: sp.profile.map(p => p[2] / 1000), label: "velocity", color: "#5fb4e8" },
      { x: days, y: sp.profile.map(p => p[1] / 1e9), label: "distance", color: "#7ed491", axis: "y2" },
    ],
    vlines: [{ x: sp.t_burn / 86400, label: "burnout", color: "#e8d05f" }],
    height: 240,
  });
}

/* ============================ LASER LAB ================================= */

const latestLaser = makeLatest();

function defaultProfiles() {
  const st = S();
  return [{ id: uid(), name: "Missile kill", material: "Ti-C hybrid (missile hull)",
            t_pulse_s: st.pulse_missile_s, threshold_mm: st.kill_threshold_mm }];
}

function laserWeapons() {
  const out = [];
  for (const d of DB.designs)
    for (const c of d.components)
      if (c.kind === "laser") out.push({ id: d.id + ":" + c.id, d, c });
  return out;
}

function renderLaser(main) {
  const st = S();
  const L = UI.laser;
  if (L.drill == null) L.drill = st.eta_drill;
  if (!L.profiles) L.profiles = defaultProfiles();
  if (L.weapon && L.loadedWeapon !== L.weapon) {
    const w = laserWeapons().find(x => x.id === L.weapon);
    if (w) {
      L.p_gw = w.c.p_beam_w / 1e9; L.ap = w.c.aperture_m;
      L.lambda_nm = w.c.lambda_m * 1e9; L.wall = w.c.eta_wall;
      L.pulse = w.c.t_pulse_s;
      L.profiles = (w.c.profiles || []).length ? JSON.parse(JSON.stringify(w.c.profiles)) : defaultProfiles();
      const s = sums(w.d); L.fly = s.flyCap_mj; L.sink = s.sinkCap_mj;
      L.qlow = Math.round(s.radLow.reduce((q, r) => q + radiatorPower_w(r), 0));
      L.dirty = false; L.loadedWeapon = L.weapon;
    }
  }

  const weapons = laserWeapons();
  const wOpts = [`<option value="custom">— custom parameters —</option>`,
    ...weapons.map(w => `<option value="${w.id}" ${w.id === L.weapon ? "selected" : ""}>
      ${esc(w.d.name)} — ${esc(w.c.name)}</option>`)].join("");

  const matOpts = sel => DB.materials.map(m =>
    `<option value="${esc(m.name)}" ${m.name === sel ? "selected" : ""}>${esc(m.name)}</option>`).join("");
  const pulseOpts = [{ v: st.pulse_missile_s, label: st.pulse_missile_s + " s missile" },
                     { v: st.pulse_ship_s, label: st.pulse_ship_s + " s ship" }];

  const profRows = L.profiles.map((p, i) => `
    <tr>
      <td><span style="color:${Plot.PALETTE[i % Plot.PALETTE.length]}">■</span>
        <input type="text" id="pf-name-${i}" value="${esc(p.name)}" style="width:130px"></td>
      <td><select id="pf-mat-${i}">${matOpts(p.material)}</select></td>
      <td>${choiceField("pf-pulse-" + i, p.t_pulse_s, pulseOpts)}</td>
      <td><input type="number" step="any" id="pf-thr-${i}" value="${p.threshold_mm}" style="width:70px"> mm</td>
      <td>${p.chosen_range_m ? fmtDist(p.chosen_range_m) : "—"}</td>
      <td><button class="small" data-pf-up="${i}" ${i === 0 ? "disabled" : ""}>↑</button>
        <button class="small" data-pf-down="${i}" ${i === L.profiles.length - 1 ? "disabled" : ""}>↓</button>
        <button class="small" data-pf-copy="${i}">duplicate</button>
        <button class="small danger" data-pf-del="${i}" aria-label="Delete profile">✕</button></td>
    </tr>`).join("");

  main.innerHTML = `
    <div class="panel">
      <h2>Laser Lab</h2>
      <div class="field"><label>Weapon</label><select id="las-weapon">${wOpts}</select></div>
      <span class="pill ${L.dirty ? "on" : ""}" id="las-dirty">${L.dirty ? "unsaved draft" : "saved"}</span>
      <div>
        <div class="field"><label>Beam (GW)</label><input type="number" step="any" id="las-p" value="${L.p_gw}"></div>
        <div class="field"><label>Aperture (m)</label><input type="number" step="any" id="las-ap" value="${L.ap}"></div>
        <div class="field"><label>λ</label>${choiceField("las-nm", L.lambda_nm,
          [{ v: 200, label: "200 nm near-UV" }, { v: 532, label: "532 nm green" }])} nm</div>
        <div class="field"><label>η_wall</label><input type="number" step="any" id="las-wall" value="${L.wall}"></div>
        <div class="field"><label>η_drill</label><input type="number" step="any" id="las-drill" value="${L.drill}"></div>
      </div>
      <h3>Kill profiles — each line is a pulse length against a material</h3>
      <table><tr><th>Profile</th><th>Material</th><th>Pulse</th><th>Threshold</th><th>Chosen range</th><th></th></tr>
        ${profRows}</table>
      <div class="actions">
        <button id="btn-pf-add">+ profile</button>
        ${L.weapon ? `<button id="btn-pf-save" class="primary">Save laser</button>`
          : `<button id="btn-las-save-new" class="primary">Save to design…</button>`}
      </div>
      <div class="segmented" id="las-graph-mode">
        ${[["damage","Damage"],["spot","Spot"],["power","Power"],["energy","Energy"]].map(([v,l]) =>
          `<button data-las-mode="${v}" class="${L.graphMode === v ? "active" : ""}">${l}</button>`).join("")}
      </div>
      <canvas class="plot" id="las-plot" height="300" title="click to pick a range"></canvas>
      <div id="las-click" class="note">Click the graph to inspect a range and save it to a profile.</div>
      <div id="las-table"></div>
      <p class="note">Honest limits: pure vaporization-drilling model. Ignores melt ejection
      (flatters thick targets) and lateral thermal bleed (flatters the laser at low flux).
      η_drill is your fudge knob. Doctrine: open fire at ${st.open_fire_factor}× kill range —
      a blinded missile is a dead missile.</p>
    </div>
    <div class="panel">
      <h2>Power &amp; heat</h2>
      <div class="field"><label>Pulse (s)</label>${choiceField("las-pulse", L.pulse, pulseOpts)}</div>
      <div class="field"><label>Flywheel bank (MJ)</label><input type="number" step="any" id="las-fly" value="${L.fly}"></div>
      <div class="field"><label>Sink capacity (MJ)</label><input type="number" step="any" id="las-sink" value="${L.sink}"></div>
      <div class="field"><label>Low loop (W)</label><input type="number" step="any" id="las-qlow" value="${L.qlow}"></div>
      <div class="field"><label class="matcheck"><input type="checkbox" id="las-deployed" ${L.deployed ? "checked" : ""}>
        low loop deployed while firing</label></div>
      <div class="readout" id="las-power"></div>
    </div>`;

  const rereadProfiles = () => {
    L.profiles.forEach((p, i) => {
      p.name = $("pf-name-" + i).value;
      p.material = $("pf-mat-" + i).value;
      p.t_pulse_s = num("pf-pulse-" + i);
      p.threshold_mm = num("pf-thr-" + i);
    });
  };
  const reread = () => {
    L.p_gw = num("las-p"); L.ap = num("las-ap"); L.lambda_nm = num("las-nm");
    L.wall = num("las-wall"); L.drill = num("las-drill");
    L.pulse = num("las-pulse"); L.fly = num("las-fly"); L.sink = num("las-sink");
    L.qlow = num("las-qlow"); L.deployed = $("las-deployed").checked;
    rereadProfiles();
    L.dirty = true;
    if ($("las-dirty")) { $("las-dirty").textContent = "unsaved draft"; $("las-dirty").classList.add("on"); }
    updateLaser();
  };
  const queueReread = () => { clearTimeout(renderLaser._input); renderLaser._input = setTimeout(reread, 100); };
  ["las-p", "las-ap", "las-nm", "las-wall", "las-drill",
   "las-pulse", "las-fly", "las-sink", "las-qlow"].forEach(id => {
    $(id).oninput = queueReread; $(id).onchange = reread;
  });
  $("las-deployed").onchange = reread;
  bindChoice("las-nm", reread);
  bindChoice("las-pulse", reread);
  L.profiles.forEach((p, i) => {
    ["pf-name-" + i, "pf-mat-" + i, "pf-thr-" + i].forEach(id => {
      $(id).oninput = queueReread; $(id).onchange = reread;
    });
    bindChoice("pf-pulse-" + i, reread);
  });
  main.querySelectorAll("[data-pf-del]").forEach(b =>
    b.onclick = () => { L.profiles.splice(+b.dataset.pfDel, 1); L.dirty = true; renderLaser(main); });
  const moveProfile = (i, d) => { rereadProfiles(); const [p] = L.profiles.splice(i, 1);
    L.profiles.splice(i + d, 0, p); L.dirty = true; renderLaser(main); };
  main.querySelectorAll("[data-pf-up]").forEach(b => b.onclick = () => moveProfile(+b.dataset.pfUp, -1));
  main.querySelectorAll("[data-pf-down]").forEach(b => b.onclick = () => moveProfile(+b.dataset.pfDown, 1));
  main.querySelectorAll("[data-pf-copy]").forEach(b => b.onclick = () => {
    rereadProfiles(); const p = JSON.parse(JSON.stringify(L.profiles[+b.dataset.pfCopy]));
    p.id = uid(); p.name += " copy"; L.profiles.splice(+b.dataset.pfCopy + 1, 0, p); L.dirty = true; renderLaser(main);
  });
  $("btn-pf-add").onclick = () => {
    rereadProfiles();
    L.profiles.push({ id: uid(), name: "Ship kill", material: "Tungsten (radiator)",
                      t_pulse_s: st.pulse_ship_s, threshold_mm: st.kill_threshold_mm });
    L.dirty = true;
    renderLaser(main);
  };
  if (L.weapon) $("btn-pf-save").onclick = async () => {
    rereadProfiles();
    const [did, cid] = L.weapon.split(":");
    const comp = designById(did)?.components.find(c => c.id === cid);
    if (!comp) return toast("Weapon no longer exists.");
    comp.profiles = JSON.parse(JSON.stringify(L.profiles));
    comp.p_beam_w = L.p_gw * 1e9; comp.aperture_m = L.ap;
    comp.lambda_m = L.lambda_nm * 1e-9; comp.eta_wall = L.wall; comp.t_pulse_s = L.pulse;
    await syncDesignerMasses(designById(did));
    L.dirty = false;
    if ($("las-dirty")) { $("las-dirty").textContent = "saved"; $("las-dirty").classList.remove("on"); }
    toast(`Saved ${comp.name} with ${L.profiles.length} profiles.`, true);
    touch(false);
  };
  if (!L.weapon) $("btn-las-save-new").onclick = () => saveLaserToDesign();
  main.querySelectorAll("[data-las-mode]").forEach(b => b.onclick = () => {
    L.graphMode = b.dataset.lasMode; updateLaser();
    main.querySelectorAll("[data-las-mode]").forEach(x => x.classList.toggle("active", x === b));
  });

  $("las-weapon").onchange = async () => {
    const v = $("las-weapon").value;
    L.weapon = v === "custom" ? null : v;
    L.loadedWeapon = L.weapon;
    L.clickR = null;
    if (v === "custom") L.dirty = true;
    if (v !== "custom") {
      const [did] = v.split(":");
      const w = laserWeapons().find(x => x.id === v);
      L.p_gw = w.c.p_beam_w / 1e9; L.ap = w.c.aperture_m;
      L.lambda_nm = w.c.lambda_m * 1e9; L.wall = w.c.eta_wall;
      L.pulse = w.c.t_pulse_s;
      L.profiles = (w.c.profiles || []).length
        ? JSON.parse(JSON.stringify(w.c.profiles)) : defaultProfiles();
      L.dirty = false;
      const s = sums(designById(did));
      L.fly = s.flyCap_mj; L.sink = s.sinkCap_mj;
      // Low-loop capability comes from the radiator endpoint, not local math.
      let q = 0;
      for (const r of s.radLow) {
        const res = await calc("radiator", { area: r.area_m2, t_k: r.t_k, eps: r.eps, sigma: st.sigma });
        q += res.q_w;
      }
      L.qlow = Math.round(q);
    }
    renderLaser(main);
  };

  $("las-plot").onclick = ev => {
    const r = Plot.xFromEvent($("las-plot"), ev);
    if (r == null) return;
    L.clickR = r * 1e6; // plot x-axis is in Mm
    updateLaser();
  };

  updateLaser();
}

function saveLaserToDesign() {
  const L = UI.laser;
  const opts = DB.designs.map(d => `<option value="${d.id}">${esc(d.name)}</option>`).join("");
  modal("Install laser on a design", `<div class="grid2">
    <label>Design</label><select id="ls-design">${opts}</select>
    <label>Component name</label><input id="ls-name" value="New laser" class="wide">
    <label>Count</label><input type="number" min="1" step="1" id="ls-count" value="1">
    <label>Specific power (MW/kg)</label><input type="number" min="0" step="any" id="ls-mwkg" value="0.006">
    <label>Default pulse (s)</label><input type="number" step="any" id="ls-pulse" value="${L.pulse}">
  </div><p class="note">This installs on the shared design; every commissioned ship using that design inherits it.</p>`, {
    submitLabel: "Install & save",
    async onSubmit() {
      const d = designById($("ls-design").value);
      const c = { id: uid(), kind: "laser", name: $("ls-name").value.trim() || "Laser",
        mass_t: 0, mass_override: false, mw_per_kg: Math.max(num("ls-mwkg") || 0.006, 1e-12),
        count: Math.max(1, Math.round(num("ls-count") || 1)),
        p_beam_w: L.p_gw * 1e9, aperture_m: L.ap, lambda_m: L.lambda_nm * 1e-9,
        eta_wall: L.wall, t_pulse_s: num("ls-pulse") || L.pulse,
        profiles: JSON.parse(JSON.stringify(L.profiles)) };
      d.components.push(c);
      await syncDesignerMasses(d);
      L.weapon = d.id + ":" + c.id; L.loadedWeapon = L.weapon; L.pulse = c.t_pulse_s; L.dirty = false;
      touch(); toast(`Installed ${c.name} on ${d.name}.`, true);
    },
  });
}

async function updateLaser() {
  await latestLaser(async fresh => {
    const st = S(), L = UI.laser;
    const valid = L.profiles.filter(p => materialByName(p.material) && p.t_pulse_s > 0 && p.threshold_mm > 0);
    const [prof, power] = await Promise.all([
      valid.length ? calc("laser_profiles", {
        p_beam: L.p_gw * 1e9, aperture: L.ap, lambda: L.lambda_nm * 1e-9,
        eta_drill: L.drill, open_fire_factor: st.open_fire_factor,
        profiles: valid.map(p => {
          const mat = materialByName(p.material);
          return { name: p.name, rho: mat.rho, e_vap_mj: mat.e_vap_mj,
                   t_pulse_s: p.t_pulse_s, threshold_mm: p.threshold_mm };
        }),
      }) : null,
      calc("laser", {
        p_beam: L.p_gw * 1e9, aperture: L.ap, lambda: L.lambda_nm * 1e-9,
        eta_drill: L.drill, cutoff_mm_s: st.laser_cutoff_mm_s, materials: [],
        eta_wall: L.wall, t_pulse: L.pulse,
        flywheel_mj: L.fly, sink_mj: L.sink, q_low_w: L.qlow,
      }),
    ]);
    if (!fresh() || !$("las-plot")) return;

    if (prof) {
      const mm = prof.range_m.map(x => x / 1e6); // Mm
      let series, ylabel, y2label = null;
      if (L.graphMode === "spot") {
        ylabel = "spot diameter (m)";
        series = [{ x: mm, y: prof.spot_diameter_m, label: "Airy spot", color: "#5fb4e8" }];
      } else if (L.graphMode === "power") {
        ylabel = "irradiance (W/m²)"; y2label = "beam power (W)";
        series = [
          { x: mm, y: prof.irradiance_w_m2, label: "irradiance", color: "#f0a050" },
          { x: mm, y: mm.map(() => prof.beam_power_w), label: "total beam power", color: "#5fb4e8", axis: "y2" },
        ];
      } else if (L.graphMode === "energy") {
        ylabel = "fluence (J/m²)"; y2label = "pulse energy (J)";
        series = prof.profiles.flatMap((p, i) => [
          { x: mm, y: p.fluence_j_m2, label: `${p.name} fluence`, color: Plot.PALETTE[i % Plot.PALETTE.length] },
          { x: mm, y: mm.map(() => p.pulse_energy_j), label: `${p.name} total`,
            color: Plot.PALETTE[i % Plot.PALETTE.length], axis: "y2", dash: true },
        ]);
      } else {
        ylabel = "penetration per pulse (mm)";
        series = prof.profiles.map((p, i) => ({
          x: mm, y: p.pen_mm, label: `${p.name} (${valid[i].t_pulse_s} s)`,
          color: Plot.PALETTE[i % Plot.PALETTE.length],
        }));
      }
      Plot.draw($("las-plot"), {
        xlog: true, ylog: true,
        xlabel: "range (Mm)", ylabel, y2label, series,
        vlines: [
          ...(L.graphMode === "damage" ? prof.profiles.map((p, i) => ({
            x: p.r_kill / 1e6, label: "kill " + fmtDist(p.r_kill),
            color: Plot.PALETTE[i % Plot.PALETTE.length],
          })) : []),
          L.clickR ? { x: L.clickR / 1e6, label: "▼ " + fmtDist(L.clickR), color: "#e8d05f" } : null,
        ].filter(Boolean),
        height: 300,
      });

      const rows = prof.profiles.map((p, i) => `
        <tr><td><span style="color:${Plot.PALETTE[i % Plot.PALETTE.length]}">■</span> ${esc(p.name)}</td>
          <td>${esc(valid[i].material)} @ ${valid[i].t_pulse_s} s ≥ ${valid[i].threshold_mm} mm</td>
          <td class="num">${fmtDist(p.r_kill)}</td>
          <td class="num">${fmtDist(p.r_open)}</td>
          <td class="num">${valid[i].chosen_range_m ? fmtDist(valid[i].chosen_range_m) : "—"}</td></tr>`).join("");
      $("las-table").innerHTML = `
        <table><tr><th>Profile</th><th>Kill criterion</th><th class="num">Kill range</th>
        <th class="num">Open fire (×${st.open_fire_factor})</th><th class="num">Chosen</th></tr>${rows}</table>`;

      if (L.clickR) {
        const idx = prof.range_m.reduce((best, r, i) =>
          Math.abs(r - L.clickR) < Math.abs(prof.range_m[best] - L.clickR) ? i : best, 0);
        const readout = prof.profiles.map(p => `${esc(p.name)}: ${p.pen_mm[idx].toPrecision(3)} mm · ` +
          `${fmtSI(p.pulse_energy_j, "J")} · ${fmtSI(p.fluence_j_m2[idx], "J/m²")}`).join("<br>");
        const opts = valid.map((p, i) => `<option value="${i}">${esc(p.name)}</option>`).join("");
        $("las-click").innerHTML = `<b>${fmtDist(L.clickR)}</b> · spot ${fmtDist(prof.spot_diameter_m[idx])} ·
          ${fmtSI(prof.beam_power_w, "W")} total · ${fmtSI(prof.irradiance_w_m2[idx], "W/m²")}<br>${readout}<br>
          <select id="las-assign-sel">${opts}</select>
          <button class="small" id="las-assign">Set as chosen range</button>`;
        $("las-assign").onclick = () => {
          const p = valid[+$("las-assign-sel").value];
          p.chosen_range_m = L.clickR;
          L.dirty = true;
          toast(`${p.name}: chosen range ${fmtDist(L.clickR)} — save profiles to keep it on the weapon.`, true);
          updateLaser();
        };
      } else {
        $("las-click").textContent = "Click the graph to inspect a range and save it to a profile.";
      }
    } else {
      $("las-table").innerHTML = `<p class="note">Add at least one profile.</p>`;
    }

    const sh = power.shot, su = power.sustain;
    const endurance = L.deployed ? su?.endurance_deployed_s : su?.endurance_retracted_s;
    $("las-power").innerHTML = `
      <div class="r"><span class="k">Beam</span><span class="v accent">${fmtSI(L.p_gw*1e9,"W")}</span>
        <span class="u">${L.ap} m aperture · ${fmtSI(L.lambda_nm*1e-9,"m")}</span></div>
      <div class="r"><span class="k">Wall-plug</span><span class="v">${(L.wall*100).toFixed(1)}%</span>
        <span class="u">${fmtSI(L.p_gw*1e9/L.wall,"W")} electrical while firing</span></div>
      <div class="r"><span class="k">Beam / shot</span><span class="v">${fmtMJ(sh.beam_mj)}</span></div>
      <div class="r"><span class="k">Electrical / shot</span><span class="v">${fmtMJ(sh.electrical_mj)}</span></div>
      <div class="r"><span class="k">Waste heat / shot</span><span class="v warn">${fmtMJ(sh.waste_mj)}</span></div>
      <div class="r"><span class="k">Shots / bank</span><span class="v accent">${sh.shots_per_bank ? fmtSI(sh.shots_per_bank, "") : "—"}</span></div>
      <div class="r"><span class="k">Continuous waste</span><span class="v">${fmtSI(su.waste_w, "W")}</span></div>
      <div class="r"><span class="k">Sustained fire until sink full</span>
        <span class="v ${endurance == null ? "good" : ""}">${endurance == null ? "indefinite" : fmtDur(endurance)}</span>
        <span class="u">low loop ${L.deployed ? "deployed" : "retracted"}</span></div>`;
  });
}

/* ========================= LIDAR & POINT DEFENSE ======================= */

const LP_MISSING_META = new Set();
const lpIn = (what, effect, unit, nature = "user assumption") =>
  `${what} This is an entered ${nature}${unit ? `, displayed in ${unit}` : ""}. ${effect}`;
const lpOut = (what, effect, unit, nature = "calculated result") =>
  `${what} This is a ${nature}${unit ? `, displayed in ${unit}` : ""}. ${effect}`;

// Single source of truth for every value rendered by the Lidar & PD page.
const LP_META = Object.freeze({
  scenario_preset: lpIn("Loads a complete starting scenario.", "Changing it replaces the current draft but does not affect fleet data.", "named presets", "preset choice"),
  detector_preset: lpIn("Loads a lidar transmitter and receiver configuration.", "A larger receiver usually collects more target and jammer light.", "named presets", "preset choice"),
  target_preset: lpIn("Loads target size, reflectivity, and damage assumptions.", "Smaller or darker targets are generally harder to track.", "named presets", "preset choice"),
  weapon_preset: lpIn("Loads a built-in or installed point-defense laser.", "More power or aperture generally shortens kill time.", "named presets", "preset choice"),
  geometry_mode: lpIn("Chooses simple range-and-offset labels or full coordinates.", "Both modes edit the same three-dimensional vectors.", "mode", "display choice"),
  "scenario_name": lpIn("A human-readable name carried into exported scenarios and results.", "It does not change the calculation.", "text"),
  "detector.wavelength_m": lpIn("The lidar light wavelength.", "Shorter wavelengths make smaller diffraction spots but may change target reflectivity.", "nanometres"),
  "detector.transmitter_power_w": lpIn("Average optical power sent by the lidar during the sample.", "More power normally produces more returned target photons.", "megawatts"),
  "detector.transmitter_aperture_m": lpIn("The effective diameter of the lidar transmitter.", "A larger aperture makes a tighter illumination spot.", "metres"),
  "detector.transmitter_m2": lpIn("How much wider the lidar beam is than an ideal diffraction-limited beam.", "Values above one spread the beam and weaken illumination.", "dimensionless engineering factor"),
  "detector.receiver_aperture_m": lpIn("The effective diameter of the receiving mirror.", "A larger receiver collects more target and countermeasure light and improves angular resolution.", "metres"),
  "detector.integration_time_s": lpIn("How long the detector gathers light for this snapshot.", "Longer samples collect more photons but may blur changing geometry.", "milliseconds"),
  "detector.optical_throughput": lpIn("The fraction of collected light surviving the receiver optics.", "Higher throughput increases every optical signal reaching the detector.", "percent"),
  "detector.quantum_efficiency": lpIn("The fraction of arriving photons converted into detector counts.", "Higher efficiency increases signal and photon noise together.", "percent"),
  "detector.filter_center_m": lpIn("The wavelength centered in the receiver filter.", "A jammer far from this wavelength is strongly rejected.", "nanometres"),
  "detector.filter_fwhm_m": lpIn("The receiver filter's effective spectral width.", "A narrower filter rejects more off-wavelength jammer light.", "picometres"),
  "detector.gate_width_s": lpIn("How long the direct detector is open for each receive gate.", "Short gates reduce temporal overlap with untimed jammers.", "nanoseconds"),
  "detector.pulse_repetition_hz": lpIn("How often receive gates occur.", "It documents the sampling cadence used to judge temporal overlap.", "hertz"),
  "detector.pixel_scale_rad": lpIn("The sky angle represented by one detector pixel.", "Larger pixels admit sources farther from the expected target location.", "nanoradians"),
  "detector.tracking_window_radius_rad": lpIn("The angular radius kept by centroid processing.", "A larger window retains offset targets but admits more false sources.", "microradians"),
  "detector.detector_floor_sigma_rad": lpIn("Residual measurement error not caused by photon statistics.", "A larger floor prevents added photons from improving accuracy beyond this limit.", "nanoradians", "engineering assumption"),
  "detector.background_photons": lpIn("Expected accepted sky and optical background counts.", "More background lowers target signal-to-noise.", "photons per sample"),
  "detector.dark_counts": lpIn("Detector counts produced without incoming light.", "More dark counts lower signal-to-noise.", "counts per sample"),
  "detector.read_noise_e": lpIn("Electronic readout noise for the sample.", "More read noise makes weak returns harder to distinguish.", "electron-equivalent RMS"),
  "detector.full_well_e": lpIn("Maximum detector charge before saturation.", "A larger well tolerates brighter pre-processing jammer light.", "electron-equivalent counts"),
  "detector.recovery_time_s": lpIn("Assumed time needed after saturation.", "Longer recovery adds more delay to a saturated engagement.", "milliseconds", "engineering assumption"),
  "detector.track_snr_min": lpIn("Lowest signal-to-noise that retains a track.", "Raising it makes track dropout more likely.", "ratio", "doctrine threshold"),
  "detector.fire_control_snr_min": lpIn("Lowest signal-to-noise accepted for nominal weapon pointing.", "Raising it makes degraded fire-control status more likely.", "ratio", "doctrine threshold"),
  "detector.ambiguity_ratio_min": lpIn("False-source strength relative to the target that triggers ambiguity.", "A lower threshold makes deception easier to flag.", "ratio", "doctrine threshold"),
  "detector.speckle_single_sigma_rad": lpIn("Centroid jitter assumed for one fully correlated speckle view.", "A larger value raises the measurement floor unless diversity averages it down.", "nanoradians", "engineering assumption"),
  "detector.speckle_temporal_modes": lpIn("Number of genuinely independent time samples.", "More independent samples reduce residual speckle error.", "mode count", "engineering assumption"),
  "detector.speckle_wavelength_modes": lpIn("Number of independent wavelength samples.", "More wavelength diversity reduces residual speckle error.", "mode count", "engineering assumption"),
  "detector.speckle_polarization_modes": lpIn("Number of independent polarization views.", "More polarization diversity reduces residual speckle error.", "mode count", "engineering assumption"),
  "detector.speckle_view_modes": lpIn("Number of independent viewing geometries.", "More viewpoints reduce residual speckle error.", "mode count", "engineering assumption"),
  "target.position_m": lpIn("Target position relative to the defending sensor.", "Greater range weakens the return and increases causal delay and weapon spot size.", "megametres"),
  "target.projected_area_m2": lpIn("Target area presented to the lidar.", "A larger area intercepts more illumination.", "square metres", "target assumption"),
  "target.characteristic_diameter_m": lpIn("Target size used for angular extent and speckle scale.", "A larger apparent target can increase centroid spread.", "metres", "target assumption"),
  "target.body_radius_m": lpIn("Radius used when asking whether the weapon lands anywhere on the body.", "A larger body is easier to capture.", "metres", "target assumption"),
  "target.vulnerable_patch_radius_m": lpIn("Radius of the specific damage patch the laser must reach.", "A smaller patch makes an effective kill less likely.", "metres", "target assumption"),
  "target.uv_reflectivity": lpIn("Effective diffuse reflectivity at the lidar wavelength.", "A darker target returns fewer photons.", "percent", "target assumption"),
  "target.aspect_factor": lpIn("How target orientation changes presented area and return.", "A lower value represents an unfavorable aspect and weaker return.", "fraction", "target assumption"),
  "target.closure_velocity_m_s": lpIn("Positive speed at which range is shrinking.", "Faster closure leaves less time before warhead standoff.", "kilometres per second"),
  "target.warhead_standoff_m": lpIn("Range by which point defense should have produced the effect.", "A larger standoff leaves less engagement time.", "kilometres", "engagement assumption"),
  "target.soft_kill_fluence_j_m2": lpIn("Energy per area needed to disable a vulnerable sensor or subsystem.", "A higher requirement lengthens soft-kill dwell time.", "joules per square metre", "damage assumption"),
  "target.structural_kill_fluence_j_m2": lpIn("Energy per area needed for structural defeat.", "A higher requirement lengthens structural kill time.", "joules per square metre", "damage assumption"),
  "fire_control.processing_latency_s": lpIn("Delay in processing and beam control beyond light travel.", "More latency gives target motion longer to spoil the aim point.", "milliseconds"),
  "fire_control.position_sigma_m": lpIn("Current one-axis position uncertainty before this lidar update.", "More uncertainty increases the future aim bubble.", "metres", "track assumption"),
  "fire_control.velocity_sigma_m_s": lpIn("Residual one-axis velocity uncertainty.", "More velocity uncertainty grows into position error during delay.", "metres per second", "track assumption"),
  "fire_control.acceleration_sigma_m_s2": lpIn("Unpredictable target acceleration left after filtering.", "More maneuver uncertainty expands the future aim bubble.", "metres per second squared", "maneuver assumption"),
  "fire_control.maneuver_persistence_s": lpIn("How long one unpredictable acceleration command tends to persist.", "It selects the resolved or unresolved maneuver approximation.", "milliseconds", "maneuver assumption"),
  "fire_control.boresight_sigma_rad": lpIn("Random alignment error between sensor and weapon.", "More misalignment increases miss distance.", "nanoradians", "alignment assumption"),
  "fire_control.platform_sigma_rad": lpIn("Random pointing motion of the defending platform.", "More platform jitter increases miss distance.", "nanoradians", "platform assumption"),
  "fire_control.beam_sigma_rad": lpIn("Random outgoing beam-placement error.", "More beam error reduces capture probability.", "nanoradians", "weapon assumption"),
  "fire_control.reacquisition_time_s": lpIn("Extra delay after drop, deception, or saturation.", "Longer reacquisition directly increases effective kill time.", "milliseconds", "doctrine assumption"),
  "fire_control.minimum_capture_probability": lpIn("Minimum body-hit probability accepted as usable fire control.", "Raising it makes degraded status more likely.", "percent", "doctrine threshold"),
  "weapon.wavelength_m": lpIn("Point-defense laser wavelength.", "Shorter wavelengths create smaller diffraction spots at the same aperture.", "nanometres"),
  "weapon.aperture_m": lpIn("Effective diameter of the firing aperture.", "A larger aperture tightens the weapon spot and raises flux.", "metres"),
  "weapon.m2": lpIn("How much wider the weapon beam is than an ideal beam.", "Values above one enlarge the spot and increase dwell time.", "dimensionless engineering factor"),
  "weapon.listed_optical_power_w": lpIn("Total optical output before the central-lobe fraction.", "More power raises target-plane flux.", "megawatts"),
  "weapon.central_lobe_fraction": lpIn("Fraction of weapon power in the useful central spot.", "A higher fraction increases useful flux.", "percent", "optical assumption"),
  "weapon.duty_cycle": lpIn("Fraction of time the weapon can deliver its listed power.", "A lower duty cycle lengthens average dwell time.", "percent", "operating assumption"),
  "weapon.slew_time_s": lpIn("Delay to place the weapon on this target.", "More slew time consumes the engagement window.", "milliseconds", "service assumption"),
  "jammers[].id": lpIn("Stable label for this jammer.", "It changes only identification in audit output.", "text"),
  "jammers[].enabled": lpIn("Whether this jammer contributes to the calculation.", "Disabled entries remain visible but contribute zero light.", "on or off"),
  "jammers[].mode": lpIn("Noise adds variance; false source can also pull the measured center of light.", "False-source mode can create systematic aim bias.", "mode", "model choice"),
  "jammers[].position_m": lpIn("Jammer position relative to the defending receiver.", "Angular separation from the target controls how much enters the tracking cell.", "megametres"),
  "jammers[].optical_power_w": lpIn("Total optical jammer output.", "More power can saturate the detector or lower signal-to-noise.", "watts"),
  "jammers[].aperture_m": lpIn("Jammer transmit aperture diameter.", "A larger aperture delivers a tighter, brighter beam to the defender.", "metres"),
  "jammers[].m2": lpIn("Jammer beam quality relative to ideal diffraction.", "Higher values spread the jammer beam and reduce delivered brightness.", "dimensionless engineering factor"),
  "jammers[].wavelength_m": lpIn("Center wavelength of jammer light.", "Mismatch with the receiver filter reduces accepted light.", "nanometres"),
  "jammers[].spectral_fwhm_m": lpIn("Spectral width of jammer light.", "A broader line may overlap more of the receiver filter but spreads power spectrally.", "picometres"),
  "jammers[].pointing_error_rad": lpIn("Jammer aiming error relative to the defender.", "More error reduces delivered jammer power.", "nanoradians"),
  "jammers[].polarization_overlap": lpIn("Fraction matching the receiver polarization path.", "Lower overlap rejects more jammer light.", "percent"),
  "jammers[].temporal_overlap": lpIn("Fraction arriving while the detector is open.", "Lower overlap reduces pre-processing contamination.", "percent"),
  "jammers[].code_correlation": lpIn("Fraction surviving coded or coherent processing.", "Lower correlation reduces post-processing noise but cannot undo saturation.", "percent"),
  "jammers[].range_response": lpIn("Fraction surviving range or Doppler filtering.", "Lower response reduces post-processing contamination.", "percent"),
  "jammers[].central_lobe_fraction": lpIn("Fraction of jammer power in its modeled central beam.", "A higher fraction delivers more useful jammer light.", "percent", "optical assumption"),
  "chaff[].id": lpIn("Stable label for this cloud.", "It changes only identification in audit output.", "text"),
  "chaff[].enabled": lpIn("Whether this cloud contributes to the calculation.", "Disabled clouds remain visible but contribute zero.", "on or off"),
  "chaff[].position_m": lpIn("Cloud center relative to the defending receiver.", "Its offset decides whether the lidar beam crosses it.", "megametres"),
  "chaff[].width_m": lpIn("Initial transverse cloud width.", "A wider cloud overlaps more of the beam.", "metres", "cloud assumption"),
  "chaff[].height_m": lpIn("Initial transverse cloud height.", "A taller cloud overlaps more of the beam.", "metres", "cloud assumption"),
  "chaff[].depth_m": lpIn("Initial line-of-sight cloud depth.", "It is reported as the cloud expands; optical depth controls extinction directly.", "metres", "cloud assumption"),
  "chaff[].age_s": lpIn("Time since the cloud was deployed.", "Older clouds are larger and have lower conserved optical depth.", "seconds"),
  "chaff[].expansion_speed_m_s": lpIn("Symmetric expansion speed on each cloud dimension.", "Faster expansion spreads the cloud and lowers optical depth.", "metres per second", "cloud assumption"),
  "chaff[].optical_depth": lpIn("Initial strength of extinction through the cloud.", "Higher optical depth blocks more target return and produces more scatter.", "dimensionless", "cloud assumption"),
  "chaff[].single_scatter_albedo": lpIn("Fraction of removed light scattered instead of absorbed.", "Higher albedo creates more structured chaff return.", "percent", "scattering assumption"),
  "chaff[].backscatter_fraction": lpIn("Fraction of scattered light assigned to the receiver-facing approximation.", "Higher values make the cloud brighter to the lidar.", "percent", "engineering approximation"),
  "chaff[].range_response": lpIn("Fraction of chaff return surviving range processing.", "Lower response reduces structured contamination.", "percent"),
  "chaff[].clearance_fluence_j_m2": lpIn("Energy per area assumed necessary to clear the cloud.", "A higher requirement increases burn-through time.", "joules per square metre", "engineering assumption"),

  "summary.detector_state": lpOut("Overall detector and track condition after applying state precedence.", "Saturated, dropped, or ambiguous states add reacquisition delay.", "status"),
  "summary.fire_control_usable": lpOut("Whether SNR and capture meet the configured firing thresholds.", "False means the track is retained only as degraded or has failed.", "yes or no"),
  "summary.target_photons": lpOut("Expected detected target photons after chaff attenuation.", "More target photons normally improve SNR and centroid accuracy.", "photons"),
  "summary.jammer_to_signal": lpOut("Accepted post-processing jammer photons divided by target photons.", "Higher values generally worsen measurement quality.", "ratio"),
  "summary.snr": lpOut("Target signal compared with combined photon and read noise.", "Higher values make tracking and fire control more reliable.", "ratio"),
  "summary.measurement_r95_m": lpOut("Radius containing 95% of random current measurement error.", "A smaller bubble gives a more precise present target location.", "metres"),
  "summary.centroid_bias_m": lpOut("Systematic aim offset caused by structured false light.", "Unlike random error, this shifts the center of the solution.", "metres", "calculated engineering approximation"),
  "summary.future_aim_r95_m": lpOut("Bias-inclusive 95% aim radius after causal delay.", "A larger future bubble lowers body and patch capture.", "metres", "calculated engineering approximation"),
  "summary.body_capture_probability": lpOut("Chance the beam lands somewhere on the target body.", "Higher is better for acquiring any target surface.", "percent", "calculated Gaussian approximation"),
  "summary.patch_capture_probability": lpOut("Conditional chance of reaching the vulnerable patch once the body is captured.", "Higher is better for producing the intended damage.", "percent", "calculated Gaussian approximation"),
  "summary.structural_kill_time_s": lpOut("Structural dwell time after capture losses and reacquisition delay.", "Shorter time improves snapshot feasibility.", "seconds", "calculated snapshot estimate"),
  "summary.time_to_standoff_s": lpOut("Time until the closing target reaches warhead standoff.", "More time provides a larger engagement window.", "seconds", "calculated constant-speed estimate"),
  "summary.structural_kill_feasible": lpOut("Whether effective structural dwell plus slew fits before standoff.", "This is a snapshot verdict, not a time-stepped engagement simulation.", "yes or no", "calculated snapshot estimate"),
});

// Detailed output explanations reuse summary wording where the concepts match.
const LP_DETAIL_META = Object.freeze({
  "geometry.target_range_m": lpOut("Straight-line range from defender to target.", "Longer range weakens lidar return and weapon flux.", "metres"),
  "geometry.target_line_of_sight": lpOut("Normalized direction from defender to target.", "It defines the tangent plane used for angular offsets.", "unit vector"),
  "geometry.receiver_area_m2": lpOut("Collecting area of the receiver aperture.", "More area gathers more optical power.", "square metres"),
  "geometry.receiver_diffraction_rad": lpOut("Ideal angular scale set by wavelength and receiver aperture.", "A smaller scale supports finer centroiding.", "radians"),
  "geometry.angular_acceptance_rad": lpOut("Combined angular region admitted by diffraction, pixels, and tracking window.", "A wider region admits more offset jammer light.", "radians", "calculated engineering approximation"),
  "signal.lidar_spot_diameter_m": lpOut("Effective central lidar spot diameter at the target.", "A smaller spot concentrates illumination.", "metres"),
  "signal.lidar_spot_area_m2": lpOut("Area of the effective lidar spot.", "A larger area dilutes transmitter power.", "square metres"),
  "signal.target_effective_area_m2": lpOut("Projected target area after aspect adjustment.", "A larger value intercepts more lidar power.", "square metres"),
  "signal.intercepted_power_w": lpOut("Transmitter power intercepted by the target.", "More intercepted power produces a brighter return.", "watts"),
  "signal.target_received_power_clean_w": lpOut("Target return reaching the receiver before chaff.", "This is the clean reference for countermeasure comparisons.", "watts"),
  "signal.target_received_power_actual_w": lpOut("Target return reaching the receiver after chaff.", "Lower power means fewer detected target photons.", "watts"),
  "signal.photon_energy_j": lpOut("Energy carried by one lidar photon.", "It converts received optical energy into expected photon count.", "joules"),
  "signal.target_photons_clean": lpOut("Expected target photons without chaff attenuation.", "This is the clean photon reference.", "photons"),
  "signal.target_photons_actual": LP_META["summary.target_photons"],
  "signal.target_transmittance": lpOut("Fraction of target return left after all enabled chaff clouds.", "Lower values mean stronger obscuration.", "fraction", "calculated cloud approximation"),
  "detector.state": LP_META["summary.detector_state"],
  "detector.primary_cause": lpOut("First condition responsible for the detector state.", "It identifies the dominant immediate limitation.", "text"),
  "detector.causes": lpOut("All thresholds contributing to the current state.", "Multiple causes can be active at once.", "list"),
  "detector.total_pre_processing_counts": lpOut("All detector counts before code and range rejection.", "This value controls physical saturation.", "counts"),
  "detector.full_well_utilization": lpOut("Pre-processing counts divided by detector capacity.", "Values above one are saturated.", "ratio"),
});

// Object.freeze above prevents accidental mutation; build the complete lookup by spread.
const LP_ALL_META = Object.freeze({ ...LP_META, ...LP_DETAIL_META,
  "detector.target_photons_clean": LP_DETAIL_META["signal.target_photons_clean"],
  "detector.target_photons_actual": LP_META["summary.target_photons"],
  "detector.background_photons": lpOut("Accepted background and dark counts.", "More background lowers SNR.", "counts"),
  "detector.noise_jammer_photons": lpOut("Post-processing photons from noise-mode jammers.", "More noise jammer light lowers SNR without directly shifting the centroid.", "photons"),
  "detector.structured_photons": lpOut("Accepted chaff and false-source photons.", "These photons add noise and can shift the measured center.", "photons"),
  "detector.read_noise_variance": lpOut("Read-noise contribution after squaring RMS noise.", "More variance lowers SNR.", "counts squared"),
  "detector.snr": LP_META["summary.snr"],
  "detector.photon_centroid_sigma_clean_rad": lpOut("Centroid error from clean target photon statistics.", "Smaller values mean more precise angular measurement.", "radians"),
  "detector.photon_centroid_sigma_actual_rad": lpOut("Photon-statistical centroid error with countermeasure and background noise.", "More accepted contaminating light increases it.", "radians"),
  "detector.speckle_angular_scale_rad": lpOut("Angular scale of coherent target speckle.", "It sets the raw correlation size before diversity averaging.", "radians", "calculated approximation"),
  "detector.speckle_cell_m": lpOut("Speckle correlation scale projected at the receiver plane.", "A smaller cell allows more spatial diversity across the aperture.", "metres", "calculated approximation"),
  "detector.speckle_spatial_modes": lpOut("Estimated independent speckle cells across the receiver.", "More cells reduce residual speckle error.", "mode count", "calculated approximation"),
  "detector.speckle_total_modes": lpOut("Combined spatial, time, wavelength, polarization, and view diversity.", "More independent modes reduce speckle error.", "mode count", "calculated approximation"),
  "detector.speckle_residual_sigma_rad": lpOut("Speckle centroid error remaining after diversity.", "A larger residual expands the measurement bubble.", "radians", "calculated approximation"),
  "detector.measurement_sigma_rad": lpOut("Combined one-axis photon, speckle, and detector-floor error.", "It feeds the current and future aim bubbles.", "radians"),
  "detector.centroid_bias_vector_rad": lpOut("Two-dimensional angular shift from structured false light.", "Its direction identifies where the measured center is pulled.", "radians", "calculated approximation"),
  "detector.centroid_bias_magnitude_rad": lpOut("Magnitude of structured angular bias.", "A larger bias moves the aim point farther from the target center.", "radians", "calculated approximation"),
  "detector.random_r50_m": lpOut("Radius containing 50% of random measurement error.", "Smaller is more precise.", "metres"),
  "detector.random_r90_m": lpOut("Radius containing 90% of random measurement error.", "Smaller is more precise.", "metres"),
  "detector.random_r95_m": LP_META["summary.measurement_r95_m"],
  "detector.bias_inclusive_r95_m": lpOut("Random 95% radius plus structured bias distance.", "It is a conservative current-location bubble.", "metres", "calculated approximation"),
  "detector.projected_recovery_time_s": lpOut("Configured recovery delay reported when currently saturated.", "V1 reports it but does not simulate the later recovering state.", "seconds", "projected assumption"),
  "fire_control.causal_delay_s": lpOut("Round-trip light time plus processing latency.", "Longer delay gives motion uncertainty more time to grow.", "seconds"),
  "fire_control.maneuver_regime": lpOut("Whether maneuver persistence is resolved or unresolved over the delay.", "It selects the corresponding uncertainty approximation.", "state", "calculated model choice"),
  "fire_control.position_contribution_m": lpOut("Present position uncertainty carried into the future solution.", "More uncertainty enlarges the aim bubble.", "metres"),
  "fire_control.velocity_contribution_m": lpOut("Velocity uncertainty grown over causal delay.", "More delay or velocity error enlarges it.", "metres"),
  "fire_control.maneuver_contribution_m": lpOut("Position uncertainty from unpredictable acceleration.", "More maneuver severity enlarges the aim bubble.", "metres", "calculated approximation"),
  "fire_control.measurement_contribution_m": lpOut("Angular measurement uncertainty converted to target-plane distance.", "It grows linearly with range.", "metres"),
  "fire_control.boresight_contribution_m": lpOut("Sensor-to-weapon alignment uncertainty at target range.", "More alignment error enlarges the aim bubble.", "metres"),
  "fire_control.platform_contribution_m": lpOut("Platform pointing uncertainty at target range.", "More jitter enlarges the aim bubble.", "metres"),
  "fire_control.beam_contribution_m": lpOut("Outgoing beam-placement uncertainty at target range.", "More beam error enlarges the aim bubble.", "metres"),
  "fire_control.random_aim_sigma_m": lpOut("Root-sum-square random future aim error.", "Smaller values improve capture probability.", "metres"),
  "fire_control.systematic_bias_m": LP_META["summary.centroid_bias_m"],
  "fire_control.equivalent_aim_sigma_m": lpOut("Random error plus bias folded into an equivalent circular error.", "This scalar drives the simplified capture model.", "metres", "calculated approximation"),
  "fire_control.future_r95_m": LP_META["summary.future_aim_r95_m"],
  "fire_control.body_capture_probability": LP_META["summary.body_capture_probability"],
  "fire_control.patch_capture_probability": LP_META["summary.patch_capture_probability"],
  "fire_control.fire_control_usable": LP_META["summary.fire_control_usable"],
  "point_defense.weapon_spot_diameter_m": lpOut("Effective weapon central-spot diameter at current range.", "A smaller spot produces higher flux.", "metres"),
  "point_defense.useful_central_lobe_power_w": lpOut("Weapon power assigned to the useful central spot.", "More useful power shortens clean dwell time.", "watts"),
  "point_defense.average_flux_w_m2": lpOut("Duty-averaged useful power per target area.", "Higher flux shortens clean kill time.", "watts per square metre"),
  "point_defense.clean_soft_kill_time_s": lpOut("Dwell required for soft kill with perfect placement.", "It excludes capture losses and reacquisition.", "seconds", "calculated snapshot estimate"),
  "point_defense.clean_structural_kill_time_s": lpOut("Dwell required for structural kill with perfect placement.", "It excludes capture losses and reacquisition.", "seconds", "calculated snapshot estimate"),
  "point_defense.body_capture_factor": LP_META["summary.body_capture_probability"],
  "point_defense.patch_capture_factor": LP_META["summary.patch_capture_probability"],
  "point_defense.applied_reacquisition_delay_s": lpOut("Delay added because the detector is saturated, dropped, or ambiguous.", "Tracked and degraded states add no reacquisition delay.", "seconds"),
  "point_defense.effective_soft_kill_time_s": lpOut("Soft-kill dwell after capture losses and reacquisition.", "Shorter time improves feasibility.", "seconds", "calculated snapshot estimate"),
  "point_defense.effective_structural_kill_time_s": LP_META["summary.structural_kill_time_s"],
  "point_defense.time_to_standoff_s": LP_META["summary.time_to_standoff_s"],
  "point_defense.soft_kill_feasible": lpOut("Whether effective soft-kill time plus slew fits before standoff.", "It is a constant-range snapshot verdict.", "yes or no", "calculated snapshot estimate"),
  "point_defense.structural_kill_feasible": LP_META["summary.structural_kill_feasible"],
  "point_defense.model_limitation": lpOut("Plain statement of what the point-defense result does not simulate.", "Use it when interpreting the snapshot verdict.", "text"),
  "jammers[].id": LP_META["jammers[].id"], "jammers[].enabled": LP_META["jammers[].enabled"],
  "jammers[].mode": LP_META["jammers[].mode"],
  "jammers[].source_range_m": lpOut("Range from jammer to defender.", "It controls jammer footprint at the receiver.", "metres"),
  "jammers[].angular_separation_rad": lpOut("Angle between jammer and target as seen by the defender.", "Larger separation reduces tracking-cell overlap.", "radians"),
  "jammers[].tangent_offset_rad": lpOut("Two-axis jammer offset in the target tangent plane.", "False sources use this direction to pull the centroid.", "radians"),
  "jammers[].divergence_rad": lpOut("Effective angular spread of the jammer beam.", "More divergence spreads power over a wider footprint.", "radians"),
  "jammers[].footprint_diameter_m": lpOut("Jammer central-lobe diameter at the defender.", "A larger footprint dilutes received power.", "metres"),
  "jammers[].pointing_weight": lpOut("Fraction left after jammer pointing error.", "One is perfect pointing; lower values deliver less light.", "fraction", "Gaussian approximation"),
  "jammers[].angular_overlap_weight": lpOut("Fraction admitted because of angular collinearity.", "One is aligned with the target; lower values mean stronger rejection.", "fraction", "Gaussian approximation"),
  "jammers[].spectral_overlap_weight": lpOut("Fraction admitted by spectral overlap.", "One is a close filter match; lower values mean stronger rejection.", "fraction", "Gaussian approximation"),
  "jammers[].polarization_overlap": LP_META["jammers[].polarization_overlap"],
  "jammers[].temporal_overlap": LP_META["jammers[].temporal_overlap"],
  "jammers[].code_correlation": LP_META["jammers[].code_correlation"],
  "jammers[].range_response": LP_META["jammers[].range_response"],
  "jammers[].pre_processing_photons": lpOut("Jammer photons reaching the detector before digital rejection.", "These counts can saturate the detector.", "photons"),
  "jammers[].post_processing_photons": lpOut("Jammer photons left after code and range rejection.", "These counts affect SNR and structured bias.", "photons"),
  "jammers[].jammer_to_signal": lpOut("This jammer's accepted photons divided by target photons.", "Higher values mean stronger contamination.", "ratio"),
  "jammers[].inside_target_cell": lpOut("Whether angular separation falls inside the configured acceptance radius.", "Outside sources may still have a small Gaussian overlap.", "yes or no"),
  "jammers[].warnings": lpOut("Plain-English cautions specific to this jammer.", "They identify rejection or modeling conditions worth reviewing.", "list"),
  "chaff[].id": LP_META["chaff[].id"], "chaff[].enabled": LP_META["chaff[].enabled"],
  "chaff[].current_dimensions_m": lpOut("Cloud width, height, and depth after expansion.", "Larger dimensions spread the original optical depth over more area.", "metres", "calculated cloud approximation"),
  "chaff[].current_optical_depth": lpOut("Extinction strength after expansion.", "Higher depth blocks and scatters more lidar light.", "dimensionless", "calculated cloud approximation"),
  "chaff[].cloud_range_m": lpOut("Range from defender to cloud center.", "It sets lidar and weapon spot size at the cloud.", "metres"),
  "chaff[].angular_offset_rad": lpOut("Cloud-center angle from the target line of sight.", "Larger offsets reduce beam overlap.", "radians"),
  "chaff[].tangent_offset_rad": lpOut("Two-axis cloud offset in the target tangent plane.", "It sets the direction of structured centroid pull.", "radians"),
  "chaff[].lidar_beam_overlap": lpOut("Fraction of the lidar spot covered by the rectangular cloud.", "More overlap strengthens attenuation and chaff return.", "fraction", "fixed-grid approximation"),
  "chaff[].target_transmittance": lpOut("Fraction of target return surviving this cloud.", "Lower values mean stronger obscuration.", "fraction", "calculated cloud approximation"),
  "chaff[].accepted_chaff_photons": lpOut("Chaff-scattered photons accepted by the receiver.", "More photons add structured noise and centroid bias.", "photons", "calculated scattering approximation"),
  "chaff[].centroid_offset_rad": lpOut("Angular location assigned to this cloud's structured return.", "It determines the direction of centroid bias.", "radians", "calculated approximation"),
  "chaff[].lidar_clearance_time_s": lpOut("Estimated lidar illumination time needed to clear the cloud.", "Longer time means the lidar is less useful for burn-through.", "seconds", "engineering approximation"),
  "chaff[].weapon_clearance_time_s": lpOut("Estimated weapon illumination time needed to clear the cloud.", "Longer time consumes more of the engagement window.", "seconds", "engineering approximation"),
  "chaff[].warnings": lpOut("Plain-English cautions specific to this cloud.", "They identify geometry or modeling conditions worth reviewing.", "list"),
});

function lpPathKey(path) {
  return path.replace(/\[\d+\]/g, "[]");
}
function lpMeta(path) {
  const key = lpPathKey(path);
  const value = LP_ALL_META[key] || LP_ALL_META[key.replace(/\[\]$/, "")];
  if (!value) {
    LP_MISSING_META.add(key);
    console.error("Missing Lidar & PD tooltip metadata:", key);
    return "This value is missing its required plain-English explanation. Please report this coverage defect.";
  }
  return value;
}
function lpTip(path) {
  const id = "lp-tip-" + path.replace(/[^a-z0-9]+/gi, "-");
  return `<span class="lp-help-wrap"><button type="button" class="lp-help" data-lp-help
    aria-label="Explain ${esc(path)}" aria-expanded="false" aria-describedby="${id}">?</button>
    <span class="lp-tooltip" role="tooltip" id="${id}">${esc(lpMeta(path))}</span></span>`;
}
function lpTokens(path) { return path.replace(/\[(\d+)\]/g, ".$1").split("."); }
function lpGet(root, path) { return lpTokens(path).reduce((v, k) => v?.[k], root); }
function lpSet(root, path, value) {
  const keys = lpTokens(path); let at = root;
  keys.slice(0, -1).forEach(k => { at = at[k]; });
  at[keys[keys.length - 1]] = value;
}
const lpClone = value => JSON.parse(JSON.stringify(value));

function lpDefaultScenario() {
  return {
    schema_version: "1.0", scenario_name: "10,000 km aligned jammer stress case",
    detector: { wavelength_m: 266e-9, transmitter_power_w: 1e6, transmitter_aperture_m: 1,
      transmitter_m2: 1, receiver_aperture_m: 1, integration_time_s: .001,
      optical_throughput: .526, quantum_efficiency: .95, filter_center_m: 266e-9,
      filter_fwhm_m: 1e-11, gate_width_s: 7e-9, pulse_repetition_hz: 10000,
      pixel_scale_rad: 5e-8, tracking_window_radius_rad: 1e-6,
      detector_floor_sigma_rad: 1e-9, background_photons: 10, dark_counts: .1,
      read_noise_e: 1, full_well_e: 1e8, recovery_time_s: .002,
      track_snr_min: 5, fire_control_snr_min: 20, ambiguity_ratio_min: 1,
      speckle_single_sigma_rad: 1e-9, speckle_temporal_modes: 1,
      speckle_wavelength_modes: 1, speckle_polarization_modes: 1, speckle_view_modes: 1 },
    target: { position_m: [1e7, 0, 0], projected_area_m2: 1, characteristic_diameter_m: 1,
      body_radius_m: .5, vulnerable_patch_radius_m: .05, uv_reflectivity: .1,
      aspect_factor: 1, closure_velocity_m_s: 130000, warhead_standoff_m: 300000,
      soft_kill_fluence_j_m2: 1e8, structural_kill_fluence_j_m2: 1e9 },
    fire_control: { processing_latency_s: .001, position_sigma_m: .01, velocity_sigma_m_s: .1,
      acceleration_sigma_m_s2: 100, maneuver_persistence_s: .02, boresight_sigma_rad: 4e-9,
      platform_sigma_rad: 4e-9, beam_sigma_rad: 3e-9, reacquisition_time_s: .05,
      minimum_capture_probability: .5 },
    weapon: { wavelength_m: 532e-9, aperture_m: 30, m2: 1,
      listed_optical_power_w: 1e9, central_lobe_fraction: .84, duty_cycle: .5, slew_time_s: .05 },
    jammers: [{ id: "J-1", enabled: true, mode: "noise", position_m: [1.2e7, 0, 0],
      optical_power_w: 100, aperture_m: .1, m2: 1, wavelength_m: 266e-9,
      spectral_fwhm_m: 5e-12, pointing_error_rad: 0, polarization_overlap: 1,
      temporal_overlap: 7e-5, code_correlation: .01, range_response: 1,
      central_lobe_fraction: .84 }],
    chaff: [], options: { include_disabled_entries: true, return_intermediates: true },
  };
}

function lpNewJammer(n) { return { id: "J-" + n, enabled: true, mode: "noise",
  position_m: [1.2e7, 0, 0], optical_power_w: 100, aperture_m: .1, m2: 1,
  wavelength_m: 266e-9, spectral_fwhm_m: 5e-12, pointing_error_rad: 0,
  polarization_overlap: 1, temporal_overlap: 7e-5, code_correlation: .01,
  range_response: 1, central_lobe_fraction: .84 }; }
function lpNewChaff(n) { return { id: "C-" + n, enabled: true, position_m: [9.9e6, 0, 0],
  width_m: 100, height_m: 100, depth_m: 100, age_s: 1, expansion_speed_m_s: 10,
  optical_depth: 1, single_scatter_albedo: .8, backscatter_fraction: .1,
  range_response: 1, clearance_fluence_j_m2: 1e7 }; }

const LP_DETECTORS = {
  dedicated: { label: "Dedicated 1 m lidar", values: { transmitter_aperture_m: 1, receiver_aperture_m: 1 } },
  segment: { label: "Stolen 1.65 m mirror segment", values: { transmitter_aperture_m: 1, receiver_aperture_m: 1.65 } },
  full: { label: "Full 30 m off-phase receiver", values: { transmitter_aperture_m: 1, receiver_aperture_m: 30 } },
};
const LP_TARGETS = {
  diffuse: { label: "1 m² diffuse missile", values: { projected_area_m2: 1, characteristic_diameter_m: 1,
    body_radius_m: .5, vulnerable_patch_radius_m: .05, uv_reflectivity: .1 } },
  dark: { label: "Dark rough missile", values: { projected_area_m2: 1, characteristic_diameter_m: 1,
    body_radius_m: .5, vulnerable_patch_radius_m: .05, uv_reflectivity: .04 }, speckle: 5e-9 },
  torplet: { label: "Torplet", values: { projected_area_m2: .25, characteristic_diameter_m: .5,
    body_radius_m: .25, vulnerable_patch_radius_m: .02, uv_reflectivity: .08 } },
};
const LP_WEAPONS = {
  bb_main: ["BB main full array", 30, 1e9], bb_task: ["BB main task group", 9.5, 1e8],
  bb_secondary: ["BB secondary", 10, 2e8], corvette: ["Corvette spinal", 15, 5e8],
  pdl: ["Standard PDL", 3, 1.5e8], drone: ["Drone PDL", 1, 4e7],
};

function lpState() {
  if (!UI.lidarPd) UI.lidarPd = { scenario: lpDefaultScenario(), result: null, stale: false,
    error: null, geometryMode: "simple", scenarioPreset: "stress", detectorPreset: "dedicated",
    targetPreset: "diffuse", weaponPreset: "bb_main" };
  return UI.lidarPd;
}
function lpField(path, label, scale = 1, unit = "", type = "number") {
  const s = lpState().scenario, value = lpGet(s, path), id = "lp-" + path.replace(/[^a-z0-9]+/gi, "-");
  const described = "lp-tip-" + path.replace(/[^a-z0-9]+/gi, "-");
  return `<label class="lp-field" for="${id}"><span>${esc(label)} ${lpTip(path)}</span>
    <span class="lp-control"><input id="${id}" type="${type}" step="any" data-lp-path="${path}"
      data-lp-scale="${scale}" aria-describedby="${described}" value="${esc(type === "number" ? value * scale : value)}">
      ${unit ? `<small>${esc(unit)}</small>` : ""}</span></label>`;
}
function lpCheck(path, label) {
  const value = lpGet(lpState().scenario, path), id = "lp-" + path.replace(/[^a-z0-9]+/gi, "-");
  return `<label class="lp-check" for="${id}"><input id="${id}" type="checkbox" data-lp-path="${path}"
    ${value ? "checked" : ""}> <span>${esc(label)} ${lpTip(path)}</span></label>`;
}
function lpSelect(path, label, options) {
  const value = lpGet(lpState().scenario, path), id = "lp-" + path.replace(/[^a-z0-9]+/gi, "-");
  return `<label class="lp-field" for="${id}"><span>${esc(label)} ${lpTip(path)}</span><select id="${id}"
    data-lp-path="${path}">${options.map(([v,l]) => `<option value="${esc(v)}" ${v === value ? "selected" : ""}>${esc(l)}</option>`).join("")}</select></label>`;
}
function lpSection(title, body, open = false) {
  return `<details class="panel lp-section" ${open ? "open" : ""}><summary><h2>${esc(title)}</h2></summary><div class="lp-fields">${body}</div></details>`;
}
function lpJammerCard(j, n) {
  const p = `jammers[${n}]`;
  return `<div class="lp-card"><div class="lp-card-head"><b>${esc(j.id || `Jammer ${n+1}`)}</b><span class="spacer"></span>
    <button class="small" data-lp-jup="${n}" ${n===0?"disabled":""}>↑</button><button class="small" data-lp-jdown="${n}" ${n===lpState().scenario.jammers.length-1?"disabled":""}>↓</button>
    <button class="small" data-lp-jcopy="${n}">duplicate</button><button class="small danger" data-lp-jdel="${n}">✕</button></div>
    <div class="lp-fields">${lpField(p+".id", "ID", 1, "", "text")}${lpCheck(p+".enabled", "Enabled")}
    ${lpSelect(p+".mode", "Mode", [["noise","noise"],["false_source","false source"]])}
    ${[0,1,2].map((_,k) => lpField(`${p}.position_m[${k}]`, (lpState().geometryMode === "simple" ? ["Range","Lateral Y","Lateral Z"] : ["X","Y","Z"])[k], 1e-6, "Mm")).join("")}
    ${lpField(p+".optical_power_w", "Optical power",1,"W")}${lpField(p+".aperture_m","Aperture",1,"m")}
    ${lpField(p+".m2","M²")}${lpField(p+".wavelength_m","Wavelength",1e9,"nm")}
    ${lpField(p+".spectral_fwhm_m","Spectral width",1e12,"pm")}${lpField(p+".pointing_error_rad","Pointing error",1e9,"nrad")}
    ${lpField(p+".polarization_overlap","Polarization",100,"%")}${lpField(p+".temporal_overlap","Temporal overlap",100,"%")}
    ${lpField(p+".code_correlation","Code correlation",100,"%")}${lpField(p+".range_response","Range response",100,"%")}
    ${lpField(p+".central_lobe_fraction","Central lobe",100,"%")}</div></div>`;
}
function lpChaffCard(c, n) {
  const p = `chaff[${n}]`;
  return `<div class="lp-card"><div class="lp-card-head"><b>${esc(c.id || `Chaff ${n+1}`)}</b><span class="spacer"></span>
    <button class="small" data-lp-cup="${n}" ${n===0?"disabled":""}>↑</button><button class="small" data-lp-cdown="${n}" ${n===lpState().scenario.chaff.length-1?"disabled":""}>↓</button>
    <button class="small" data-lp-ccopy="${n}">duplicate</button><button class="small danger" data-lp-cdel="${n}">✕</button></div>
    <div class="lp-fields">${lpField(p+".id","ID",1,"","text")}${lpCheck(p+".enabled","Enabled")}
    ${[0,1,2].map((_,k) => lpField(`${p}.position_m[${k}]`, (lpState().geometryMode === "simple" ? ["Range","Lateral Y","Lateral Z"] : ["X","Y","Z"])[k], 1e-6, "Mm")).join("")}
    ${lpField(p+".width_m","Initial width",1,"m")}${lpField(p+".height_m","Initial height",1,"m")}
    ${lpField(p+".depth_m","Initial depth",1,"m")}${lpField(p+".age_s","Age",1,"s")}
    ${lpField(p+".expansion_speed_m_s","Expansion",1,"m/s")}${lpField(p+".optical_depth","Optical depth")}
    ${lpField(p+".single_scatter_albedo","Scatter albedo",100,"%")}${lpField(p+".backscatter_fraction","Backscatter",100,"%")}
    ${lpField(p+".range_response","Range response",100,"%")}${lpField(p+".clearance_fluence_j_m2","Clearance fluence",1,"J/m²")}</div></div>`;
}

function lpPretty(path, value) {
  if (value == null) return "—";
  if (typeof value === "boolean") return value ? "Yes" : "No";
  if (Array.isArray(value)) return value.map(x => typeof x === "number" ? x.toPrecision(5) : x).join(", ");
  if (typeof value !== "number") return String(value).replaceAll("_", " ");
  if (path.endsWith("_w_m2")) return fmtSI(value,"W/m²");
  if (path.endsWith("_j_m2")) return fmtSI(value,"J/m²");
  if (path.endsWith("_m")) return fmtDist(value);
  if (path.endsWith("_s")) return fmtDur(value);
  if (path.endsWith("_w")) return fmtSI(value,"W");
  if (path.endsWith("_m2")) return fmtSI(value,"m²");
  if (path.endsWith("_rad")) return fmtSI(value,"rad");
  if (path.includes("probability") || path.includes("factor") || path.endsWith("transmittance") || path.includes("weight") || path.includes("overlap")) return (value*100).toPrecision(4)+"%";
  return Math.abs(value) >= 1e5 || (Math.abs(value) > 0 && Math.abs(value) < 1e-3) ? value.toExponential(4) : value.toPrecision(6);
}
function lpAuditRows(obj, prefix) {
  return Object.entries(obj).filter(([,v]) => !Array.isArray(v) || v.every(x => typeof x !== "object"))
    .map(([key,value]) => { const path = prefix+"."+key;
      return `<tr><th>${esc(key.replaceAll("_"," "))} ${lpTip(path)}</th><td>${esc(lpPretty(path,value))}</td>
        <td class="num">${typeof value === "number" ? esc(value.toPrecision(10)) : "—"}</td></tr>`; }).join("");
}
function lpAudit(title, obj, prefix, open = false) {
  return `<details class="panel lp-audit" ${open ? "open" : ""}><summary><h3>${esc(title)}</h3></summary>
    <table><thead><tr><th>Value</th><th>Friendly</th><th class="num">Raw SI</th></tr></thead><tbody>${lpAuditRows(obj,prefix)}</tbody></table></details>`;
}
function lpMetric(path, label, value, cls = "") {
  return `<div class="lp-metric ${cls}"><span>${esc(label)} ${lpTip(path)}</span><b>${esc(lpPretty(path,value))}</b></div>`;
}
function lpResults(st) {
  if (st.error) return `<div id="lp-validation" tabindex="-1" class="panel lp-error"><h2>Check the scenario</h2><p>${esc(st.error)}</p></div>`;
  if (!st.result) return `<div class="panel lp-empty"><h2>Audit result</h2><p class="note">Set the scenario, then choose Calculate. The result will expose every stage from received photons through point-defense feasibility.</p></div>`;
  const r = st.result, s = r.summary;
  const cards = [lpMetric("summary.detector_state","Detector",s.detector_state),
    lpMetric("summary.snr","SNR",s.snr), lpMetric("summary.jammer_to_signal","J/S",s.jammer_to_signal),
    lpMetric("summary.measurement_r95_m","Measurement 95%",s.measurement_r95_m),
    lpMetric("summary.future_aim_r95_m","Future aim 95%",s.future_aim_r95_m),
    lpMetric("summary.body_capture_probability","Body capture",s.body_capture_probability),
    lpMetric("summary.patch_capture_probability","Patch capture",s.patch_capture_probability),
    lpMetric("summary.structural_kill_time_s","Structural kill",s.structural_kill_time_s),
    lpMetric("summary.time_to_standoff_s","Time to standoff",s.time_to_standoff_s),
    lpMetric("summary.structural_kill_feasible","Snapshot feasible",s.structural_kill_feasible,s.structural_kill_feasible?"good":"bad")].join("");
  const jammer = r.jammers.map((x,n)=>lpAudit(`Jammer ${n+1}: ${x.id}`,x,`jammers[${n}]`)).join("");
  const chaff = r.chaff.map((x,n)=>lpAudit(`Chaff ${n+1}: ${x.id}`,x,`chaff[${n}]`)).join("");
  return `${st.stale?`<p class="note warn lp-stale">Inputs changed — this result is stale. Calculate again before relying on it.</p>`:""}
    <div class="lp-summary">${cards}</div>
    ${r.warnings.length?`<div class="panel"><h3>Warnings</h3>${r.warnings.map(x=>`<p class="note warn"><b>${esc(x.code)}</b> — ${esc(x.message)}</p>`).join("")}</div>`:""}
    ${lpAudit("Geometry",r.geometry,"geometry",true)}${lpAudit("Lidar signal",r.signal,"signal",true)}
    ${lpAudit("Detector",r.detector,"detector")}${lpAudit("Fire control",r.fire_control,"fire_control")}
    ${lpAudit("Point defense",r.point_defense,"point_defense")}${jammer}${chaff}
    <div class="panel"><h3>Assumptions & limits</h3>${r.assumptions.map(x=>`<p class="note">${esc(x)}</p>`).join("")}
    <p class="note warn">${esc(r.point_defense.model_limitation)}</p></div>`;
}

function bindLpTooltips(root) {
  const close = except => root.querySelectorAll(".lp-help-wrap.open").forEach(w => {
    if (w === except) return; w.classList.remove("open"); w.dataset.locked = "";
    w.querySelector("button").setAttribute("aria-expanded","false");
  });
  const open = (wrap, locked = false) => {
    close(wrap); wrap.classList.add("open"); wrap.dataset.locked = locked ? "1" : "";
    const b = wrap.querySelector("button"), tip = wrap.querySelector(".lp-tooltip"), box = b.getBoundingClientRect();
    b.setAttribute("aria-expanded","true");
    tip.style.left = Math.max(8, Math.min(window.innerWidth - 328, box.left - 10)) + "px";
    tip.style.top = (box.bottom + 7) + "px";
    requestAnimationFrame(() => { if (tip.getBoundingClientRect().bottom > window.innerHeight - 8)
      tip.style.top = Math.max(8, box.top - tip.offsetHeight - 7) + "px"; });
  };
  root.querySelectorAll(".lp-help-wrap").forEach(wrap => {
    const b = wrap.querySelector("button");
    wrap.onmouseenter = () => open(wrap, wrap.dataset.locked === "1");
    wrap.onmouseleave = () => { if (wrap.dataset.locked !== "1" && !wrap.contains(document.activeElement)) close(); };
    b.onfocus = () => open(wrap, false);
    b.onblur = () => { if (wrap.dataset.locked !== "1") close(); };
    b.onclick = e => { e.stopPropagation(); const lock = wrap.dataset.locked !== "1"; lock ? open(wrap,true) : close(); };
  });
  root.onclick = e => { if (!e.target.closest(".lp-help-wrap")) close(); };
  root.onkeydown = e => { if (e.key === "Escape" && root.querySelector(".lp-help-wrap.open")) {
    close(); document.activeElement?.blur(); } };
}
function lpDownload(name, data) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(data,null,2)],{type:"application/json"}));
  const a=document.createElement("a"); a.href=url; a.download=name; a.click(); setTimeout(()=>URL.revokeObjectURL(url),0);
}

function renderLidarPd(main) {
  const st = lpState(), s = st.scenario, geom = st.geometryMode === "simple";
  const detector = Object.entries(LP_DETECTORS).map(([k,v])=>`<option value="${k}" ${st.detectorPreset===k?"selected":""}>${v.label}</option>`).join("");
  const target = Object.entries(LP_TARGETS).map(([k,v])=>`<option value="${k}" ${st.targetPreset===k?"selected":""}>${v.label}</option>`).join("");
  const installed = laserWeapons().map(x=>`<option value="installed:${x.id}">${esc(x.d.name)} — ${esc(x.c.name)}</option>`).join("");
  const weapons = Object.entries(LP_WEAPONS).map(([k,v])=>`<option value="${k}" ${st.weaponPreset===k?"selected":""}>${v[0]}</option>`).join("")+installed;
  const detectorFields = `${lpField("detector.wavelength_m","Lidar wavelength",1e9,"nm")}${lpField("detector.transmitter_power_w","Transmitter power",1e-6,"MW")}
    ${lpField("detector.transmitter_aperture_m","Transmitter aperture",1,"m")}${lpField("detector.transmitter_m2","Transmitter M²")}
    ${lpField("detector.receiver_aperture_m","Receiver aperture",1,"m")}${lpField("detector.integration_time_s","Integration",1e3,"ms")}
    ${lpField("detector.optical_throughput","Optical throughput",100,"%")}${lpField("detector.quantum_efficiency","Quantum efficiency",100,"%")}
    ${lpField("detector.filter_center_m","Filter center",1e9,"nm")}${lpField("detector.filter_fwhm_m","Filter width",1e12,"pm")}
    ${lpField("detector.gate_width_s","Gate width",1e9,"ns")}${lpField("detector.pulse_repetition_hz","Pulse repetition",1,"Hz")}
    ${lpField("detector.pixel_scale_rad","Pixel scale",1e9,"nrad")}${lpField("detector.tracking_window_radius_rad","Tracking window",1e6,"µrad")}
    ${lpField("detector.detector_floor_sigma_rad","Detector floor",1e9,"nrad")}${lpField("detector.background_photons","Background",1,"photons")}
    ${lpField("detector.dark_counts","Dark counts",1,"counts")}${lpField("detector.read_noise_e","Read noise",1,"e⁻ RMS")}
    ${lpField("detector.full_well_e","Full well",1,"e⁻")}${lpField("detector.recovery_time_s","Recovery",1e3,"ms")}
    ${lpField("detector.track_snr_min","Track SNR min")}${lpField("detector.fire_control_snr_min","Fire-control SNR min")}
    ${lpField("detector.ambiguity_ratio_min","Ambiguity ratio")}${lpField("detector.speckle_single_sigma_rad","Single-mode speckle",1e9,"nrad")}
    ${lpField("detector.speckle_temporal_modes","Temporal modes")}${lpField("detector.speckle_wavelength_modes","Wavelength modes")}
    ${lpField("detector.speckle_polarization_modes","Polarization modes")}${lpField("detector.speckle_view_modes","View modes")}`;
  const posLabels = geom ? ["Range","Lateral Y","Lateral Z"] : ["X","Y","Z"];
  const targetFields = `${[0,1,2].map((_,k)=>lpField(`target.position_m[${k}]`,posLabels[k],1e-6,"Mm")).join("")}
    ${lpField("target.projected_area_m2","Projected area",1,"m²")}${lpField("target.characteristic_diameter_m","Characteristic diameter",1,"m")}
    ${lpField("target.body_radius_m","Body radius",1,"m")}${lpField("target.vulnerable_patch_radius_m","Patch radius",1,"m")}
    ${lpField("target.uv_reflectivity","UV reflectivity",100,"%")}${lpField("target.aspect_factor","Aspect factor",100,"%")}
    ${lpField("target.closure_velocity_m_s","Closing velocity",1e-3,"km/s")}${lpField("target.warhead_standoff_m","Warhead standoff",1e-3,"km")}
    ${lpField("target.soft_kill_fluence_j_m2","Soft-kill fluence",1,"J/m²")}${lpField("target.structural_kill_fluence_j_m2","Structural fluence",1,"J/m²")}`;
  const fcFields = `${lpField("fire_control.processing_latency_s","Processing latency",1e3,"ms")}${lpField("fire_control.position_sigma_m","Position sigma",1,"m")}
    ${lpField("fire_control.velocity_sigma_m_s","Velocity sigma",1,"m/s")}${lpField("fire_control.acceleration_sigma_m_s2","Acceleration sigma",1,"m/s²")}
    ${lpField("fire_control.maneuver_persistence_s","Maneuver persistence",1e3,"ms")}${lpField("fire_control.boresight_sigma_rad","Boresight sigma",1e9,"nrad")}
    ${lpField("fire_control.platform_sigma_rad","Platform sigma",1e9,"nrad")}${lpField("fire_control.beam_sigma_rad","Beam sigma",1e9,"nrad")}
    ${lpField("fire_control.reacquisition_time_s","Reacquisition",1e3,"ms")}${lpField("fire_control.minimum_capture_probability","Minimum capture",100,"%")}`;
  const weaponFields = `${lpField("weapon.wavelength_m","Weapon wavelength",1e9,"nm")}${lpField("weapon.aperture_m","Aperture",1,"m")}
    ${lpField("weapon.m2","M²")}${lpField("weapon.listed_optical_power_w","Listed power",1e-6,"MW")}
    ${lpField("weapon.central_lobe_fraction","Central lobe",100,"%")}${lpField("weapon.duty_cycle","Duty cycle",100,"%")}
    ${lpField("weapon.slew_time_s","Slew time",1e3,"ms")}`;
  main.innerHTML = `<div class="lp-toolbar panel"><h2>Lidar Countermeasures & Point Defense</h2><span class="spacer"></span>
    <label>Scenario ${lpTip("scenario_preset")}<select id="lp-scenario-preset"><option value="stress">Aligned jammer stress case</option><option value="clean">Clean baseline</option><option value="dark">Dark rough target</option><option value="full">Full receiver baseline</option></select></label>
    <button id="lp-import">Import</button><input id="lp-import-file" type="file" accept="application/json" hidden>
    <button id="lp-export">Export scenario</button><button id="lp-export-result" ${st.result?"":"disabled"}>Export result</button>
    <button id="lp-calculate" class="primary">Calculate</button></div>
    <div class="lp-workspace"><div class="lp-input-column">
      <div class="panel lp-context">${lpField("scenario_name","Scenario name",1,"","text")}
        <label>Geometry ${lpTip("geometry_mode")}<select id="lp-geometry"><option value="simple" ${geom?"selected":""}>Simple range + offsets</option><option value="advanced" ${!geom?"selected":""}>Advanced X/Y/Z</option></select></label>
        <button id="lp-use-map" class="small">Use map geometry</button></div>
      <div class="panel lp-preset"><label>Detector ${lpTip("detector_preset")}<select id="lp-detector-preset">${detector}</select></label></div>
      ${lpSection("Detector",detectorFields,true)}
      <div class="panel lp-preset"><label>Target ${lpTip("target_preset")}<select id="lp-target-preset">${target}</select></label></div>
      ${lpSection("Target",targetFields,true)}${lpSection("Fire control",fcFields)}
      <div class="panel lp-preset"><label>Weapon ${lpTip("weapon_preset")}<select id="lp-weapon-preset">${weapons}</select></label></div>
      ${lpSection("Point-defense weapon",weaponFields,true)}
      ${lpSection(`Jammers (${s.jammers.length})`,s.jammers.map(lpJammerCard).join("")+`<button id="lp-add-jammer">+ jammer</button>`,false)}
      ${lpSection(`Chaff (${s.chaff.length})`,s.chaff.map(lpChaffCard).join("")+`<button id="lp-add-chaff">+ chaff</button>`,false)}
    </div><div class="lp-result-column" aria-live="polite">${lpResults(st)}</div></div>`;

  bindLpTooltips(main);
  main.querySelectorAll("[data-lp-path]").forEach(input => input.onchange = () => {
    const path=input.dataset.lpPath, old=lpGet(s,path); let value;
    if (input.type === "checkbox") value=input.checked;
    else if (input.type === "number") value=Number(input.value)/(Number(input.dataset.lpScale)||1);
    else value=input.value;
    lpSet(s,path,value); st.stale=!!st.result; st.error=null;
    if (typeof old === "string" && path.endsWith(".id")) renderLidarPd(main);
    else { const note=main.querySelector(".lp-stale"); if(st.result&&!note) main.querySelector(".lp-result-column").insertAdjacentHTML("afterbegin",`<p class="note warn lp-stale">Inputs changed — this result is stale. Calculate again before relying on it.</p>`); }
  });
  $("lp-geometry").onchange=()=>{st.geometryMode=$("lp-geometry").value;renderLidarPd(main);};
  $("lp-scenario-preset").value=st.scenarioPreset;
  $("lp-scenario-preset").onchange=()=>{const v=$("lp-scenario-preset").value;let x=lpDefaultScenario();
    if(v==="clean")x.jammers=[]; if(v==="dark"){x.target.uv_reflectivity=.04;x.detector.speckle_single_sigma_rad=5e-9;x.scenario_name="Dark rough missile";}
    if(v==="full"){x.detector.receiver_aperture_m=30;x.jammers=[];x.scenario_name="Full 30 m receiver baseline";}
    st.scenario=x;st.scenarioPreset=v;st.result=null;st.error=null;renderLidarPd(main);};
  $("lp-detector-preset").onchange=()=>{st.detectorPreset=$("lp-detector-preset").value;Object.assign(s.detector,LP_DETECTORS[st.detectorPreset].values);st.stale=!!st.result;renderLidarPd(main);};
  $("lp-target-preset").onchange=()=>{st.targetPreset=$("lp-target-preset").value;const p=LP_TARGETS[st.targetPreset];Object.assign(s.target,p.values);s.detector.speckle_single_sigma_rad=p.speckle||1e-9;st.stale=!!st.result;renderLidarPd(main);};
  $("lp-weapon-preset").value=st.weaponPreset;
  $("lp-weapon-preset").onchange=()=>{const v=$("lp-weapon-preset").value;st.weaponPreset=v;
    if(v.startsWith("installed:")){const x=laserWeapons().find(q=>q.id===v.slice(10));if(x)Object.assign(s.weapon,{wavelength_m:x.c.lambda_m,aperture_m:x.c.aperture_m,listed_optical_power_w:x.c.p_beam_w});}
    else {const p=LP_WEAPONS[v];Object.assign(s.weapon,{wavelength_m:532e-9,aperture_m:p[1],listed_optical_power_w:p[2],central_lobe_fraction:.84,duty_cycle:.5});}
    st.stale=!!st.result;renderLidarPd(main);};
  $("lp-add-jammer").onclick=()=>{s.jammers.push(lpNewJammer(s.jammers.length+1));st.stale=!!st.result;renderLidarPd(main);};
  $("lp-add-chaff").onclick=()=>{s.chaff.push(lpNewChaff(s.chaff.length+1));st.stale=!!st.result;renderLidarPd(main);};
  main.querySelectorAll("[data-lp-jdel]").forEach(b=>b.onclick=()=>{s.jammers.splice(+b.dataset.lpJdel,1);st.stale=!!st.result;renderLidarPd(main);});
  main.querySelectorAll("[data-lp-cdel]").forEach(b=>b.onclick=()=>{s.chaff.splice(+b.dataset.lpCdel,1);st.stale=!!st.result;renderLidarPd(main);});
  main.querySelectorAll("[data-lp-jcopy]").forEach(b=>b.onclick=()=>{const x=lpClone(s.jammers[+b.dataset.lpJcopy]);x.id+=" copy";s.jammers.splice(+b.dataset.lpJcopy+1,0,x);st.stale=!!st.result;renderLidarPd(main);});
  main.querySelectorAll("[data-lp-ccopy]").forEach(b=>b.onclick=()=>{const x=lpClone(s.chaff[+b.dataset.lpCcopy]);x.id+=" copy";s.chaff.splice(+b.dataset.lpCcopy+1,0,x);st.stale=!!st.result;renderLidarPd(main);});
  const move=(arr,n,d)=>{const [x]=arr.splice(n,1);arr.splice(n+d,0,x);st.stale=!!st.result;renderLidarPd(main);};
  main.querySelectorAll("[data-lp-jup]").forEach(b=>b.onclick=()=>move(s.jammers,+b.dataset.lpJup,-1));
  main.querySelectorAll("[data-lp-jdown]").forEach(b=>b.onclick=()=>move(s.jammers,+b.dataset.lpJdown,1));
  main.querySelectorAll("[data-lp-cup]").forEach(b=>b.onclick=()=>move(s.chaff,+b.dataset.lpCup,-1));
  main.querySelectorAll("[data-lp-cdown]").forEach(b=>b.onclick=()=>move(s.chaff,+b.dataset.lpCdown,1));
  $("lp-calculate").onclick=async()=>{try{$("lp-calculate").disabled=true;st.result=await calc("lidar_pd",s);st.stale=false;st.error=null;}catch(e){st.error=e.message;}renderLidarPd(main);if(st.error)$("lp-validation")?.focus();};
  $("lp-export").onclick=()=>lpDownload((s.scenario_name||"lidar-pd")+".json",s);
  $("lp-export-result").onclick=()=>{if(st.result)lpDownload((s.scenario_name||"lidar-pd")+"-result.json",st.result);};
  $("lp-import").onclick=()=>$("lp-import-file").click();
  $("lp-import-file").onchange=async()=>{const file=$("lp-import-file").files[0];if(!file)return;try{const x=JSON.parse(await file.text());if(String(x.schema_version).split(".")[0]!=="1")throw new Error("Unsupported schema major version; expected 1.x.");
    if(!x.detector||!x.target||!x.fire_control||!x.weapon)throw new Error("The file is not a complete Lidar & PD scenario.");st.scenario=x;st.result=null;st.error=null;renderLidarPd(main);}catch(e){st.error=e.message;renderLidarPd(main);$("lp-validation")?.focus();}};
  $("lp-use-map").onclick=()=>{const ship=planSourceShip(),geo=ship&&missionGeometry(ship);if(!ship||!geo)return toast("Choose a mapped source ship and target on the System Map first.");
    if(!(geo.distance>0))return toast("Map geometry has zero range.");if(!(geo.radialClosing>0))return toast("The selected target is opening range; v1 point-defense feasibility requires positive closing speed.");
    s.target.position_m=[geo.distance,0,0];s.target.closure_velocity_m_s=geo.radialClosing;s.scenario_name=`${ship.name} vs ${geo.name}`;st.stale=!!st.result;renderLidarPd(main);toast("Copied current map range and radial closing speed.",true);};
  window.__LP_TOOLTIP_COVERAGE__={missing:[...LP_MISSING_META],rendered:main.querySelectorAll(".lp-help").length};
  if(LP_MISSING_META.size)main.querySelector(".lp-result-column").insertAdjacentHTML("afterbegin",`<p class="note bad">Tooltip coverage defect: ${esc([...LP_MISSING_META].join(", "))}</p>`);
}

/* =========================== MISSILE LAB ================================ */

const latestMissile = makeLatest();

function defaultPhases(missile) {
  if (!missile) return [];
  const saved = missile.default_phases || [];
  if (saved.length) return saved.map(p => ({ stage_id: p.stage_id,
    frac: p.prop_frac * 100, coast_mm: p.coast_to_range_m == null ? null : p.coast_to_range_m / 1e6 }));
  return missile.stages.map((s, i) => ({ stage_id: s.id, frac: 100,
    coast_mm: i === 0 && missile.stages.length > 1 ? 25 : null }));
}

function renderMissile(main) {
  if (!UI.missile.sel && DB.missiles.length) UI.missile.sel = DB.missiles[0].id;
  const sel = missileById(UI.missile.sel);
  if (!UI.missile.phases) UI.missile.phases = defaultPhases(sel);

  const phaseRows = UI.missile.phases.map((p, i) => `
    <tr>
      <td><select id="ph-stage-${i}">${(sel?.stages || []).map(s => `<option value="${s.id}"
        ${s.id === p.stage_id ? "selected" : ""}>${esc(s.name)}</option>`).join("")}</select></td>
      <td>Burn <input type="number" step="any" id="ph-frac-${i}" value="${p.frac}" style="width:60px">%</td>
      <td>then coast until <input type="number" step="any" id="ph-coast-${i}"
        value="${p.coast_mm ?? ""}" placeholder="—" style="width:80px"> Mm to go</td>
      <td><button class="small" data-ph-up="${i}" ${i === 0 ? "disabled" : ""}>↑</button>
        <button class="small" data-ph-down="${i}" ${i === UI.missile.phases.length - 1 ? "disabled" : ""}>↓</button>
        <button class="small danger" data-ph-del="${i}">✕</button></td>
    </tr>`).join("");

  const op = UI.missile.optimizer || {
    total_mass_kg: 27000, a0_g: 0.5, reactor_specific_power_mw_kg: 1.1,
    radiator_specific_power_mw_kg: 1.6, waste_heat_fraction: 0.05,
    mh_ve: S().prop_mh_ve_m_s, h2_cooling_j_kg: 7e6, mh_cooling_j_kg: 0.4e6,
    n_submunitions: 10, submunition_dv: 50000, tank_fraction: 0.04,
    guidance_mass_kg: 250, reference_sub_dry_kg: 50,
  };
  UI.missile.optimizer = op;

  main.innerHTML = `<div class="cols">
    <div class="col">
      <div class="panel">
        <h2>Missile designs</h2>
        <div id="mis-table"><p class="note">Computing…</p></div>
        <div class="actions">
          <button id="btn-new-missile">New missile</button>
          ${sel ? `<button id="btn-edit-missile">Edit ${esc(sel.name)}</button>
          <button id="btn-del-missile" class="danger">Delete</button>` : ""}
        </div>
        <canvas class="plot" id="mis-accel" height="200"></canvas>
        <div id="mis-stage-detail"></div>
      </div>
    </div>
    <div class="col">
      <div class="panel">
        <h2>Intercept calculator ${sel ? "— " + esc(sel.name) : ""}</h2>
        <div class="field"><label>Launch range (Mm)</label>
          <input type="number" step="any" id="int-range" value="${UI.missile.range_mm}"></div>
        <div class="field"><label>Closing velocity (km/s)</label>
          <input type="number" step="any" id="int-v" value="${UI.missile.vclose_kms}"></div>
        <h3>Burn schedule</h3>
        <table><tr><th>Stage</th><th>Propellant</th><th>After burn</th><th></th></tr>${phaseRows}</table>
        <div class="actions">
          <button id="btn-ph-add">+ phase</button>
          <button id="btn-ph-reset">Standard profile</button>
          <button id="btn-ph-save">Save as doctrine</button>
          <button id="btn-intercept" class="primary">Compute</button>
        </div>
        <p class="note">Standard doctrine: boost ~35% establishing the vector, dark coast, terminal
        burn at the PD gauntlet — the bird arrives with maneuvering reserves, not just speed.
        Negative closing velocity = target opening the range. Evasion modeling deferred to v2.</p>
        <div id="int-out"></div>
      </div>
    </div>
  </div>
  <div class="panel optimizer-panel">
    <h2>Two-stage missile optimizer</h2>
    <p class="note">Size a fusion-heated metallic-hydrogen bus around a payload of complete MH
      submunitions. The search includes reactor, radiator, tank, and guidance mass.</p>
    <div class="optimizer-grid">
      ${optimizerField("opt-mass", "Total bus mass", op.total_mass_kg, "kg")}
      ${optimizerField("opt-a0", "Ignition accel", op.a0_g, "g")}
      ${optimizerField("opt-reactor", "Reactor specific power", op.reactor_specific_power_mw_kg, "MW/kg")}
      ${optimizerField("opt-radiator", "Radiator specific power", op.radiator_specific_power_mw_kg, "MW/kg")}
      ${optimizerField("opt-waste", "Waste heat", op.waste_heat_fraction, "fraction")}
      ${optimizerField("opt-mh-ve", "MH exhaust velocity", op.mh_ve / 1000, "km/s")}
      ${optimizerField("opt-h2-cool", "H₂ flow cooling", op.h2_cooling_j_kg / 1e6, "MJ/kg")}
      ${optimizerField("opt-mh-cool", "MH flow cooling", op.mh_cooling_j_kg / 1e6, "MJ/kg")}
      ${optimizerField("opt-subs", "Submunitions", op.n_submunitions, "count")}
      ${optimizerField("opt-sub-dv", "Submunition Δv", op.submunition_dv / 1000, "km/s")}
      ${optimizerField("opt-tank", "Tank fraction", op.tank_fraction, "of propellant")}
      ${optimizerField("opt-guidance", "Guidance + ECM", op.guidance_mass_kg, "kg")}
      ${optimizerField("opt-ref-sub", "Reference sub dry", op.reference_sub_dry_kg, "kg")}
    </div>
    <div class="actions"><button id="btn-optimize-missile" class="primary">Optimize</button></div>
    <div id="opt-out"></div>
  </div>`;

  $("btn-new-missile").onclick = () => missileModal(null);
  if (sel) {
    $("btn-edit-missile").onclick = () => missileModal(sel);
    $("btn-del-missile").onclick = () => {
      const used = DB.designs.some(d => d.components.some(c => c.missile_id === sel.id));
      if (used) return toast("A magazine references this missile — repoint it first.");
      if (!confirm(`Delete ${sel.name}?`)) return;
      DB.missiles = DB.missiles.filter(m => m.id !== sel.id);
      UI.missile.sel = null; UI.missile.phases = null;
      touch();
    };
  }
  const rereadPhases = () => {
    UI.missile.phases.forEach((p, i) => {
      p.stage_id = $("ph-stage-" + i).value;
      p.frac = num("ph-frac-" + i);
      const c = num("ph-coast-" + i);
      p.coast_mm = Number.isFinite(c) ? c : null;
    });
  };
  main.querySelectorAll("[data-ph-del]").forEach(b =>
    b.onclick = () => { rereadPhases(); UI.missile.phases.splice(+b.dataset.phDel, 1); render(); });
  const movePhase = (i, d) => { rereadPhases(); const [p] = UI.missile.phases.splice(i, 1);
    UI.missile.phases.splice(i + d, 0, p); render(); };
  main.querySelectorAll("[data-ph-up]").forEach(b => b.onclick = () => movePhase(+b.dataset.phUp, -1));
  main.querySelectorAll("[data-ph-down]").forEach(b => b.onclick = () => movePhase(+b.dataset.phDown, 1));
  $("btn-ph-add").onclick = () => {
    rereadPhases();
    UI.missile.phases.push({ stage_id: sel?.stages[0]?.id, frac: 10, coast_mm: null });
    render();
  };
  $("btn-ph-reset").onclick = () => { UI.missile.phases = defaultPhases(sel); render(); };
  $("btn-ph-save").onclick = () => {
    if (!sel) return;
    rereadPhases();
    sel.default_phases = UI.missile.phases.map(p => ({ stage_id: p.stage_id,
      prop_frac: (p.frac || 0) / 100,
      coast_to_range_m: p.coast_mm == null ? null : p.coast_mm * 1e6 }));
    touch(false); toast("Saved intercept doctrine to " + sel.name + ".", true);
  };
  $("btn-intercept").onclick = () => { rereadPhases(); runIntercept(); };
  $("btn-optimize-missile").onclick = runMissileOptimizer;

  updateMissileTable();
  if (UI.missile.result) renderInterceptResult(UI.missile.result);
  if (UI.missile.optimizerResult) renderMissileOptimizerResult(UI.missile.optimizerResult);
}

function optimizerField(id, label, value, unit) {
  return `<label for="${id}"><span>${label}</span><small>${unit}</small>
    <input type="number" step="any" id="${id}" value="${value}"></label>`;
}

function readMissileOptimizer() {
  const p = UI.missile.optimizer;
  Object.assign(p, {
    total_mass_kg: num("opt-mass"), a0_g: num("opt-a0"),
    reactor_specific_power_mw_kg: num("opt-reactor"),
    radiator_specific_power_mw_kg: num("opt-radiator"),
    waste_heat_fraction: num("opt-waste"), mh_ve: num("opt-mh-ve") * 1000,
    h2_cooling_j_kg: num("opt-h2-cool") * 1e6,
    mh_cooling_j_kg: num("opt-mh-cool") * 1e6,
    n_submunitions: Math.round(num("opt-subs")),
    submunition_dv: num("opt-sub-dv") * 1000,
    tank_fraction: num("opt-tank"), guidance_mass_kg: num("opt-guidance"),
    reference_sub_dry_kg: num("opt-ref-sub"),
  });
  return { ...p, g: S().g };
}

async function runMissileOptimizer() {
  try {
    const button = $("btn-optimize-missile");
    button.disabled = true; button.textContent = "Searching…";
    const result = await calc("missile_optimize", readMissileOptimizer());
    UI.missile.optimizerResult = result;
    renderMissileOptimizerResult(result);
  } catch (e) { toast(e.message); }
  finally {
    const button = $("btn-optimize-missile");
    if (button) { button.disabled = false; button.textContent = "Optimize"; }
  }
}

function optimizerDesignRows(d) {
  if (!d) return `<p class="note bad">Insufficient mass for this reference design.</p>`;
  return `<table><tr><th>Exhaust velocity</th><td class="num">${fmtVel(d.ve)}</td></tr>
    <tr><th>Bus Δv</th><td class="num">${fmtVel(d.dv)}</td></tr>
    <tr><th>Mass ratio</th><td class="num">${d.mass_ratio.toFixed(3)}</td></tr>
    <tr><th>Reactor power</th><td class="num">${fmtSI(d.reactor_power_w, "W")}</td></tr>
    <tr><th>Reactor + radiator</th><td class="num">${fmtT(d.power_system_mass_kg / 1000)}</td></tr>
    <tr><th>Submunition payload</th><td class="num">${fmtT(d.submunitions_wet_kg / 1000)}</td></tr>
    <tr><th>Bus dry / propellant</th><td class="num">${fmtT(d.bus_dry_mass_kg / 1000)} / ${fmtT(d.propellant_kg / 1000)}</td></tr>
    <tr><th>Burn / final accel</th><td class="num">${fmtDur(d.burn_time_s)} / ${d.final_accel_g.toFixed(2)} g</td></tr></table>`;
}

function renderMissileOptimizerResult(r) {
  const out = $("opt-out");
  if (!out) return;
  const p = UI.missile.optimizer;
  out.innerHTML = `<div class="readout compact">
      <div class="r"><span class="k">Submunition MR</span><span class="v">${r.submunition_mass_ratio.toFixed(2)}</span></div>
      <div class="r"><span class="k">Bus thrust</span><span class="v accent">${fmtSI(r.thrust_n, "N")}</span></div>
      <div class="r"><span class="k">H₂ cooling limit</span><span class="v">${fmtVel(r.h2_critical_ve)}</span></div>
    </div>
    <canvas class="plot" id="opt-plot" height="240"></canvas>
    <div class="optimizer-results">
      <div><h3>MH + fusion</h3>${optimizerDesignRows(r.reference_fusion)}</div>
      <div><h3>H₂ + fusion</h3>${optimizerDesignRows(r.reference_h2_fusion)}</div>
      <div><h3>Pure MH</h3>${optimizerDesignRows(r.reference_pure_mh)}</div>
    </div>
    <div class="actions"><button id="btn-create-optimized" class="primary"
      ${r.reference_fusion ? "" : "disabled"}>Create optimized bus missile</button></div>
    <p class="note">The created calculator design treats the ${p.n_submunitions} wet submunitions as
      payload; its stage dry mass contains guidance, reactor, radiator, and tankage. The submunition
      terminal Δv remains a separate capability rather than being double-counted as bus Δv.</p>`;

  const series = [
    ["fusion_dv", "MH + fusion", Plot.PALETTE[0]],
    ["h2_fusion_dv", "H₂ + fusion", Plot.PALETTE[1]],
    ["pure_mh_dv", "Pure MH", Plot.PALETTE[2]],
  ].map(([key, label, color]) => {
    const pts = r.sweep.filter(d => d[key] != null);
    return { x: pts.map(d => d.sub_dry_kg), y: pts.map(d => d[key] / 1000), label, color };
  });
  Plot.draw($("opt-plot"), { xlabel: "submunition dry mass (kg)", ylabel: "bus Δv (km/s)",
    series, height: 240 });
  $("btn-create-optimized").onclick = () => createOptimizedMissile(r.reference_fusion);
}

function createOptimizedMissile(d) {
  if (!d) return;
  const p = UI.missile.optimizer;
  const stageId = uid();
  const missile = {
    id: uid(), name: `Optimized fusion bus (${p.n_submunitions} subs)`,
    payload_kg: d.submunitions_wet_kg,
    stages: [{ id: stageId, name: "Optimized fusion bus", dry_mass_kg: d.bus_dry_mass_kg,
      propellant_kg: d.propellant_kg, propulsion: "fusion", isp_s: d.ve / S().g,
      a0_g: p.a0_g, jettison: false }],
    default_phases: [{ stage_id: stageId, prop_frac: 0.35, coast_to_range_m: 25e6 },
                     { stage_id: stageId, prop_frac: 0.65, coast_to_range_m: null }],
    note: `${p.n_submunitions} MH submunitions; ${d.submunition_wet_each_kg.toFixed(1)} kg wet each, ` +
          `${(p.submunition_dv / 1000).toFixed(1)} km/s terminal Δv each.`,
  };
  DB.missiles.push(missile);
  UI.missile.sel = missile.id; UI.missile.phases = defaultPhases(missile);
  UI.missile.result = null;
  touch();
  toast("Created " + missile.name + " and loaded it into the calculator.", true);
}

async function updateMissileTable() {
  await latestMissile(async fresh => {
    const g = S().g;
    const results = await Promise.all(DB.missiles.map(m => calc("missile", missilePayload(m))));
    if (!fresh() || !$("mis-table")) return;
    const rows = DB.missiles.map((m, i) => {
      const r = results[i];
      return `<tr class="clickable ${m.id === UI.missile.sel ? "sel" : ""}" data-mis="${m.id}">
        <td>${esc(m.name)}</td>
        <td>${m.stages.map(s => esc(PROPULSION[s.propulsion] || "custom")).join(" → ")}</td>
        <td class="num">${fmtT(r.m_wet / 1000)}</td>
        <td class="num">${m.stages.length}</td>
        <td class="num">${fmtVel(r.dv)}</td>
        <td class="num">${fmtDur(r.t_burn)}</td>
        <td class="num">${r.stage_reports[0].accel_ignition_g.toFixed(1)} → ${r.a_burnout_g.toFixed(0)} g</td>
      </tr>`;
    }).join("");
    $("mis-table").innerHTML = `<table>
      <tr><th>Name</th><th>Propulsion stages</th><th class="num">Wet</th><th class="num">Stages</th>
      <th class="num">Δv</th><th class="num">Burn</th><th class="num">Accel</th></tr>${rows}</table>`;
    document.querySelectorAll("[data-mis]").forEach(r =>
      r.onclick = () => {
        UI.missile.sel = r.dataset.mis;
        UI.missile.result = null;
        UI.missile.phases = defaultPhases(missileById(UI.missile.sel));
        render();
      });

    const sel = missileById(UI.missile.sel);
    const selIdx = DB.missiles.findIndex(m => m.id === UI.missile.sel);
    if (sel && selIdx >= 0 && $("mis-accel")) {
      const result = results[selIdx];
      Plot.draw($("mis-accel"), {
        xlabel: "burn time (s)", ylabel: "accel (g)",
        series: result.stage_reports.map((sr, i) => {
          const prof = result.profile.filter(p => p.stage_id === sr.id && p.event !== "jettison");
          return { x: prof.map(p => p.t), y: prof.map(p => p.accel_g),
                   label: sr.name, color: Plot.PALETTE[i % Plot.PALETTE.length] };
        }),
        height: 200,
      });
      $("mis-stage-detail").innerHTML = `<h3>Stage performance</h3><table>
        <tr><th>Stage</th><th class="num">Ignition → burnout</th><th class="num">Δv</th>
          <th class="num">Burn</th><th class="num">Accel</th><th>Event</th></tr>
        ${result.stage_reports.map(sr => `<tr><td>${esc(sr.name)}</td>
          <td class="num">${fmtT(sr.ignition_mass_kg/1000)} → ${fmtT(sr.burnout_mass_kg/1000)}</td>
          <td class="num">${fmtVel(sr.dv)}</td><td class="num">${fmtDur(sr.t_burn)}</td>
          <td class="num">${sr.accel_ignition_g.toFixed(1)} → ${sr.accel_burnout_g.toFixed(1)} g</td>
          <td>${sr.jettison ? `jettison ${fmtT(sr.dry_mass_kg/1000)}` : "retained"}</td></tr>`).join("")}</table>`;
    }
  });
}

function missileModal(m) {
  const isNew = !m;
  const draft = m ? JSON.parse(JSON.stringify(m)) : {
    id: uid(), name: "New staged missile", payload_kg: 100,
    stages: [{ id: uid(), name: "Main stage", dry_mass_kg: 100,
      propellant_kg: 800, propulsion: "mh", a0_g: 10, jettison: false }],
    default_phases: [],
  };
  const originalStageIds = draft.stages.map(s => s.id).join(":");
  modal(isNew ? "New missile" : "Edit missile", `
    <div class="grid2">
      <label>Name</label><input type="text" id="mf-name" value="${esc(draft.name)}">
      <label>Payload / bus (kg)</label><input type="number" step="any" id="mf-payload" value="${draft.payload_kg || 0}">
    </div>
    <h3>Stages — ignition order</h3>
    <div id="mf-stages"></div>
    <div class="actions"><button type="button" id="mf-add-stage">+ stage</button></div>
    <p class="note">Each stage's acceleration is defined at ignition for the complete remaining stack.
    Thrust stays constant through that stage. Dry mass is dropped only when jettison is enabled.</p>`, {
    wide: true,
    submitLabel: "Save",
    async onSubmit() {
      readStages();
      draft.name = $("mf-name").value.trim() || "Unnamed missile";
      draft.payload_kg = Math.max(0, num("mf-payload") || 0);
      if (!draft.stages.length) throw new Error("Add at least one stage.");
      await calc("missile", missilePayload(draft)); // server validation before persistence
      if (originalStageIds !== draft.stages.map(s => s.id).join(":") || !draft.default_phases.length)
        draft.default_phases = draft.stages.map((s, i) => ({ stage_id: s.id, prop_frac: 1,
          coast_to_range_m: i === 0 && draft.stages.length > 1 ? 25e6 : null }));
      if (isNew) { DB.missiles.push(draft); UI.missile.sel = draft.id; }
      else Object.assign(m, draft);
      UI.missile.phases = defaultPhases(draft);
      await syncAllDesignerMasses();
      touch();
    },
  });

  function readStages() {
    draft.stages.forEach((s, i) => {
      s.name = $("ms-name-" + i).value.trim() || "Stage " + (i + 1);
      s.propulsion = $("ms-prop-" + i).value;
      s.dry_mass_kg = num("ms-dry-" + i);
      s.propellant_kg = num("ms-fuel-" + i);
      s.a0_g = num("ms-a0-" + i);
      s.jettison = $("ms-jet-" + i).checked;
      if (s.propulsion === "mh") delete s.isp_s;
      else s.isp_s = num("ms-isp-" + i) || (s.propulsion === "fusion" ? S().prop_fusion_isp_s : S().prop_am_isp_s);
    });
  }

  function renderStages() {
    $("mf-stages").innerHTML = draft.stages.map((s, i) => {
      const propOpts = Object.entries(PROPULSION).map(([v, l]) =>
        `<option value="${v}" ${s.propulsion === v ? "selected" : ""}>${l}</option>`).join("");
      const isp = s.isp_s || (s.propulsion === "fusion" ? S().prop_fusion_isp_s : S().prop_am_isp_s);
      return `<div class="stage-card">
        <div class="stage-head"><b>${i + 1}</b><input id="ms-name-${i}" value="${esc(s.name)}" class="wide">
          <span class="spacer"></span>
          <button type="button" class="small" data-ms-up="${i}" ${i === 0 ? "disabled" : ""}>↑</button>
          <button type="button" class="small" data-ms-down="${i}" ${i === draft.stages.length - 1 ? "disabled" : ""}>↓</button>
          <button type="button" class="small" data-ms-copy="${i}">duplicate</button>
          <button type="button" class="small danger" data-ms-del="${i}">delete</button></div>
        <div class="stage-grid">
          <label>Propulsion</label><select id="ms-prop-${i}">${propOpts}</select>
          <label>ISP (s)</label><input type="number" step="any" id="ms-isp-${i}" value="${isp}" ${s.propulsion === "mh" ? "disabled" : ""}>
          <label>Dry mass (kg)</label><input type="number" step="any" id="ms-dry-${i}" value="${s.dry_mass_kg}">
          <label>Propellant (kg)</label><input type="number" step="any" id="ms-fuel-${i}" value="${s.propellant_kg}">
          <label>Ignition accel (g)</label><input type="number" step="any" id="ms-a0-${i}" value="${s.a0_g}">
          <label>After burnout</label><label class="matcheck"><input type="checkbox" id="ms-jet-${i}" ${s.jettison ? "checked" : ""}> jettison ${fmtT((s.dry_mass_kg || 0) / 1000)}</label>
        </div><p class="note">Ve ${fmtVel(stageVe(s))}</p></div>`;
    }).join("");
    draft.stages.forEach((s, i) => {
      $("ms-prop-" + i).onchange = () => {
        readStages();
        if (s.propulsion !== "mh" && !s.isp_s)
          s.isp_s = s.propulsion === "fusion" ? S().prop_fusion_isp_s : S().prop_am_isp_s;
        renderStages();
      };
    });
    document.querySelectorAll("[data-ms-up]").forEach(b => b.onclick = () => move(+b.dataset.msUp, -1));
    document.querySelectorAll("[data-ms-down]").forEach(b => b.onclick = () => move(+b.dataset.msDown, 1));
    document.querySelectorAll("[data-ms-copy]").forEach(b => b.onclick = () => {
      readStages(); const i = +b.dataset.msCopy;
      const c = JSON.parse(JSON.stringify(draft.stages[i]));
      c.id = uid(); c.name += " copy";
      if (i === draft.stages.length - 1) { draft.stages[i].jettison = true; c.jettison = false; }
      else c.jettison = true;
      draft.stages.splice(i + 1, 0, c); renderStages();
    });
    document.querySelectorAll("[data-ms-del]").forEach(b => b.onclick = () => {
      readStages(); draft.stages.splice(+b.dataset.msDel, 1);
      if (draft.stages.length) draft.stages[draft.stages.length - 1].jettison = false;
      renderStages();
    });
  }
  function move(i, d) {
    readStages(); const [s] = draft.stages.splice(i, 1); draft.stages.splice(i + d, 0, s); renderStages();
  }
  $("mf-add-stage").onclick = () => {
    readStages(); const n = draft.stages.length;
    if (n) draft.stages[n - 1].jettison = true;
    draft.stages.push({ id: uid(), name: "Stage " + (n + 1), dry_mass_kg: 50,
      propellant_kg: 500, propulsion: "mh", a0_g: 10, jettison: false });
    renderStages();
  };
  renderStages();
}

async function runIntercept() {
  try {
    const m = missileById(UI.missile.sel);
    if (!m) throw new Error("Select a missile design first.");
    UI.missile.range_mm = num("int-range");
    UI.missile.vclose_kms = num("int-v");
    const byStage = {};
    UI.missile.phases.forEach(p => byStage[p.stage_id] = (byStage[p.stage_id] || 0) + (p.frac || 0));
    const over = Object.entries(byStage).find(([, v]) => v > 100.001);
    if (over) throw new Error(`Burn schedule uses ${over[1].toFixed(0)}% of stage ${over[0]}.`);
    const r = await calc("intercept", {
      range: UI.missile.range_mm * 1e6, v_close0: UI.missile.vclose_kms * 1000,
      ...missilePayload(m),
      phases: UI.missile.phases.map(p => ({
        stage_id: p.stage_id,
        prop_frac: (p.frac || 0) / 100,
        coast_to_range: p.coast_mm != null ? p.coast_mm * 1e6 : null,
      })),
    });
    UI.missile.result = r;
    renderInterceptResult(r);
  } catch (e) { toast(e.message); }
}

function renderInterceptResult(r) {
  const out = $("int-out");
  if (!out) return;
  const timeline = r.timeline.map(p => `
    <tr><td>${esc(p.kind)}</td><td>${esc(p.stage_id || "—")}</td>
      <td class="num">${fmtDur(p.t0)} → ${fmtDur(p.t1)}</td>
      <td class="num">${fmtDist(p.x1)}</td>
      <td class="num">${fmtVel(p.v1)}</td>
      <td class="num">${p.dv > 0 ? "+" + fmtVel(p.dv) : "—"}</td></tr>`).join("");
  out.innerHTML = `
    ${r.hit ? "" : `<p class="note bad">MISS — ${esc(r.miss_reason || "no intercept")}.</p>`}
    <div class="readout">
      <div class="r"><span class="k">Result</span>
        <span class="v ${r.hit ? "good" : "bad"}">${r.hit ? "HIT (" + r.phase + ")" : "MISS"}</span></div>
      ${r.hit ? `<div class="r"><span class="k">Time to target</span><span class="v accent">${fmtDur(r.t_hit)}</span></div>
      <div class="r"><span class="k">Terminal velocity</span><span class="v accent">${fmtVel(r.v_terminal)}</span></div>
      <div class="r"><span class="k">Δv spent</span><span class="v">${fmtVel(r.dv_spent)} / ${fmtVel(r.dv_total)}</span>
        <span class="u">${fmtVel(Math.max(0, r.dv_total - r.dv_spent))} kept for jinking</span></div>` : ""}
    </div>
    <table><tr><th>Phase</th><th>Stage</th><th class="num">Time</th><th class="num">Closed</th>
      <th class="num">Closing v</th><th class="num">Δv</th></tr>${timeline}</table>
    <canvas class="plot" id="int-plot" height="240"></canvas>`;
  Plot.draw($("int-plot"), {
    xlabel: "time (s)", ylabel: "closing velocity (km/s)", y2label: "distance closed (Mm)",
    series: [
      { x: r.profile.map(p => p[0]), y: r.profile.map(p => p[2] / 1000), label: "closing v", color: "#5fb4e8" },
      { x: r.profile.map(p => p[0]), y: r.profile.map(p => p[1] / 1e6), label: "closed", color: "#7ed491", axis: "y2" },
    ],
    vlines: r.timeline.filter(p => p.kind === "burn").map(p =>
      ({ x: p.t1, label: "burnout", color: "#e8d05f" })),
    height: 240,
  });
}

/* ============================ SYSTEM MAP ================================ */

function SOL_BODIES() {
  return JSON.parse(JSON.stringify([
    { id: "sol", name: "Sol", mass_kg: 1.9885e30, radius_m: 6.957e8 },
    { id: "mercury", name: "Mercury", mass_kg: 3.3011e23, radius_m: 2.4397e6, a_m: 5.7909e10, phase0_deg: 252.3, parent: "sol" },
    { id: "venus", name: "Venus", mass_kg: 4.8675e24, radius_m: 6.0518e6, a_m: 1.08209e11, phase0_deg: 182.0, parent: "sol" },
    { id: "earth", name: "Earth", mass_kg: 5.97237e24, radius_m: 6.371e6, a_m: 1.49598e11, phase0_deg: 100.5, parent: "sol" },
    { id: "luna", name: "Luna", mass_kg: 7.342e22, radius_m: 1.7374e6, a_m: 3.844e8, phase0_deg: 0, parent: "earth" },
    { id: "mars", name: "Mars", mass_kg: 6.4171e23, radius_m: 3.3895e6, a_m: 2.27939e11, phase0_deg: 355.5, parent: "sol" },
    { id: "jupiter", name: "Jupiter", mass_kg: 1.8982e27, radius_m: 6.9911e7, a_m: 7.7857e11, phase0_deg: 34.4, parent: "sol" },
    { id: "io", name: "Io", mass_kg: 8.9319e22, radius_m: 1.8216e6, a_m: 4.217e8, phase0_deg: 0, parent: "jupiter" },
    { id: "europa", name: "Europa", mass_kg: 4.7998e22, radius_m: 1.5608e6, a_m: 6.709e8, phase0_deg: 90, parent: "jupiter" },
    { id: "ganymede", name: "Ganymede", mass_kg: 1.4819e23, radius_m: 2.6341e6, a_m: 1.0704e9, phase0_deg: 180, parent: "jupiter" },
    { id: "callisto", name: "Callisto", mass_kg: 1.0759e23, radius_m: 2.4103e6, a_m: 1.8827e9, phase0_deg: 270, parent: "jupiter" },
    { id: "saturn", name: "Saturn", mass_kg: 5.6834e26, radius_m: 5.8232e7, a_m: 1.43353e12, phase0_deg: 50.0, parent: "sol" },
    { id: "uranus", name: "Uranus", mass_kg: 8.681e25, radius_m: 2.5362e7, a_m: 2.87246e12, phase0_deg: 313.2, parent: "sol" },
    { id: "neptune", name: "Neptune", mass_kg: 1.02413e26, radius_m: 2.4622e7, a_m: 4.49506e12, phase0_deg: 304.9, parent: "sol" },
  ]));
}

const latestMap = makeLatest();
const SYS = () => DB.system;
const bodyById = id => SYS().bodies.find(b => b.id === id);
const navOf = ship => SYS().nav[ship.id];
const epochStr = () => "T+" + fmtDur(SYS().epoch_s);
const mappedShips = () => DB.states.filter(s => SYS().nav[s.id]);
const planSourceShip = () => UI.plan.source?.startsWith("ship:")
  ? shipById(UI.plan.source.slice(5)) : null;
function planTargetLabel() {
  if (!UI.plan.target) return "No target";
  const [kind, id] = UI.plan.target.split(":");
  return kind === "ship" ? (shipById(id)?.name || id) : (bodyById(id)?.name || id);
}

// Payload for /api/calc/nav_tick: live mass and floor come from the fleet.
function navShipPayload(ship) {
  const nav = navOf(ship);
  const b = nav.burn;
  return {
    id: ship.id, x: nav.x, y: nav.y, vx: nav.vx, vy: nav.vy,
    mass_kg: shipMass_t(ship) * 1000,
    m_floor_kg: shipDry_t(ship) * 1000,
    landed_on: nav.landed_on || null,
    burn: b ? { thrust: b.thrust_n, mdot: b.mdot_kg_s, t_remaining_s: b.t_remaining_s,
                mode: b.mode, angle_deg: b.angle_deg || 0,
                target_body: b.target_body || null,
                t_start_s: b.t_start_s || 0 } : null,
  };
}

function navTickPayload(dt_s, path_points) {
  const st = S();
  return {
    g_const: st.g_const, epoch_s: SYS().epoch_s, dt_s,
    substep_s: Math.max(st.map_substep_s, dt_s / 200000),
    bodies: SYS().bodies, ships: mappedShips().map(navShipPayload),
    path_points,
  };
}

// One event per burn, logged when it finishes or is cancelled. Propellant was
// already drawn tick by tick, so the deltas are empty — the note carries it.
function flushNavBurn(ship, reason) {
  const nav = navOf(ship);
  const b = nav.burn;
  if (!b) return;
  DB.events.push({
    id: uid(), ship_id: ship.id, date: epochStr(), kind: "burn",
    note: `Nav burn (${b.mode}${b.target_body ? " " + b.target_body : ""}) — ` +
          `Δv ${fmtVel(b.dv_gained || 0)}, propellant −${fmtT(b.prop_drawn_t || 0)}` +
          (reason ? ` — ${reason}` : ""),
    deltas: {},
  });
  delete nav.burn;
}

async function mapTick() {
  if (UI.map.busy) return;
  UI.map.busy = true;
  try {
    const dt = UI.map.tick_s * Math.max(1, Math.round(UI.map.nticks));
    if (!(dt > 0)) throw new Error("Tick length must be positive.");
    const out = await calc("nav_tick", navTickPayload(dt, 40));
    // Commit the clock first so events flushed below carry the end-of-tick stamp.
    SYS().epoch_s = out.epoch_s;
    for (const so of out.ships) {
      const ship = shipById(so.id);
      const nav = navOf(ship);
      if (!ship || !nav) continue;
      Object.assign(nav, { x: so.x, y: so.y, vx: so.vx, vy: so.vy,
                           landed_on: so.landed_on || null });
      // The 2D nav vector is authoritative while mapped; keep the legacy scalar
      // as a derived magnitude for Fleet summaries and eventual un-placement.
      ship.velocity_kms = Math.hypot(so.vx, so.vy) / 1000;
      if (so.prop_used_kg > 0)
        ship.propellant_t = Math.max(0, ship.propellant_t - so.prop_used_kg / 1000);
      if (nav.burn) {
        nav.burn.t_remaining_s = so.burn_t_remaining_s;
        nav.burn.t_start_s = so.burn_t_start_remaining_s;
        nav.burn.prop_drawn_t = (nav.burn.prop_drawn_t || 0) + so.prop_used_kg / 1000;
        nav.burn.dv_gained = (nav.burn.dv_gained || 0) + so.dv_spent;
        const cut = so.notes.find(n => n.includes("floor"));
        if (so.burn_t_remaining_s <= 0) flushNavBurn(ship, cut || null);
      }
      for (const n of so.notes)
        if (/Landed|IMPACT|Lifting/.test(n))
          DB.events.push({ id: uid(), ship_id: ship.id, date: epochStr(),
                           kind: "nav", note: n, deltas: {} });
    }
    UI.map.bodyOut = out.bodies;
    // Time moved: pending (uncommitted) nodes and intercept solutions are stale.
    UI.map.node = null;
    UI.map.nodeUndo = null;
    UI.map.intercept = null;
    touch(false);
    UI.map.busy = false;
    render();
    if (UI.map.playing) setTimeout(() => { if (UI.map.playing) mapTick(); }, 250);
  } catch (e) {
    UI.map.playing = false;
    UI.map.busy = false;
    toast(e.message);
    render();
  } finally { UI.map.busy = false; }
}

function toggleMapPlay() {
  UI.map.playing = !UI.map.playing;
  if (UI.map.playing) mapTick(); else render();
}

// Course projection: same engine, results drawn but never committed. A pending
// maneuver node overrides its ship's committed burn so you see the plan live.
async function updateProjection() {
  await latestMap(async fresh => {
    const dt = Math.max(1, S().map_project_d * 86400);
    const payload = navTickPayload(dt, 220);
    const node = UI.map.node;
    if (node?.burn) {
      const sp = payload.ships.find(s => s.id === node.shipId);
      if (sp) sp.burn = node.burn;
    }
    const key = JSON.stringify({ epoch: SYS().epoch_s, dt, ships: payload.ships,
      bodies: SYS().bodies.map(b => [b.id, b.mass_kg, b.a_m, b.phase0_deg, b.parent]),
      node: node?.burn || null });
    if (key === UI.map.projKey && Object.keys(UI.map.proj.ships).length) {
      drawMap(); return;
    }
    const out = await calc("nav_tick", payload);
    if (!fresh()) return;
    UI.map.proj = {
      ships: Object.fromEntries(out.ships.map(s => [s.id, s.path])),
      bodies: Object.fromEntries(out.bodies.map(b => [b.id, b.path])),
    };
    UI.map.projKey = key;
    drawMap();
  });
}

async function refreshBodyPositions() {
  const out = await calc("nav_tick", navTickPayload(0, 2));
  UI.map.bodyOut = out.bodies;
}

function mapView() {
  if (!UI.map.view) {
    // Start framed on the inner system, out a bit past Mars.
    const cv = $("map-canvas");
    const px = Math.min(cv?.clientWidth || 800, cv?.clientHeight || 560);
    UI.map.view = { cx: 0, cy: 0, mpp: (2.6e11 * 2) / px };
  }
  return UI.map.view;
}

function fitMap(maxR) {
  const cv = $("map-canvas");
  const px = Math.min(cv.clientWidth, cv.clientHeight);
  UI.map.view = { cx: 0, cy: 0, mpp: (maxR * 2.2) / px };
  drawMap();
}

// Frame transform: positions shift by the frame body's current position;
// future path samples shift by the frame body's position at the same time.
const frameBody = () =>
  UI.map.frame ? (UI.map.bodyOut || []).find(b => b.id === UI.map.frame) : null;
const framePath = () => (UI.map.frame ? UI.map.proj.bodies[UI.map.frame] : null) || null;

function relPath(path, fpath, f) {
  if (!path) return null;
  if (!fpath || !f) return path;
  let j = 0;
  return path.map(p => {
    while (j < fpath.length - 1 &&
           Math.abs(fpath[j + 1][0] - p[0]) < Math.abs(fpath[j][0] - p[0])) j++;
    return [p[0], p[1] - fpath[j][1], p[2] - fpath[j][2]];
  });
}

function drawMap() {
  const cv = $("map-canvas");
  if (!cv || !UI.map.bodyOut) return;
  const f = frameBody();
  const fx = f ? f.x : 0, fy = f ? f.y : 0;
  const fvx = f ? f.vx : 0, fvy = f ? f.vy : 0;
  const fpath = framePath();

  const pos = Object.fromEntries(UI.map.bodyOut.map(b => [b.id, b]));
  const bodies = SYS().bodies.map(b => ({
    id: b.id, name: b.name || b.id,
    x: (pos[b.id]?.x ?? 0) - fx, y: (pos[b.id]?.y ?? 0) - fy,
    radius: b.radius_m, orbit_a: b.a_m || 0,
    parent_xy: b.parent && pos[b.parent]
      ? [pos[b.parent].x - fx, pos[b.parent].y - fy] : null,
    sel: UI.map.sel === "body:" + b.id,
    target: UI.plan.target === "body:" + b.id,
  }));
  const ships = mappedShips().map(s => {
    const nav = navOf(s);
    return {
      id: s.id, name: s.name, x: nav.x - fx, y: nav.y - fy,
      vx: nav.vx - fvx, vy: nav.vy - fvy,
      sel: UI.map.sel === "ship:" + s.id,
      source: UI.plan.source === "ship:" + s.id,
      target: UI.plan.target === "ship:" + s.id,
      landed: !!nav.landed_on, burning: !!(nav.burn && nav.burn.t_remaining_s > 0),
      path: relPath(UI.map.proj.ships[s.id], fpath, f),
    };
  });

  const extraPaths = [];
  if (UI.map.intercept?.result?.path)
    extraPaths.push({ path: relPath(UI.map.intercept.result.path, fpath, f),
                      color: "#6ae8d4", dash: [2, 3] });
  let node = null;
  if (UI.map.node) {
    const n = UI.map.node;
    // node sits on the projected path; transform via the frame path at its time
    let nx = n.x, ny = n.y;
    if (fpath && f) {
      let j = 0;
      while (j < fpath.length - 1 &&
             Math.abs(fpath[j + 1][0] - n.t_s) < Math.abs(fpath[j][0] - n.t_s)) j++;
      nx -= fpath[j][1]; ny -= fpath[j][2];
    }
    node = { x: nx, y: ny, angle_deg: n.angle_deg,
             label: `Δv ${fmtVel(n.dv)} @ T+${fmtDur(SYS().epoch_s + n.t_s)}` };
  }

  SysMap.draw(cv, { bodies, ships, extraPaths, node, layers: UI.map.layers,
                    weaponRanges: UI.map.weaponRanges || [] }, mapView());
}

function renderMap(main) {
  const st = S();
  if (UI.map.tick_s == null) UI.map.tick_s = st.map_tick_s;

  const tickOpts = [[60, "1 min"], [600, "10 min"], [3600, "1 h"], [21600, "6 h"],
                    [86400, "1 d"], [864000, "10 d"]];
  const bodyRows = SYS().bodies.map(b => `
    <tr class="clickable ${UI.map.sel === "body:" + b.id ? "sel" : ""}" data-mapsel="body:${b.id}">
      <td>${esc(b.name || b.id)}</td>
      <td class="num">${b.a_m ? fmtDist(b.a_m) : "root"}</td></tr>`).join("");
  const shipRows = DB.states.map(s => {
    const nav = navOf(s);
    const status = !nav ? `<button class="small" data-place="${s.id}">Place</button>`
      : nav.landed_on ? `⏚ ${esc(bodyById(nav.landed_on)?.name || nav.landed_on)}`
      : fmtVel(Math.hypot(nav.vx, nav.vy)) + (nav.burn?.t_remaining_s > 0 ? " ▶" : "");
    return `<tr class="clickable ${UI.map.sel === "ship:" + s.id ? "sel" : ""}"
      data-mapsel="${nav ? "ship:" + s.id : ""}">
      <td>${esc(s.name)}</td><td>${status}</td></tr>`;
  }).join("");

  const source = planSourceShip();
  main.classList.add("map-main");
  main.innerHTML = `<div class="map-workspace ${UI.map.sidebarCollapsed ? "left-collapsed" : ""} ${UI.map.inspectorCollapsed ? "right-collapsed" : ""}">
    <aside class="map-sidebar">
      <div class="panel">
        <h2>Simulation</h2>
        <div class="readout"><div class="r"><span class="k">Clock</span>
          <span class="v accent">${epochStr()}</span></div></div>
        <div class="field"><label>Tick</label>
          <select id="map-tick-sel">${tickOpts.map(([v, l]) =>
            `<option value="${v}" ${UI.map.tick_s === v ? "selected" : ""}>${l}</option>`).join("")}
            <option value="__custom" ${tickOpts.some(([v]) => v === UI.map.tick_s) ? "" : "selected"}>custom…</option>
          </select>
          <input type="number" step="any" id="map-tick" value="${UI.map.tick_s}"
            style="width:90px;display:${tickOpts.some(([v]) => v === UI.map.tick_s) ? "none" : "inline-block"}"> s</div>
        <div class="field"><label>× count</label>
          <input type="number" id="map-nticks" value="${UI.map.nticks}" min="1" style="width:70px"></div>
        <button id="map-advance" class="primary" ${UI.map.busy ? "disabled" : ""}>${UI.map.busy ? "Advancing…" : "Advance"}</button>
        <button id="map-play">${UI.map.playing ? "Pause" : "Play"}</button>
        <p class="note">Active nav burns draw propellant and thrust through every tick;
        gravity from every body acts throughout.</p>
      </div>
      <div class="panel">
        <h2>Bodies</h2>
        <table><tr><th>Name</th><th class="num">Orbit</th></tr>${bodyRows}</table>
        <div class="actions">
          <button id="map-add-body" class="small">Add body</button>
          <button id="map-reset-sol" class="small">Reset to Sol</button>
        </div>
      </div>
      <div class="panel">
        <h2>Ships</h2>
        <table><tr><th>Name</th><th>Status</th></tr>${shipRows ||
          `<tr><td colspan=2 class="note">commission ships on the Fleet tab</td></tr>`}</table>
      </div>
    </aside>
    <div class="map-center">
      <div class="panel plan-bar">
        <button id="map-toggle-left" class="small" title="Toggle object browser">${UI.map.sidebarCollapsed ? "Show objects" : "Hide objects"}</button>
        <span class="chip"><span>Source</span><b>${esc(source?.name || "Select a mapped ship")}</b></span>
        <span class="chip ${UI.plan.target ? "active" : ""}"><span>Target</span><b>${esc(planTargetLabel())}</b></span>
        <button id="map-pick-target" ${source ? "" : "disabled"} class="${UI.map.pickTarget ? "primary" : ""}">${UI.map.pickTarget ? "Click target…" : "Pick target"}</button>
        ${UI.plan.target ? `<button id="map-clear-target" class="small">Clear</button>` : ""}
        <button id="map-toggle-right" class="small" title="Toggle inspector">${UI.map.inspectorCollapsed ? "Show inspector" : "Hide inspector"}</button>
      </div>
      <div class="panel">
        <div id="map-hover" class="map-hover note">Hover an object for position and velocity.</div>
        <canvas id="map-canvas" class="plot" style="height:540px;cursor:grab"></canvas>
        <div class="actions">
          <span class="field"><label>Frame</label>
            <select id="map-frame">
              <option value="">heliocentric</option>
              ${SYS().bodies.map(b => `<option value="${b.id}"
                ${UI.map.frame === b.id ? "selected" : ""}>${esc(b.name || b.id)}</option>`).join("")}
            </select></span>
          <button class="small" id="map-fit-inner">Fit inner system</button>
          <button class="small" id="map-fit-all">Fit all</button>
          <span class="field"><label>Project courses (d)</label>
            <input type="number" step="any" id="map-proj" value="${st.map_project_d}" style="width:70px"></span>
          <span class="layer-toggles">${Object.entries(UI.map.layers).map(([k,v]) =>
            `<button class="small ${v ? "active" : ""}" data-map-layer="${k}">${k}</button>`).join("")}</span>
          <span class="map-legend"><i class="source"></i>source <i class="target"></i>target
            <i class="plan"></i>planned <i class="kill"></i>weapon</span>
          <span class="note">drag pan · wheel zoom · click select · click a course to drop a node, drag its ring to aim</span>
        </div>
      </div>
    </div>
    <aside class="map-inspector">
      <div class="panel inset" id="map-detail"><p class="note">Select a ship or body.</p></div>
    </aside>
  </div>`;

  $("map-tick-sel").onchange = () => {
    const v = $("map-tick-sel").value;
    if (v === "__custom") { $("map-tick").style.display = "inline-block"; }
    else { $("map-tick").style.display = "none"; $("map-tick").value = v; UI.map.tick_s = +v; }
  };
  $("map-tick").onchange = () => { UI.map.tick_s = num("map-tick"); };
  $("map-nticks").onchange = () => { UI.map.nticks = Math.max(1, Math.round(num("map-nticks"))); };
  $("map-advance").onclick = mapTick;
  $("map-play").onclick = () => toggleMapPlay();
  $("map-toggle-left").onclick = () => { UI.map.sidebarCollapsed = !UI.map.sidebarCollapsed; render(); };
  $("map-toggle-right").onclick = () => { UI.map.inspectorCollapsed = !UI.map.inspectorCollapsed; render(); };
  $("map-pick-target").onclick = () => { UI.map.pickTarget = !UI.map.pickTarget; render(); };
  if ($("map-clear-target")) $("map-clear-target").onclick = () => { UI.plan.target = null; UI.map.pickTarget = false; render(); };
  $("map-add-body").onclick = () => bodyModal(null);
  $("map-reset-sol").onclick = () => {
    if (!confirm("Replace all bodies with the default Sol system? Ships stay where they are.")) return;
    SYS().bodies = SOL_BODIES();
    UI.map.bodyOut = null;
    touch();
    refreshBodyPositions().then(() => { drawMap(); renderMapDetail(); });
  };
  $("map-frame").onchange = () => {
    UI.map.frame = $("map-frame").value;
    UI.map.weaponRanges = [];
    // Reframe the view on the new origin at a sensible scale.
    const fb = UI.map.frame ? bodyById(UI.map.frame) : null;
    const children = fb ? SYS().bodies.filter(b => b.parent === fb.id) : [];
    const r = !fb ? 2.6e11
      : children.length ? Math.max(...children.map(c => c.a_m)) * 1.4
      : fb.radius_m * 200;
    fitMap(r);
    renderMapDetail();
  };
  $("map-fit-inner").onclick = () => fitMap(2.6e11);
  $("map-fit-all").onclick = () =>
    fitMap(Math.max(2.6e11, ...SYS().bodies.map(b => b.a_m || 0)));
  $("map-proj").onchange = () => { S().map_project_d = num("map-proj"); touch(false); updateProjection(); };
  main.querySelectorAll("[data-map-layer]").forEach(b => b.onclick = () => {
    UI.map.layers[b.dataset.mapLayer] = !UI.map.layers[b.dataset.mapLayer];
    b.classList.toggle("active", UI.map.layers[b.dataset.mapLayer]); drawMap();
  });
  main.querySelectorAll("[data-mapsel]").forEach(r => r.onclick = () => {
    if (!r.dataset.mapsel) return;
    if (UI.map.pickTarget) {
      if (r.dataset.mapsel === UI.plan.source) return toast("Source and target must be different.");
      UI.plan.target = r.dataset.mapsel; UI.map.pickTarget = false; render(); return;
    }
    UI.map.sel = r.dataset.mapsel;
    if (r.dataset.mapsel.startsWith("ship:")) UI.plan.source = r.dataset.mapsel;
    render();
  });
  main.querySelectorAll("[data-place]").forEach(b =>
    b.onclick = e => { e.stopPropagation(); placeShipModal(shipById(b.dataset.place)); });

  bindMapCanvas();
  (UI.map.bodyOut ? Promise.resolve() : refreshBodyPositions())
    .then(() => { drawMap(); renderMapDetail(); updateProjection(); })
    .catch(e => toast(e.message));
}

// The pending node's frame-transformed position (must match drawMap's math).
function nodeXY() {
  const n = UI.map.node;
  if (!n) return null;
  const f = frameBody(), fpath = framePath();
  let nx = n.x, ny = n.y;
  if (f && fpath) {
    let j = 0;
    while (j < fpath.length - 1 &&
           Math.abs(fpath[j + 1][0] - n.t_s) < Math.abs(fpath[j][0] - n.t_s)) j++;
    nx -= fpath[j][1]; ny -= fpath[j][2];
  }
  return [nx, ny];
}

function bindMapCanvas() {
  const cv = $("map-canvas");
  let drag = null;
  cv.onmousedown = e => {
    const v = mapView();
    // Grabbing the maneuver node ring starts an aim drag instead of a pan.
    const nxy = nodeXY();
    if (!UI.map.pickTarget && nxy && SysMap.pick(cv, v, e, [{ kind: "node", id: "n", x: nxy[0], y: nxy[1] }], 16)) {
      UI.map.nodeUndo = JSON.parse(JSON.stringify(UI.map.node));
      drag = { mode: "node" };
      return;
    }
    drag = { mode: "pan", x: e.clientX, y: e.clientY, cx: v.cx, cy: v.cy, moved: false };
  };
  cv.onmousemove = e => {
    if (!drag) {
      const f = frameBody(), fx = f ? f.x : 0, fy = f ? f.y : 0;
      const objs = [
        ...mappedShips().map(s => ({ kind: "ship", id: s.id, x: navOf(s).x - fx, y: navOf(s).y - fy })),
        ...(UI.map.bodyOut || []).map(b => ({ kind: "body", id: b.id, x: b.x - fx, y: b.y - fy })),
      ];
      const hit = SysMap.pick(cv, mapView(), e, objs, 18);
      const hover = $("map-hover");
      if (hover) {
        if (!hit) hover.textContent = UI.map.pickTarget ? "Click a body or ship to set the target." : "Hover an object for position and velocity.";
        else if (hit.kind === "ship") {
          const s = shipById(hit.id), n = navOf(s);
          hover.textContent = `${s.name} · ${fmtVel(Math.hypot(n.vx, n.vy))} · click ${UI.map.pickTarget ? "to target" : "to select"}`;
        } else {
          const b = bodyById(hit.id), p = (UI.map.bodyOut || []).find(x => x.id === hit.id);
          hover.textContent = `${b.name} · ${p ? fmtVel(Math.hypot(p.vx, p.vy)) : ""} · click ${UI.map.pickTarget ? "to target" : "to select"}`;
        }
      }
      cv.style.cursor = hit ? "pointer" : "grab";
      return;
    }
    const v = mapView();
    if (drag.mode === "node") {
      const n = UI.map.node;
      const nxy = nodeXY();
      const [wx, wy] = SysMap.eventWorld(cv, v, e);
      const dx = wx - nxy[0], dy = wy - nxy[1];
      n.angle_deg = (Math.atan2(dy, dx) * 180 / Math.PI + 360) % 360;
      // drag length sets Δv: screen pixels, 50 m/s per px, zoom-independent
      n.dv = Math.max(1, Math.hypot(dx, dy) / v.mpp * 50);
      drawMap();
      renderNodeReadout();
      return;
    }
    const dx = e.clientX - drag.x, dy = e.clientY - drag.y;
    if (Math.abs(dx) + Math.abs(dy) > 3) drag.moved = true;
    v.cx = drag.cx - dx * v.mpp;
    v.cy = drag.cy + dy * v.mpp;
    if (drag.moved) drawMap();
  };
  cv.onmouseup = e => {
    const d = drag;
    drag = null;
    if (d?.mode === "node") { previewNode(); return; }
    if (d?.moved) return;
    // Click on the selected ship's projected course → drop / move a node there.
    const selShip = UI.map.sel?.startsWith("ship:") ? UI.map.sel.slice(5) : null;
    if (!UI.map.pickTarget && selShip && UI.map.proj.ships[selShip]) {
      const f = frameBody();
      const shown = relPath(UI.map.proj.ships[selShip], framePath(), f);
      const hit = SysMap.pathHit(cv, mapView(), e, shown, 10);
      if (hit) {
        const abs = UI.map.proj.ships[selShip][hit.index];
        const prev = UI.map.proj.ships[selShip][Math.max(0, hit.index - 1)];
        const prograde = Math.atan2(abs[2] - prev[2], abs[1] - prev[1]) * 180 / Math.PI;
        const old = UI.map.node;
        UI.map.nodeUndo = old ? JSON.parse(JSON.stringify(old)) : null;
        UI.map.node = {
          shipId: selShip, t_s: abs[0], x: abs[1], y: abs[2],
          angle_deg: old?.shipId === selShip ? old.angle_deg : (prograde + 360) % 360,
          dv: old?.shipId === selShip ? old.dv : 100,
          ve_kms: old?.ve_kms ?? sliderToVe(UI.plan.slider) / 1000,
          burn: null,
        };
        previewNode();
        renderMapDetail();
        return;
      }
    }
    // otherwise: select nearest ship or body (in the current frame)
    const f = frameBody();
    const fx = f ? f.x : 0, fy = f ? f.y : 0;
    const objs = [
      ...mappedShips().map(s => ({ kind: "ship", id: s.id,
                                   x: navOf(s).x - fx, y: navOf(s).y - fy })),
      ...(UI.map.bodyOut || []).map(b => ({ kind: "body", id: b.id,
                                            x: b.x - fx, y: b.y - fy })),
    ];
    const hit = SysMap.pick(cv, mapView(), e, objs);
    if (hit) {
      const ref = hit.kind + ":" + hit.id;
      if (UI.map.pickTarget) {
        if (ref === UI.plan.source) return toast("Source and target must be different.");
        UI.plan.target = ref; UI.map.pickTarget = false;
      }
      else {
        UI.map.sel = ref;
        if (hit.kind === "ship") UI.plan.source = ref;
      }
      render();
    }
  };
  cv.onmouseleave = () => { drag = null; };
  cv.onwheel = e => {
    e.preventDefault();
    const v = mapView();
    const [wx, wy] = SysMap.eventWorld(cv, v, e);
    const f = e.deltaY > 0 ? 1.3 : 1 / 1.3;
    v.mpp *= f;
    v.cx = wx - (wx - v.cx) * f;
    v.cy = wy - (wy - v.cy) * f;
    drawMap();
  };
}

/* ---- maneuver nodes: KSP-style plan → same nav.burn machinery ---------- */

// Build the node's burn (gear → thrust/flow, Δv → duration) and re-project.
async function previewNode() {
  const n = UI.map.node;
  if (!n) return;
  try {
    const ship = shipById(n.shipId);
    const st = S(), s = sums(designById(ship.design_id));
    const ve = n.ve_kms * 1000;
    const g = await calc("gear", {
      p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
      ve, ve_max: st.ve_max_m_s, f_cap: s.f_cap || null,
    });
    const m0 = shipMass_t(ship) * 1000;
    const reserveFloor = Math.min(m0, shipDry_t(ship) * 1000 * Math.exp(UI.plan.reserve_kms * 1000 / ve));
    const conv = await calc("burn_for_dv", {
      thrust: g.thrust, mdot: g.mdot,
      m0, m_floor: reserveFloor,
      dv: Math.max(1, n.dv),
    });
    // centre the finite burn on the impulsive node time
    n.burn = {
      thrust: g.thrust, mdot: g.mdot,
      t_start_s: Math.max(0, n.t_s - conv.t_burn_s / 2),
      t_remaining_s: conv.t_burn_s,
      mode: "angle", angle_deg: n.angle_deg, target_body: null,
    };
    n.conv = conv;
    renderNodeReadout();
    updateProjection();
  } catch (e) { toast(e.message); }
}

function renderNodeReadout() {
  const el = $("node-readout");
  const n = UI.map.node;
  if (!el || !n) return;
  const ship = shipById(n.shipId);
  const propAfter = n.conv && ship ? Math.max(0, ship.propellant_t - n.conv.prop_kg / 1000) : null;
  el.innerHTML = `Δv <b>${fmtVel(n.dv)}</b> at heading <b>${n.angle_deg.toFixed(1)}°</b>,
    ignition T+${fmtDur(SYS().epoch_s + (n.burn ? n.burn.t_start_s : n.t_s))}` +
    (n.conv ? ` — burn ${fmtDur(n.conv.t_burn_s)}, ${fmtT(n.conv.prop_kg / 1000)} propellant` +
      `, ${fmtT(propAfter)} propellant remains` +
      (n.conv.clamped ? ` <span class="bad">— clamped at the floor (max ${fmtVel(n.conv.dv_possible)})</span>` :
        ` <span class="good">— reserve floor respected</span>`)
    : "");
}

function commitNode() {
  const n = UI.map.node;
  if (!n?.burn) return toast("Aim the node first (drag its ring or edit the fields).");
  const ship = shipById(n.shipId);
  const nav = navOf(ship);
  if (nav.burn) flushNavBurn(ship, "replaced by new plan");
  nav.burn = {
    thrust_n: n.burn.thrust, mdot_kg_s: n.burn.mdot, ve_m_s: n.ve_kms * 1000,
    mode: "angle", angle_deg: n.angle_deg, target_body: null,
    t_start_s: n.burn.t_start_s, t_remaining_s: n.burn.t_remaining_s,
    prop_drawn_t: 0, dv_gained: 0, dv_planned: n.dv,
  };
  UI.map.node = null;
  UI.map.nodeUndo = null;
  touch();
}

/* ---- intercept planner: Terra-Invicta-style "go here on this budget" --- */

function missionGeometry(ship) {
  if (!ship || !navOf(ship) || !UI.plan.target) return null;
  const [kind, id] = UI.plan.target.split(":");
  const sn = navOf(ship);
  let tx, ty, tvx, tvy, name;
  if (kind === "ship") {
    const target = shipById(id), tn = target && navOf(target);
    if (!tn) return null;
    ({ x: tx, y: ty, vx: tvx, vy: tvy } = tn); name = target.name;
  } else {
    const target = (UI.map.bodyOut || []).find(b => b.id === id);
    if (!target) return null;
    ({ x: tx, y: ty, vx: tvx, vy: tvy } = target); name = bodyById(id)?.name || id;
  }
  const dx = tx - sn.x, dy = ty - sn.y, distance = Math.hypot(dx, dy);
  const rvx = tvx - sn.vx, rvy = tvy - sn.vy;
  const radialClosing = distance > 0 ? -(dx * rvx + dy * rvy) / distance : 0;
  return { name, distance, radialClosing, relativeSpeed: Math.hypot(rvx, rvy), tx, ty, tvx, tvy };
}

async function updateMapPlanning(ship) {
  const out = $("map-gear-out");
  if (!out) return;
  try {
    const st = S(), s = sums(designById(ship.design_id));
    const ve = sliderToVe(UI.plan.slider);
    const [g, dv] = await Promise.all([
      calc("gear", { p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
        ve, ve_max: st.ve_max_m_s, f_cap: s.f_cap || null, mass_kg: shipMass_t(ship) * 1000 }),
      calc("deltav", { ve, m_wet: shipMass_t(ship) * 1000, m_dry: shipDry_t(ship) * 1000,
        dv_reserve: UI.plan.reserve_kms * 1000 }),
    ]);
    if (!$("map-gear-out")) return;
    out.innerHTML = `<div class="r"><span class="k">Ve</span><span class="v accent">${fmtVel(ve)}</span></div>
      <div class="r"><span class="k">Thrust / accel</span><span class="v">${fmtSI(g.thrust,"N")} · ${fmtMg(g.accel)}</span>
        <span class="u">${g.capped ? "nozzle capped" : "jet limited"}</span></div>
      <div class="r"><span class="k">Flow</span><span class="v">${fmtSI(g.mdot,"kg/s")}</span></div>
      <div class="r"><span class="k">Available Δv</span><span class="v ${dv.dv > 0 ? "good" : "bad"}">${fmtVel(Math.max(0,dv.dv))}</span>
        <span class="u">holding ${UI.plan.reserve_kms} km/s</span></div>`;

    const nav = navOf(ship), f = frameBody();
    const ranges = [];
    for (const laser of s.lasers.filter(l => (l.profiles || []).length)) {
      const lr = await calc("laser_profiles", { p_beam: laser.p_beam_w, aperture: laser.aperture_m,
        lambda: laser.lambda_m, eta_drill: st.eta_drill, open_fire_factor: st.open_fire_factor,
        profiles: laser.profiles.map(p => { const mat = materialByName(p.material); return {
          name: p.name, rho: mat.rho, e_vap_mj: mat.e_vap_mj,
          t_pulse_s: p.t_pulse_s, threshold_mm: p.threshold_mm }; }) });
      const kill = Math.max(...lr.profiles.map(p => p.r_kill));
      const open = Math.max(...lr.profiles.map(p => p.r_open));
      ranges.push({ x: nav.x - (f?.x || 0), y: nav.y - (f?.y || 0), radius: open,
                    color: "rgba(240,160,80,.45)", label: laser.name + " open" });
      ranges.push({ x: nav.x - (f?.x || 0), y: nav.y - (f?.y || 0), radius: kill,
                    color: "rgba(232,106,106,.55)", label: laser.name + " kill" });
    }
    UI.map.weaponRanges = ranges; drawMap();
  } catch (e) { out.innerHTML = `<p class="note bad">${esc(e.message)}</p>`; }
}

async function compareMission(ship) {
  const el = $("mission-compare");
  const geo = missionGeometry(ship);
  if (!el || !geo) return;
  el.innerHTML = `<p class="note">Computing analytic estimates…</p>`;
  try {
    const st = S(), s = sums(designById(ship.design_id)), ve = sliderToVe(UI.plan.slider);
    const gear = await calc("gear", { p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
      ve, ve_max: st.ve_max_m_s, f_cap: s.f_cap || null });
    const m0 = shipMass_t(ship) * 1000, dry = shipDry_t(ship) * 1000;
    const dv = await calc("deltav", { ve, m_wet: m0, m_dry: dry, dv_reserve: UI.plan.reserve_kms * 1000 });
    const [flip, sprint] = await Promise.all([
      calc("travel", { distance: geo.distance, ve, thrust: gear.thrust, mdot: gear.mdot,
        m0, m_dry: dry, dv_reserve: UI.plan.reserve_kms * 1000 }),
      calc("sprint", { distance: geo.distance, v0: geo.radialClosing, ve, thrust: gear.thrust,
        mdot: gear.mdot, m0, m_floor: dv.m_floor }),
    ]);
    el.innerHTML = `<div class="compare-grid">
      <div><b>Flip-and-burn</b><span>${flip.feasible ? fmtDur(flip.t_total) : "infeasible"}</span>
        <small>analytic, rest-to-rest · ${fmtT(flip.prop_used_kg/1000)} propellant</small></div>
      <div><b>Sprint</b><span>${sprint.hit ? fmtDur(sprint.t_total) : "miss"}</span>
        <small>analytic radial estimate · arrive ${sprint.hit ? fmtVel(sprint.v_arrival) : "—"}</small></div>
      <div><b>Full gravity</b><span>Use Solve below</span><small>committable maneuver-node solution</small></div>
    </div>`;
  } catch (e) { el.innerHTML = `<p class="note bad">${esc(e.message)}</p>`; }
}

async function solveIntercept(ship) {
  try {
    const st = S();
    const nav = navOf(ship);
    const s = sums(designById(ship.design_id));
    const ve = sliderToVe(UI.plan.slider);
    const g = await calc("gear", {
      p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
      ve, ve_max: st.ve_max_m_s, f_cap: s.f_cap || null,
    });
    const targetVal = UI.plan.target;
    if (!targetVal) throw new Error("Pick a target on the map first.");
    const [tKind, tId] = targetVal.split(":");
    let target_body = null, target_ship = null, capture;
    if (tKind === "body") {
      target_body = tId;
      capture = bodyById(tId).radius_m * 5;
    } else {
      const tn = navOf(shipById(tId));
      target_ship = { x: tn.x, y: tn.y, vx: tn.vx, vy: tn.vy };
      capture = 1e7;
    }
    const capOverride = num("ic-capture");
    if (Number.isFinite(capOverride) && capOverride > 0) capture = capOverride * 1e6;
    $("ic-out").innerHTML = `<p class="note">Searching departure windows…</p>`;
    const r = await calc("nav_intercept", {
      g_const: st.g_const, epoch_s: SYS().epoch_s, bodies: SYS().bodies,
      x: nav.x, y: nav.y, vx: nav.vx, vy: nav.vy,
      mass_kg: shipMass_t(ship) * 1000, m_floor_kg: shipDry_t(ship) * 1000,
      thrust: g.thrust, mdot: g.mdot,
      target_body, target_ship,
      dv_max: num("ic-dv") * 1000,
      depart_max_s: UI.plan.depart_d * 86400,
      horizon_s: UI.plan.arrive_d * 86400,
      capture_radius_m: capture,
    });
    UI.map.intercept = { shipId: ship.id, result: r, ve_kms: ve / 1000 };
    drawMap();
    $("ic-out").innerHTML = `
      ${r.feasible ? "" : `<p class="note bad">No intercept inside the budget and window —
        closest approach ${fmtDist(r.closest_m)}. More Δv, a longer window, or wait for the geometry.</p>`}
      <div class="readout">
        <div class="r"><span class="k">Result</span>
          <span class="v ${r.feasible ? "good" : "bad"}">${r.feasible ? "INTERCEPT" : "MISS"}</span></div>
        <div class="r"><span class="k">Depart</span><span class="v">T+${fmtDur(SYS().epoch_s + r.t_depart_s)}</span>
          <span class="u">${fmtDur(r.t_depart_s)} from now</span></div>
        <div class="r"><span class="k">Burn</span><span class="v accent">${fmtVel(r.dv)}</span>
          <span class="u">hdg ${r.heading_deg.toFixed(1)}°, ${fmtDur(r.t_burn_s)}, ${fmtT(r.prop_kg / 1000)}</span></div>
        ${r.feasible ? `
        <div class="r"><span class="k">Arrive</span><span class="v accent">T+${fmtDur(SYS().epoch_s + r.t_arrival_s)}</span>
          <span class="u">transit ${fmtDur(r.transit_s)}</span></div>
        <div class="r"><span class="k">V rel at target</span><span class="v">${fmtVel(r.v_rel_arrival)}</span></div>` : ""}
      </div>
      ${r.feasible ? `<div class="actions"><button id="ic-load" class="primary">Load into burn planner</button></div>` : ""}
      <p class="note">Impulsive-burn search under full gravity; the plan converts to a finite burn
      centred on the node. Cyan dashes on the map show the solution course.</p>`;
    if (r.feasible) $("ic-load").onclick = () => {
      UI.map.node = {
        shipId: ship.id, t_s: r.t_depart_s,
        x: r.path.find(p => p[0] >= r.t_depart_s)?.[1] ?? nav.x,
        y: r.path.find(p => p[0] >= r.t_depart_s)?.[2] ?? nav.y,
        angle_deg: r.heading_deg, dv: r.dv, ve_kms: ve / 1000, burn: null,
      };
      UI.map.intercept = null;
      previewNode();
      renderMapDetail();
    };
  } catch (e) { $("ic-out").innerHTML = ""; toast(e.message); }
}

function renderMapDetail() {
  const el = $("map-detail");
  if (!el) return;
  const sel = UI.map.sel;
  if (!sel) { el.innerHTML = `<p class="note">Select a ship or body.</p>`; return; }
  const [kind, id] = sel.split(":");
  if (kind === "body") return renderBodyDetail(el, bodyById(id));
  const ship = shipById(id);
  if (!ship || !navOf(ship)) { el.innerHTML = `<p class="note">Gone.</p>`; return; }
  const nav = navOf(ship);
  const shipDesign = designById(ship.design_id);
  const shipSums = sums(shipDesign);
  const pos = Object.fromEntries((UI.map.bodyOut || []).map(b => [b.id, b]));
  const c = S().c_m_s;

  const speed = Math.hypot(nav.vx, nav.vy);
  const heading = (Math.atan2(nav.vy, nav.vx) * 180 / Math.PI + 360) % 360;
  const rSun = Math.hypot(nav.x, nav.y);

  const bodyDist = SYS().bodies.map(b => {
    const p = pos[b.id];
    if (!p) return null;
    const d = Math.hypot(nav.x - p.x, nav.y - p.y);
    return { name: b.name || b.id, d, vrel: Math.hypot(nav.vx - p.vx, nav.vy - p.vy) };
  }).filter(Boolean).sort((a, b) => a.d - b.d).slice(0, 4);

  const shipDist = mappedShips().filter(s => s.id !== ship.id).map(s => {
    const o = navOf(s);
    return { name: s.name, d: Math.hypot(nav.x - o.x, nav.y - o.y),
             vrel: Math.hypot(nav.vx - o.vx, nav.vy - o.vy) };
  }).sort((a, b) => a.d - b.d);

  const distRow = x => `<tr><td>${esc(x.name)}</td>
    <td class="num">${fmtDist(x.d)}</td>
    <td class="num">${fmtDur(x.d / c)} lag</td>
    <td class="num">${fmtVel(x.vrel)} rel</td></tr>`;

  const b = nav.burn;
  const f = frameBody();
  const frameRow = f ? (() => {
    const rvx = nav.vx - f.vx, rvy = nav.vy - f.vy;
    return `<div class="r"><span class="k">Rel ${esc(bodyById(f.id)?.name || f.id)}</span>
      <span class="v accent">${fmtVel(Math.hypot(rvx, rvy))}</span>
      <span class="u">${fmtDist(Math.hypot(nav.x - f.x, nav.y - f.y))} out</span></div>`;
  })() : "";

  const node = UI.map.node?.shipId === ship.id ? UI.map.node : null;
  const isSource = UI.plan.source === "ship:" + ship.id;
  const geo = isSource ? missionGeometry(ship) : null;

  el.innerHTML = `
    <h2>${esc(ship.name)}</h2>
    <div class="actions"><button id="map-pin-source" class="${isSource ? "primary" : ""}">${isSource ? "Planning source" : "Use as planning source"}</button>
      <button id="map-open-fleet" class="small">Open in Fleet</button>
      <button id="map-open-drive" class="small">Open Drive &amp; Travel</button></div>
    <div class="readout">
      <div class="r"><span class="k">Position</span>
        <span class="v">${fmtDist(nav.x)}, ${fmtDist(nav.y)}</span>
        <span class="u">r☉ ${fmtDist(rSun)}</span></div>
      <div class="r"><span class="k">Velocity</span>
        <span class="v accent">${fmtVel(speed)}</span>
        <span class="u">heading ${heading.toFixed(1)}°</span></div>
      ${frameRow}
      <div class="r"><span class="k">Mass</span><span class="v">${fmtT(shipMass_t(ship))}</span>
        <span class="u">${fmtT(ship.propellant_t)} propellant</span></div>
      ${nav.landed_on ? `<div class="r"><span class="k">Status</span>
        <span class="v">Landed — ${esc(bodyById(nav.landed_on)?.name || nav.landed_on)}</span></div>` : ""}
      ${b ? `<div class="r"><span class="k">Burn</span>
        <span class="v warn">${esc(b.mode)}${b.target_body ? " → " + esc(b.target_body) : ""}</span>
        <span class="u">${b.t_start_s > 0 ? "ignition in " + fmtDur(b.t_start_s) + ", " : ""}${fmtDur(b.t_remaining_s)} burn @ ${fmtSI(b.thrust_n, "N")}</span></div>` : ""}
    </div>
    <table><tr><th>Nearest</th><th class="num">Distance</th><th class="num">Light</th><th class="num">Closing</th></tr>
      ${bodyDist.map(distRow).join("")}${shipDist.map(distRow).join("")}</table>
    <h3>Shared gearing</h3>
    <div class="gear-slider"><input type="range" id="map-gear-slider" min="0" max="1000" value="${UI.plan.slider}">
      <div class="gear-ends"><span>thrust</span><span>efficiency</span></div></div>
    <div class="field"><label>Reserve (km/s)</label><input type="number" step="any" id="map-reserve" value="${UI.plan.reserve_kms}"></div>
    <div class="readout compact" id="map-gear-out"><span class="note">Computing…</span></div>
    <h3>Target &amp; estimates</h3>
    ${geo ? `<div class="readout compact">
      <div class="r"><span class="k">Target</span><span class="v accent">${esc(geo.name)}</span></div>
      <div class="r"><span class="k">Separation</span><span class="v">${fmtDist(geo.distance)}</span><span class="u">${fmtDur(geo.distance/c)} light lag</span></div>
      <div class="r"><span class="k">Radial closing</span><span class="v">${fmtVel(geo.radialClosing)}</span></div>
      <div class="r"><span class="k">Relative speed</span><span class="v">${fmtVel(geo.relativeSpeed)}</span></div></div>
      <div class="actions"><button id="mission-compare-btn">Compare travel modes</button>
        <button id="mission-seed-missile">Open target in Missile Lab</button></div>
      <div id="mission-compare"></div>`
      : `<p class="note">${isSource ? "Use Pick target above, then click a body or ship." : "Make this ship the planning source first."}</p>`}
    <h3>Installed weapons</h3>
    <div class="actions">
      ${shipSums.lasers.map(l => `<button class="small" data-map-laser="${shipDesign.id}:${l.id}">${esc(l.name)} · Laser Lab</button>`).join("")}
      ${shipSums.magazines.map(m => `<button class="small" data-map-missile="${m.missile_id}">${esc(m.name)} · Missile Lab</button>`).join("")}
      ${!shipSums.lasers.length && !shipSums.magazines.length ? `<span class="note">No installed weapons.</span>` : ""}
    </div>
    ${node ? `
    <h3>Maneuver node</h3>
    <div>
      <span class="field"><label>Node at (h from now)</label>
        <input type="number" step="any" id="nd-t" value="${(node.t_s / 3600).toFixed(2)}" style="width:80px"></span>
      <span class="field"><label>Heading (°)</label>
        <input type="number" step="any" id="nd-hdg" value="${node.angle_deg.toFixed(1)}" style="width:80px"></span>
      <span class="field"><label>Δv (m/s)</label>
        <input type="number" step="any" id="nd-dv" value="${node.dv.toFixed(0)}" style="width:90px"></span>
      <span class="field"><label>Gear Ve (km/s)</label>
        <input type="number" step="any" id="nd-ve" value="${node.ve_kms}" style="width:80px"></span>
    </div>
    <p class="note" id="node-readout"></p>
    <div class="actions">
      <button id="nd-commit" class="primary">Commit burn</button>
      ${UI.map.nodeUndo ? `<button id="nd-undo" class="small">Undo edit</button>` : ""}
      <button id="nd-discard" class="danger small">Discard node</button>
    </div>` : `<p class="note">Click this ship's projected course to drop a maneuver node,
      then drag the node's ring to aim it.</p>`}
    <h3>Intercept planner</h3>
    <div>
      <span class="field"><label>Target</label><b>${esc(geo?.name || "pick on map")}</b></span>
      <span class="field"><label>Max Δv (km/s)</label>
        <input type="number" step="any" id="ic-dv" value="100" style="width:80px"></span>
      <span class="field"><label>Depart ≤ (d)</label>
        <input type="number" step="any" id="ic-depart" value="${UI.plan.depart_d}" style="width:70px"></span>
      <span class="field"><label>Arrive ≤ (d)</label>
        <input type="number" step="any" id="ic-arrive" value="${UI.plan.arrive_d}" style="width:70px"></span>
      <span class="field"><label>Capture (Mm, blank = 5R)</label>
        <input type="number" step="any" id="ic-capture" value="${UI.plan.capture_mm ?? ""}" style="width:80px"></span>
      <button id="ic-solve" class="primary" ${geo ? "" : "disabled"}>Solve full-gravity intercept</button>
    </div>
    <div id="ic-out"></div>
    <div class="actions">
      ${b ? `<button id="map-cancel-burn" class="danger">Cut burn (logs partial)</button>`
          : `<button id="map-set-burn">Program burn manually</button>`}
      <button id="map-replace">Move / re-place</button>
      <button id="map-unplace" class="danger small">Remove from map</button>
    </div>`;

  if (node) {
    const reread = () => {
      node.t_s = Math.max(0, num("nd-t") * 3600);
      node.angle_deg = ((num("nd-hdg") % 360) + 360) % 360;
      node.dv = Math.max(1, num("nd-dv"));
      node.ve_kms = num("nd-ve") || node.ve_kms;
      // re-anchor the node marker onto the path at the new time
      const path = UI.map.proj.ships[ship.id];
      if (path) {
        const p = path.reduce((a, q) => Math.abs(q[0] - node.t_s) < Math.abs(a[0] - node.t_s) ? q : a);
        node.x = p[1]; node.y = p[2];
      }
      previewNode();
    };
    ["nd-t", "nd-hdg", "nd-dv", "nd-ve"].forEach(id => $(id).onchange = reread);
    $("nd-commit").onclick = commitNode;
    if ($("nd-undo")) $("nd-undo").onclick = () => {
      UI.map.node = UI.map.nodeUndo; UI.map.nodeUndo = null; previewNode(); renderMapDetail();
    };
    $("nd-discard").onclick = () => { UI.map.node = null; UI.map.nodeUndo = null; updateProjection(); renderMapDetail(); };
    renderNodeReadout();
  }
  $("map-pin-source").onclick = () => { UI.plan.source = "ship:" + ship.id; render(); };
  $("map-open-fleet").onclick = () => { UI.shipId = ship.id; UI.tab = "fleet"; render(); };
  $("map-open-drive").onclick = () => { UI.plan.source = "ship:" + ship.id; UI.tab = "drive"; render(); };
  $("map-gear-slider").oninput = () => {
    UI.plan.slider = +$("map-gear-slider").value;
    clearTimeout(renderMapDetail._gear); renderMapDetail._gear = setTimeout(() => updateMapPlanning(ship), 70);
  };
  $("map-reserve").oninput = () => { UI.plan.reserve_kms = num("map-reserve") || 0;
    clearTimeout(renderMapDetail._gear); renderMapDetail._gear = setTimeout(() => updateMapPlanning(ship), 70); };
  $("ic-depart").onchange = () => UI.plan.depart_d = Math.max(0, num("ic-depart") || 0);
  $("ic-arrive").onchange = () => UI.plan.arrive_d = Math.max(0.1, num("ic-arrive") || 400);
  $("ic-capture").onchange = () => UI.plan.capture_mm = Number.isFinite(num("ic-capture")) ? num("ic-capture") : null;
  if ($("mission-compare-btn")) $("mission-compare-btn").onclick = () => compareMission(ship);
  if ($("mission-seed-missile")) $("mission-seed-missile").onclick = () => {
    UI.missile.range_mm = geo.distance / 1e6; UI.missile.vclose_kms = geo.radialClosing / 1000;
    UI.tab = "missile"; render();
  };
  el.querySelectorAll("[data-map-laser]").forEach(b => b.onclick = () => {
    UI.laser.weapon = b.dataset.mapLaser; UI.laser.loadedWeapon = null; UI.tab = "laser"; render();
  });
  el.querySelectorAll("[data-map-missile]").forEach(b => b.onclick = () => {
    UI.missile.sel = b.dataset.mapMissile; UI.missile.phases = defaultPhases(missileById(b.dataset.mapMissile));
    if (geo) { UI.missile.range_mm = geo.distance / 1e6; UI.missile.vclose_kms = geo.radialClosing / 1000; }
    UI.tab = "missile"; render();
  });
  $("ic-solve").onclick = () => solveIntercept(ship);
  if (b) $("map-cancel-burn").onclick = () => {
    flushNavBurn(ship, "cut early");
    touch();
  };
  else $("map-set-burn").onclick = () => burnModal(ship);
  $("map-replace").onclick = () => placeShipModal(ship);
  $("map-unplace").onclick = () => {
    if (!confirm(`Take ${ship.name} off the map? (Fleet state is untouched.)`)) return;
    if (nav.burn) flushNavBurn(ship, "removed from map");
    delete SYS().nav[ship.id];
    if (UI.map.sel === "ship:" + ship.id) UI.map.sel = null;
    if (UI.map.node?.shipId === ship.id) UI.map.node = null;
    if (UI.map.intercept?.shipId === ship.id) UI.map.intercept = null;
    if (UI.plan.source === "ship:" + ship.id) { UI.plan.source = null; UI.plan.target = null; }
    touch();
  };
  updateMapPlanning(ship);
}

function renderBodyDetail(el, body) {
  if (!body) { el.innerHTML = `<p class="note">Gone.</p>`; return; }
  const pos = (UI.map.bodyOut || []).find(b => b.id === body.id);
  el.innerHTML = `
    <h2>${esc(body.name || body.id)}</h2>
    <div class="readout">
      <div class="r"><span class="k">Mass</span><span class="v">${body.mass_kg.toExponential(3)} kg</span></div>
      <div class="r"><span class="k">Radius</span><span class="v">${fmtDist(body.radius_m)}</span></div>
      ${body.a_m ? `<div class="r"><span class="k">Orbit</span>
        <span class="v">${fmtDist(body.a_m)}</span>
        <span class="u">around ${esc(body.parent || "?")}</span></div>` : ""}
      ${pos ? `<div class="r"><span class="k">Position</span>
        <span class="v">${fmtDist(pos.x)}, ${fmtDist(pos.y)}</span>
        <span class="u">${fmtVel(Math.hypot(pos.vx, pos.vy))}</span></div>` : ""}
    </div>
    <div class="actions">
      <button id="map-target-body" class="primary">Set as target</button>
      <button id="map-edit-body">Edit</button>
      <button id="map-del-body" class="danger">Delete</button>
    </div>`;
  $("map-target-body").onclick = () => { UI.plan.target = "body:" + body.id; UI.map.pickTarget = false; render(); };
  $("map-edit-body").onclick = () => bodyModal(body);
  $("map-del-body").onclick = () => {
    if (SYS().bodies.some(b => b.parent === body.id))
      return toast("Other bodies orbit this one — repoint them first.");
    if (Object.values(SYS().nav).some(n => n.landed_on === body.id))
      return toast("A ship is landed on this body.");
    if (!confirm(`Delete ${body.name || body.id}?`)) return;
    SYS().bodies = SYS().bodies.filter(b => b.id !== body.id);
    UI.map.sel = null;
    UI.map.bodyOut = null;
    touch();
  };
}

function bodyModal(body) {
  const isNew = !body;
  body = body || { id: "body-" + uid(), name: "New body", mass_kg: 1e23,
                   radius_m: 1e6, a_m: 3e11, phase0_deg: 0, parent: "sol" };
  const parentOpts = [`<option value="" ${!body.parent ? "selected" : ""}>— none (fixed root)</option>`,
    ...SYS().bodies.filter(b => b.id !== body.id).map(b =>
      `<option value="${b.id}" ${body.parent === b.id ? "selected" : ""}>${esc(b.name || b.id)}</option>`)].join("");
  modal(isNew ? "Add body" : "Edit " + (body.name || body.id), `
    <div class="grid2">
      <label>Name</label><input type="text" id="bf-name" value="${esc(body.name || "")}">
      <label>Mass (kg)</label><input type="number" step="any" id="bf-mass" value="${body.mass_kg}">
      <label>Radius (m)</label><input type="number" step="any" id="bf-radius" value="${body.radius_m}">
      <label>Parent</label><select id="bf-parent">${parentOpts}</select>
      <label>Orbit radius (m)</label><input type="number" step="any" id="bf-a" value="${body.a_m || 0}">
      <label>Phase at T+0 (°)</label><input type="number" step="any" id="bf-phase" value="${body.phase0_deg || 0}">
    </div>
    <p class="note">Circular, coplanar orbits — period follows from the parent's mass.
    Gravity acts on ships; bodies themselves stay on their rails.</p>`, {
    submitLabel: "Save",
    onSubmit() {
      const mass = num("bf-mass");
      const radius = num("bf-radius");
      const orbit = num("bf-a");
      const phase = num("bf-phase");
      if (!Number.isFinite(mass) || mass <= 0) throw new Error("Mass must be a positive number.");
      if (!Number.isFinite(radius) || radius <= 0) throw new Error("Radius must be a positive number.");
      if (!Number.isFinite(orbit) || orbit < 0) throw new Error("Orbit radius must be a non-negative number.");
      if (!Number.isFinite(phase)) throw new Error("Orbital phase must be a valid number.");
      body.name = $("bf-name").value.trim() || body.id;
      body.mass_kg = mass;
      body.radius_m = radius;
      body.parent = $("bf-parent").value || undefined;
      body.a_m = orbit;
      body.phase0_deg = phase;
      if (isNew) SYS().bodies.push(body);
      UI.map.bodyOut = null;
      touch();
    },
  });
}

function placeShipModal(ship) {
  const bodyOpts = SYS().bodies.filter(b => b.a_m || !b.parent).map(b =>
    `<option value="${b.id}" ${b.id === "earth" ? "selected" : ""}>${esc(b.name || b.id)}</option>`).join("");
  modal("Place " + ship.name, `
    <div class="grid2">
      <label>Mode</label>
      <select id="pl-mode">
        <option value="orbit">circular orbit around a body</option>
        <option value="surface">landed on a body</option>
        <option value="space">deep space (x, y, vx, vy)</option>
        <option value="ledger">deep space at ledger velocity (${fmtVel(shipV(ship))})</option>
      </select>
      <label id="pl-body-l">Body</label><select id="pl-body">${bodyOpts}</select>
      <label id="pl-alt-l">Altitude (km)</label><input type="number" step="any" id="pl-alt" value="1000">
      <label id="pl-dir-l">Direction</label>
      <select id="pl-dir"><option value="1">prograde (CCW)</option><option value="-1">retrograde (CW)</option></select>
      <label id="pl-x-l" style="display:none">x (Gm)</label><input type="number" step="any" id="pl-x" value="150" style="display:none">
      <label id="pl-y-l" style="display:none">y (Gm)</label><input type="number" step="any" id="pl-y" value="0" style="display:none">
      <label id="pl-vx-l" style="display:none">vx (km/s)</label><input type="number" step="any" id="pl-vx" value="0" style="display:none">
      <label id="pl-vy-l" style="display:none">vy (km/s)</label><input type="number" step="any" id="pl-vy" value="0" style="display:none">
      <label id="pl-hdg-l" style="display:none">Heading (°)</label><input type="number" step="any" id="pl-hdg" value="0" style="display:none">
    </div>`, {
    submitLabel: "Place",
    async onSubmit() {
      const mode = $("pl-mode").value;
      const pos = Object.fromEntries((UI.map.bodyOut || []).map(b => [b.id, b]));
      let nav;
      if (mode === "orbit" || mode === "surface") {
        const b = bodyById($("pl-body").value);
        const p = pos[b.id];
        if (!p) throw new Error("Body positions not loaded yet — try again.");
        if (mode === "surface") {
          nav = { x: p.x + b.radius_m, y: p.y, vx: p.vx, vy: p.vy, landed_on: b.id };
        } else {
          const r = b.radius_m + Math.max(1, num("pl-alt")) * 1000;
          const ov = await calc("orbit_v", { g_const: S().g_const, mass_kg: b.mass_kg, r_m: r });
          const dir = +$("pl-dir").value;
          nav = { x: p.x + r, y: p.y, vx: p.vx, vy: p.vy + dir * ov.v_circ, landed_on: null };
        }
      } else {
        const x = num("pl-x") * 1e9, y = num("pl-y") * 1e9;
        if (mode === "ledger") {
          const th = (num("pl-hdg") || 0) * Math.PI / 180;
          nav = { x, y, vx: shipV(ship) * Math.cos(th), vy: shipV(ship) * Math.sin(th), landed_on: null };
        } else {
          nav = { x, y, vx: num("pl-vx") * 1000, vy: num("pl-vy") * 1000, landed_on: null };
        }
      }
      const old = SYS().nav[ship.id];
      if (old?.burn) flushNavBurn(ship, "re-placed on map");
      SYS().nav[ship.id] = nav;
      ship.velocity_kms = Math.hypot(nav.vx, nav.vy) / 1000;
      UI.map.sel = "ship:" + ship.id;
      UI.plan.source = "ship:" + ship.id;
      touch();
    },
  });
  const sync = () => {
    const m = $("pl-mode").value;
    const show = (ids, on) => ids.forEach(i => {
      $(i).style.display = on ? "" : "none";
      if ($(i + "-l")) $(i + "-l").style.display = on ? "" : "none";
    });
    show(["pl-body", "pl-alt", "pl-dir"], m === "orbit");
    if (m === "surface") { show(["pl-body"], true); }
    show(["pl-x", "pl-y"], m === "space" || m === "ledger");
    show(["pl-vx", "pl-vy"], m === "space");
    show(["pl-hdg"], m === "ledger");
  };
  $("pl-mode").onchange = sync;
  sync();
}

function burnModal(ship) {
  const st = S();
  const nav = navOf(ship);
  const bodyOpts = SYS().bodies.map(b =>
    `<option value="${b.id}">${esc(b.name || b.id)}</option>`).join("");
  modal("Program burn — " + ship.name, `
    <div class="grid2">
      <label>Gear Ve (km/s)</label><input type="number" step="any" id="bn-ve" value="${sliderToVe(UI.plan.slider) / 1000}">
      <label>Direction</label>
      <select id="bn-mode">
        <option value="prograde">prograde</option>
        <option value="retrograde">retrograde</option>
        <option value="body">toward body…</option>
        <option value="angle">fixed heading…</option>
      </select>
      <label id="bn-body-l" style="display:none">Body</label>
      <select id="bn-body" style="display:none">${bodyOpts}</select>
      <label id="bn-angle-l" style="display:none">Heading (°)</label>
      <input type="number" step="any" id="bn-angle" value="0" style="display:none">
      <label>Duration</label>
      <span><input type="number" step="any" id="bn-dur" value="1" style="width:80px">
      <select id="bn-unit"><option value="3600" selected>h</option><option value="60">min</option>
      <option value="1">s</option><option value="86400">d</option></select></span>
      <label>Ignition after</label>
      <span><input type="number" step="any" id="bn-delay" value="0" style="width:80px">
      <select id="bn-delay-unit"><option value="3600" selected>h</option><option value="60">min</option>
      <option value="1">s</option><option value="86400">d</option></select></span>
    </div>
    <p class="note">Thrust and flow come from the gearing endpoint at this Ve; the burn runs
    through map ticks, draws propellant as it goes, and logs one event when it ends.
    ${nav.landed_on ? "The ship lifts off on the first tick." : ""}</p>`, {
    submitLabel: "Set burn",
    async onSubmit() {
      const design = designById(ship.design_id), s = sums(design);
      const ve = num("bn-ve") * 1000;
      const dur = num("bn-dur") * +$("bn-unit").value;
      if (!(ve > 0) || !(dur > 0)) throw new Error("Ve and duration must be positive.");
      const g = await calc("gear", {
        p_fusion: s.p_fusion, f_exh: st.f_exh, eta_noz: st.eta_noz,
        ve, ve_max: st.ve_max_m_s, f_cap: s.f_cap || null,
      });
      nav.burn = {
        thrust_n: g.thrust, mdot_kg_s: g.mdot, ve_m_s: ve,
        mode: $("bn-mode").value, angle_deg: num("bn-angle") || 0,
        target_body: $("bn-mode").value === "body" ? $("bn-body").value : null,
        t_start_s: Math.max(0, (num("bn-delay") || 0) * +$("bn-delay-unit").value),
        t_remaining_s: dur, prop_drawn_t: 0, dv_gained: 0,
      };
      touch();
    },
  });
  $("bn-mode").onchange = () => {
    const m = $("bn-mode").value;
    ["bn-body", "bn-body-l"].forEach(i => $(i).style.display = m === "body" ? "" : "none");
    ["bn-angle", "bn-angle-l"].forEach(i => $(i).style.display = m === "angle" ? "" : "none");
  };
}

/* =============================== boot =================================== */

function migrateMissileSchema() {
  let changed = false;
  for (const m of DB.missiles || []) {
    if (!Array.isArray(m.stages) || !m.stages.length) {
      const wet = Math.max(0, +m.m_wet_kg || 0);
      const mr = Math.max(1.000001, +m.mr || 1.000001);
      const dry = wet / mr;
      const stageId = m.id + "-stage-1";
      m.payload_kg = 0;
      m.stages = [{
        id: stageId, name: "Main stage", dry_mass_kg: dry,
        propellant_kg: wet - dry, propulsion: m.propulsion || "mh",
        a0_g: m.a0_g || 10, jettison: false,
      }];
      if (m.isp_s != null) m.stages[0].isp_s = m.isp_s;
      if (m.ve_m_s != null) m.stages[0].ve_m_s = m.ve_m_s;
      const boost = Math.min(1, Math.max(0.01, m.boost_frac ?? 0.35));
      m.default_phases = boost >= 0.999
        ? [{ stage_id: stageId, prop_frac: 1, coast_to_range_m: null }]
        : [{ stage_id: stageId, prop_frac: boost, coast_to_range_m: 25e6 },
           { stage_id: stageId, prop_frac: 1 - boost, coast_to_range_m: null }];
      delete m.m_wet_kg; delete m.mr; delete m.propulsion; delete m.isp_s;
      delete m.ve_m_s; delete m.a0_g; delete m.boost_frac;
      changed = true;
    }
    m.payload_kg = Math.max(0, +m.payload_kg || 0);
    m.stages.forEach((s, i) => {
      if (!s.id) { s.id = m.id + "-stage-" + (i + 1); changed = true; }
      if (!s.name) { s.name = "Stage " + (i + 1); changed = true; }
      if (s.jettison == null) { s.jettison = i < m.stages.length - 1; changed = true; }
    });
    if (!Array.isArray(m.default_phases) || !m.default_phases.length) {
      m.default_phases = m.stages.map((s, i) => ({
        stage_id: s.id, prop_frac: 1,
        coast_to_range_m: i === 0 && m.stages.length > 1 ? 25e6 : null,
      }));
      changed = true;
    }
  }
  if ((DB.schema_version || 0) < 2) { DB.schema_version = 2; changed = true; }
  return changed;
}

function migrateRadiatorSchema() {
  const st = S();
  if (!Number.isFinite(st.as_hot_mw_per_kg) || st.as_hot_mw_per_kg <= 0) {
    st.as_hot_mw_per_kg = specificPowerFromAreaDensityMwKg(st.as_hot_t_k, st.as_hot_eps, st.as_hot_kg_m2) || 0.326592;
  }
  if (!Number.isFinite(st.as_low_mw_per_kg) || st.as_low_mw_per_kg <= 0) {
    st.as_low_mw_per_kg = specificPowerFromAreaDensityMwKg(st.as_low_t_k, st.as_low_eps, st.as_low_kg_m2) || 0.0015946875;
  }
  delete st.as_hot_kg_m2;
  delete st.as_low_kg_m2;
}

(async function boot() {
  try {
    const res = await fetch("/api/data");
    DB = await res.json();
  } catch (e) {
    $("main").innerHTML = `<p class="note bad">Could not load fleet.json: ${esc(e.message)}</p>`;
    return;
  }
  setSaveStatus("saved");
  $("save-status").onclick = () => { if (lastSaveError) flushSave(); };
  // Soft migrations for hand-edited or older fleet files.
  const missileMigrated = migrateMissileSchema();
  for (const s of DB.states) if (s.velocity_kms == null) s.velocity_kms = 0;
  migrateRadiatorSchema();
  const designsBefore = JSON.stringify(DB.designs);
  try { await syncAllDesignerMasses(); }
  catch (e) {
    $("main").innerHTML = `<p class="note bad">Could not normalize designer masses: ${esc(e.message)}</p>`;
    return;
  }
  const designerMigrated = designsBefore !== JSON.stringify(DB.designs);
  if (!DB.system) DB.system = { epoch_s: 0, bodies: SOL_BODIES(), nav: {} };
  DB.system.nav = DB.system.nav || {};
  // Jovian moons joined the default system; graft them onto older saves.
  if (DB.system.bodies.some(b => b.id === "jupiter") &&
      !DB.system.bodies.some(b => b.id === "io")) {
    DB.system.bodies.push(...SOL_BODIES().filter(b => b.parent === "jupiter"));
    touch(false); // persist the migration
  }
  const mapDefaults = { g_const: 6.674e-11, c_m_s: 299792458, map_tick_s: 3600,
                        map_substep_s: 60, map_project_d: 10 };
  for (const [k, v] of Object.entries(mapDefaults)) if (S()[k] == null) S()[k] = v;
  if (missileMigrated || designerMigrated) touch(false);
  render();
  window.addEventListener("resize", () => {
    clearTimeout(boot._t);
    boot._t = setTimeout(() => UI.tab === "map" ? drawMap() : render(), 150);
  });
  document.addEventListener("visibilitychange", () => { if (document.hidden) flushSave(); });
  window.addEventListener("beforeunload", e => {
    if (savedRevision < saveRevision) { e.preventDefault(); e.returnValue = ""; }
  });
})();
