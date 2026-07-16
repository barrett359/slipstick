// Verifies that every scenario value and every displayed audit value in a real
// Lidar & PD response has centralized plain-English tooltip metadata.
const fs = require("fs");
const vm = require("vm");

const app = fs.readFileSync("static/app.js", "utf8");
const start = app.indexOf("const LP_MISSING_META");
const end = app.indexOf("function lpPathKey");
if (start < 0 || end < 0) throw new Error("Could not locate Lidar & PD tooltip registry");
const context = {};
vm.runInNewContext(app.slice(start, end) + ";globalThis.__meta = LP_ALL_META;", context);
const meta = context.__meta;

const scenario = JSON.parse(fs.readFileSync("testdata/lidar_pd_baseline.json", "utf8"));
const responsePath = process.argv[2];
const response = responsePath ? JSON.parse(fs.readFileSync(responsePath, "utf8")) : null;
const paths = new Set(["scenario_preset", "detector_preset", "target_preset", "weapon_preset", "geometry_mode"]);

function normalize(path) { return path.replace(/\[\d+\]/g, "[]"); }
function visit(value, path) {
  if (Array.isArray(value)) {
    if (value.length === 0) return;
    if (value.every(item => item == null || typeof item !== "object")) paths.add(normalize(path));
    else value.forEach((item, index) => visit(item, `${path}[${index}]`));
  } else if (value && typeof value === "object") {
    Object.entries(value).forEach(([key, child]) => visit(child, path ? `${path}.${key}` : key));
  } else paths.add(normalize(path));
}

visit(scenario, "");
for (const ignored of ["schema_version", "options.include_disabled_entries", "options.return_intermediates"])
  paths.delete(ignored);
if (response) {
  for (const key of ["summary", "geometry", "signal", "detector", "fire_control", "point_defense"])
    visit(response[key], key);
  response.jammers.forEach((item, index) => visit(item, `jammers[${index}]`));
  response.chaff.forEach((item, index) => visit(item, `chaff[${index}]`));
}

const missing = [...paths].filter(path => !meta[path]);
if (missing.length) {
  console.error("Missing tooltip metadata:\n" + missing.map(path => "- " + path).join("\n"));
  process.exit(1);
}
console.log(`Tooltip metadata covers ${paths.size} scenario and audit value paths.`);
