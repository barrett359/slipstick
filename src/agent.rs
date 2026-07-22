use crate::{combat, model, physics};
use clap::{Args, Subcommand};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const AGENT_SCHEMA_VERSION: &str = "1.0";

#[derive(Debug, Args)]
pub struct AgentArgs {
    /// Persisted fleet document used by snapshot, drafts, evaluation, and simulation.
    #[arg(long, default_value = "data/fleet.json", global = true)]
    pub data: PathBuf,
    /// Root directory for isolated draft workspaces and run artifacts.
    #[arg(long, default_value = "data/agent-workspaces", global = true)]
    pub workspaces: PathBuf,
    #[command(subcommand)]
    pub command: AgentCommand,
}

#[derive(Debug, Subcommand)]
pub enum AgentCommand {
    /// List stable commands, calculators, domains, and output conventions.
    Capabilities,
    /// Return one domain or calculator JSON Schema on demand.
    Schema { domain: String },
    /// Search or retrieve documented field metadata.
    Fields(FieldsArgs),
    /// Read all or a selected JSON Pointer from the live fleet.
    Snapshot {
        #[arg(long)]
        select: Option<String>,
    },
    /// Work with isolated, reversible design workspaces.
    Draft(DraftArgs),
    /// Run one existing Rust calculator with JSON input.
    Calculate {
        calculator: String,
        #[arg(long, default_value = "-")]
        input: PathBuf,
    },
    /// Evaluate one ship design in the live fleet or a draft.
    Evaluate(EvaluateArgs),
    /// Run and query map-backed combat simulations.
    Simulate(SimulateArgs),
}

#[derive(Debug, Args)]
pub struct FieldsArgs {
    #[command(subcommand)]
    pub command: FieldsCommand,
}

#[derive(Debug, Subcommand)]
pub enum FieldsCommand {
    Search { query: String },
    Get { path: String },
}

#[derive(Debug, Args)]
pub struct DraftArgs {
    #[command(subcommand)]
    pub command: DraftCommand,
}

#[derive(Debug, Subcommand)]
pub enum DraftCommand {
    Create {
        #[arg(long)]
        name: String,
    },
    Get {
        id: String,
        #[arg(long)]
        select: Option<String>,
    },
    Patch {
        id: String,
        #[arg(long, default_value = "-")]
        input: PathBuf,
    },
    Validate {
        id: String,
    },
    Diff {
        id: String,
    },
    Rollback {
        id: String,
        #[arg(long, default_value_t = 1)]
        steps: u64,
    },
    Commit {
        id: String,
        /// Entity selector such as designs:battleship, missiles:mh164, settings, or system.
        #[arg(long = "select")]
        selections: Vec<String>,
        /// Apply the previewed merge to the live fleet.
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Debug, Args)]
pub struct EvaluateArgs {
    /// Ship design ID.
    #[arg(long)]
    pub design: String,
    /// Optional draft workspace ID. The live fleet is used when omitted.
    #[arg(long)]
    pub draft: Option<String>,
}

#[derive(Debug, Args)]
pub struct SimulateArgs {
    #[command(subcommand)]
    pub command: SimulateCommand,
}

#[derive(Debug, Subcommand)]
pub enum SimulateCommand {
    /// Pre-flight validation of a combat scenario before running.
    Validate {
        /// Draft workspace containing fleet data.
        #[arg(long)]
        draft: String,
        /// Scenario JSON file or - for stdin.
        #[arg(long, default_value = "-")]
        input: PathBuf,
    },
    /// Generate valid initial_nav coordinates for ships at a given separation.
    Place {
        /// Draft workspace containing fleet data.
        #[arg(long)]
        draft: String,
        /// Comma-separated ship IDs.
        #[arg(long)]
        ships: String,
        /// Desired separation in metres.
        #[arg(long)]
        separation_m: f64,
    },
    /// Generate a complete scenario template from ship IDs and engagement type.
    Template {
        /// Draft workspace containing fleet data.
        #[arg(long)]
        draft: String,
        /// Comma-separated ship IDs, alternating teams (blue, red, blue, red...).
        #[arg(long)]
        ships: String,
        /// Engagement type: duel, ambush, or pursuit.
        #[arg(long, default_value = "duel")]
        engagement: String,
        /// Desired initial separation in metres.
        #[arg(long, default_value_t = 500_000_000.0)]
        separation_m: f64,
    },
    /// Combined run + summary that returns essential data in one call.
    Quick {
        /// Draft workspace containing the designs and scenario.
        #[arg(long)]
        draft: String,
        /// Scenario JSON file or - for stdin.
        #[arg(long, default_value = "-")]
        input: PathBuf,
    },
    Run {
        /// Draft workspace containing the designs and scenario.
        #[arg(long)]
        draft: String,
        /// Scenario JSON file or - for stdin.
        #[arg(long, default_value = "-")]
        input: PathBuf,
    },
    Summary {
        #[arg(long)]
        draft: String,
        #[arg(long)]
        run: String,
    },
    Events {
        #[arg(long)]
        draft: String,
        #[arg(long)]
        run: String,
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Compare {
        #[arg(long)]
        draft: String,
        #[arg(long = "run", required = true)]
        runs: Vec<String>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct Envelope {
    pub schema_version: &'static str,
    pub command: String,
    pub revision: Option<String>,
    pub summary: String,
    pub data: Value,
    pub warnings: Vec<String>,
    pub artifacts: Vec<String>,
}

impl Envelope {
    fn new(command: impl Into<String>, summary: impl Into<String>, data: Value) -> Self {
        Self {
            schema_version: AGENT_SCHEMA_VERSION,
            command: command.into(),
            revision: None,
            summary: summary.into(),
            data,
            warnings: Vec::new(),
            artifacts: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct AgentError {
    pub code: i32,
    pub kind: &'static str,
    pub message: String,
}

impl AgentError {
    fn validation(message: impl Into<String>) -> Self {
        Self {
            code: 2,
            kind: "validation",
            message: message.into(),
        }
    }
    fn conflict(message: impl Into<String>) -> Self {
        Self {
            code: 3,
            kind: "conflict",
            message: message.into(),
        }
    }
    fn missing(message: impl Into<String>) -> Self {
        Self {
            code: 4,
            kind: "not_found",
            message: message.into(),
        }
    }
    fn simulation(message: impl Into<String>) -> Self {
        Self {
            code: 5,
            kind: "simulation",
            message: message.into(),
        }
    }
    fn io(message: impl Into<String>) -> Self {
        Self {
            code: 6,
            kind: "io",
            message: message.into(),
        }
    }

    pub fn envelope(&self) -> Envelope {
        Envelope::new(
            self.kind,
            &self.message,
            json!({ "error": { "kind": self.kind, "message": self.message } }),
        )
    }
}

pub fn execute(args: AgentArgs) -> Result<Envelope, AgentError> {
    match args.command {
        AgentCommand::Capabilities => Ok(capabilities()),
        AgentCommand::Schema { domain } => schema_command(&domain),
        AgentCommand::Fields(fields) => fields_command(fields),
        AgentCommand::Snapshot { select } => snapshot_command(&args.data, select.as_deref()),
        AgentCommand::Draft(draft) => draft_command(&args.data, &args.workspaces, draft),
        AgentCommand::Calculate { calculator, input } => {
            calculate_command(&calculator, &input, &args.workspaces)
        }
        AgentCommand::Evaluate(eval) => evaluate_command(&args.data, &args.workspaces, eval),
        AgentCommand::Simulate(sim) => simulate_command(&args.workspaces, sim),
    }
}

fn capabilities() -> Envelope {
    Envelope::new(
        "capabilities",
        "Slipstick agent CLI capabilities",
        json!({
            "input": "Use --input FILE or --input - for stdin; JSON is never required as a shell argument.",
            "output": "One JSON envelope on stdout. Diagnostics use stderr. Large results are artifact-backed.",
            "exit_codes": {"0":"success", "2":"validation", "3":"revision conflict", "4":"not found", "5":"simulation", "6":"I/O"},
            "commands": ["capabilities", "schema", "fields", "snapshot", "draft", "calculate", "evaluate", "simulate"],
            "simulate_subcommands": ["validate", "place", "template", "quick", "run", "summary", "events", "compare"],
            "domains": ["fleet", "settings", "material", "missile_design", "design", "ship_state", "system", "combat"],
            "calculators": calculator_names(),
        }),
    )
}

fn calculator_names() -> Vec<&'static str> {
    vec![
        "gear",
        "drive_curve",
        "deltav",
        "travel",
        "burn",
        "sprint",
        "autosize",
        "designer",
        "laser",
        "laser_profiles",
        "radiator",
        "vent",
        "missile",
        "missile_optimize",
        "intercept",
        "design_report",
        "nav_tick",
        "orbit_v",
        "burn_for_dv",
        "nav_intercept",
        "lidar_pd",
        "missile_engagement",
    ]
}

fn schema_command(domain: &str) -> Result<Envelope, AgentError> {
    let schema = domain_schema(domain).ok_or_else(|| {
        AgentError::missing(format!("unknown schema domain or calculator {domain}"))
    })?;
    Ok(Envelope::new(
        format!("schema {domain}"),
        format!("JSON Schema for {domain}"),
        schema,
    ))
}

fn schema_pair<I: JsonSchema, O: JsonSchema>() -> Value {
    json!({ "input": documented_schema::<I>(), "output": documented_schema::<O>() })
}

fn documented_schema<T: JsonSchema>() -> Value {
    let mut schema = json!(schema_for!(T));
    enrich_schema(&mut schema);
    if let Some(object) = schema.as_object_mut() {
        object.insert(
            "x-assumptions".into(),
            json!([
                "SI units are used unless x-unit states otherwise.",
                "Null means the value is intentionally unavailable or not selected.",
                "Calculator results are deterministic for identical inputs."
            ]),
        );
    }
    schema
}

fn enrich_schema(schema: &mut Value) {
    let Some(object) = schema.as_object_mut() else {
        return;
    };
    if let Some(properties) = object.get_mut("properties").and_then(Value::as_object_mut) {
        for (name, child) in properties {
            if let Some(child_object) = child.as_object_mut() {
                let unit = field_unit(name).or_else(|| fallback_numeric_unit(name, child_object));
                child_object.entry("description").or_insert_with(|| {
                    let words = name.replace('_', " ");
                    match unit {
                        Some(unit) => json!(format!("{words}. Unit: {unit}.")),
                        None => json!(format!("{words}.")),
                    }
                });
                if let Some(unit) = unit {
                    child_object.entry("x-unit").or_insert(json!(unit));
                }
                add_inferred_range(name, child_object);
            }
            enrich_schema(child);
        }
    }
    if let Some(items) = object.get_mut("items") {
        enrich_schema(items);
    }
    if let Some(defs) = object.get_mut("$defs").and_then(Value::as_object_mut) {
        for definition in defs.values_mut() {
            enrich_schema(definition);
        }
    }
    for keyword in ["anyOf", "oneOf", "allOf"] {
        if let Some(branches) = object.get_mut(keyword).and_then(Value::as_array_mut) {
            for branch in branches {
                enrich_schema(branch);
            }
        }
    }
}

fn field_unit(name: &str) -> Option<&'static str> {
    match name {
        "mw_per_kg" => return Some("MW/kg"),
        "kg_per_m2" => return Some("kg/m^2"),
        "mw_per_m2" => return Some("MW/m^2"),
        "tonnes_per_crew" => return Some("t/person"),
        "target_accel_mg" => return Some("mg"),
        _ => {}
    }
    let suffixes = [
        ("_mj_per_kg", "MJ/kg"),
        ("_mj_per_t", "MJ/t"),
        ("_mw_per_kg", "MW/kg"),
        ("_t_per_tw", "t/TW"),
        ("_t_per_mn", "t/MN"),
        ("_m_s2", "m/s^2"),
        ("_mm_s", "mm/s"),
        ("_km_s", "km/s"),
        ("_m_s", "m/s"),
        ("_w_m2", "W/m^2"),
        ("_j_m2", "J/m^2"),
        ("_kg_s", "kg/s"),
        ("_kg_m3", "kg/m^3"),
        ("_m2", "m^2"),
        ("_kg", "kg"),
        ("_mj", "MJ"),
        ("_tw", "TW"),
        ("_mw", "MW"),
        ("_w", "W"),
        ("_j", "J"),
        ("_kms", "km/s"),
        ("_s", "s"),
        ("_min", "min"),
        ("_d", "day"),
        ("_mm", "mm"),
        ("_rad", "rad"),
        ("_deg", "deg"),
        ("_hz", "Hz"),
        ("_n", "N"),
        ("_k", "K"),
        ("_t", "t"),
        ("_m", "m"),
        ("_pct", "%"),
    ];
    if name == "rho" {
        return Some("kg/m^3");
    }
    if name == "m2" {
        return Some("dimensionless");
    }
    if [
        "fraction",
        "factor",
        "ratio",
        "probability",
        "efficiency",
        "snr",
        "eta",
        "eps",
        "mr",
    ]
    .iter()
    .any(|part| name == *part || name.ends_with(&format!("_{part}")))
        || name.contains("_frac")
        || name.contains("_overlap")
    {
        return Some("dimensionless");
    }
    suffixes
        .iter()
        .find_map(|(suffix, unit)| name.ends_with(suffix).then_some(*unit))
}

fn fallback_numeric_unit(
    name: &str,
    schema: &serde_json::Map<String, Value>,
) -> Option<&'static str> {
    let numeric = matches!(
        schema.get("type").and_then(Value::as_str),
        Some("number" | "integer")
    ) || schema
        .get("type")
        .and_then(Value::as_array)
        .is_some_and(|types| {
            types
                .iter()
                .any(|kind| kind == "number" || kind == "integer")
        });
    if !numeric {
        return None;
    }
    if name.contains("power") || name.starts_with("p_") || name == "p" {
        Some("W")
    } else if name.contains("thrust") || name.starts_with("f_") {
        Some("N")
    } else if name.starts_with('v') || name.starts_with("dv") || name.contains("velocity") {
        Some("m/s")
    } else if name.starts_with('m') && !name.starts_with("max") || name.contains("mass") {
        Some("kg")
    } else if name.starts_with('t') || name.contains("duration") || name.contains("time") {
        Some("s")
    } else if name.contains("range") || name.contains("distance") || matches!(name, "x" | "y") {
        Some("m")
    } else if matches!(schema.get("type").and_then(Value::as_str), Some("integer")) {
        Some("count")
    } else {
        Some("dimensionless")
    }
}

fn add_inferred_range(name: &str, schema: &mut serde_json::Map<String, Value>) {
    let bounded_fraction = name.contains("fraction")
        || name.contains("_frac")
        || name.contains("probability")
        || name.contains("efficiency")
        || name.contains("_overlap")
        || matches!(
            name,
            "eta" | "eps" | "exposure" | "vulnerability" | "duty_cycle"
        );
    if bounded_fraction {
        schema.entry("minimum").or_insert(json!(0.0));
        schema.entry("maximum").or_insert(json!(1.0));
    } else if name.ends_with("_pct") {
        schema.entry("minimum").or_insert(json!(0.0));
        schema.entry("maximum").or_insert(json!(100.0));
    }
}

fn domain_schema(domain: &str) -> Option<Value> {
    let value = match domain {
        "fleet" => documented_schema::<model::FleetDocument>(),
        "settings" => documented_schema::<model::Settings>(),
        "material" | "materials" => documented_schema::<model::Material>(),
        "missile_design" => documented_schema::<model::Missile>(),
        "design" | "ship_design" => documented_schema::<model::Design>(),
        "ship_state" => documented_schema::<model::ShipState>(),
        "system" | "map" => documented_schema::<model::SystemState>(),
        "combat" | "combat_scenario" => documented_schema::<combat::CombatScenario>(),
        "gear" => schema_pair::<physics::GearIn, physics::GearOut>(),
        "drive_curve" => schema_pair::<physics::DriveCurveIn, physics::DriveCurveOut>(),
        "deltav" => schema_pair::<physics::DeltavIn, physics::DeltavOut>(),
        "travel" => schema_pair::<physics::TravelIn, physics::TravelOut>(),
        "burn" => schema_pair::<physics::BurnIn, physics::BurnOut>(),
        "sprint" => schema_pair::<physics::SprintIn, physics::SprintOut>(),
        "autosize" => schema_pair::<physics::AutosizeIn, physics::AutosizeOut>(),
        "designer" => schema_pair::<physics::DesignerIn, physics::DesignerOut>(),
        "laser" => schema_pair::<physics::LaserIn, physics::LaserOut>(),
        "laser_profiles" => schema_pair::<physics::LaserProfilesIn, physics::LaserProfilesOut>(),
        "radiator" => schema_pair::<physics::RadiatorIn, physics::RadiatorOut>(),
        "vent" => schema_pair::<physics::VentIn, physics::VentOut>(),
        "missile" => schema_pair::<physics::MissileIn, physics::MissileOut>(),
        "missile_optimize" => {
            schema_pair::<physics::MissileOptimizeIn, physics::MissileOptimizeOut>()
        }
        "intercept" => schema_pair::<physics::InterceptIn, physics::InterceptOut>(),
        "design_report" => schema_pair::<physics::ReportIn, physics::ReportOut>(),
        "nav_tick" => schema_pair::<physics::NavTickIn, physics::NavTickOut>(),
        "orbit_v" => schema_pair::<physics::OrbitVIn, physics::OrbitVOut>(),
        "burn_for_dv" => schema_pair::<physics::BurnForDvIn, physics::BurnForDvOut>(),
        "nav_intercept" => schema_pair::<physics::NavInterceptIn, physics::NavInterceptOut>(),
        "lidar_pd" => schema_pair::<physics::LidarPdIn, physics::LidarPdOut>(),
        "missile_engagement" => {
            schema_pair::<physics::MissileEngagementIn, physics::MissileEngagementOut>()
        }
        _ => return None,
    };
    Some(value)
}

fn fields_command(args: FieldsArgs) -> Result<Envelope, AgentError> {
    let mut fields = Vec::new();
    collect_schema_fields(
        &documented_schema::<model::FleetDocument>(),
        "",
        &mut fields,
    );
    collect_schema_fields(
        &documented_schema::<combat::CombatScenario>(),
        "combat",
        &mut fields,
    );
    for calculator in calculator_names() {
        if let Some(schema) = domain_schema(calculator) {
            collect_schema_fields(&schema, calculator, &mut fields);
        }
    }
    match args.command {
        FieldsCommand::Search { query } => {
            let q = query.to_lowercase();
            let found: Vec<_> = fields
                .into_iter()
                .filter(|f| {
                    f.get("path")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
                        || f.get("description")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&q)
                })
                .collect();
            Ok(Envelope::new(
                "fields search",
                format!("{} matching fields", found.len()),
                json!(found),
            ))
        }
        FieldsCommand::Get { path } => {
            let normalized = path.trim_start_matches('/').replace('/', ".");
            let found = fields
                .into_iter()
                .find(|f| f["path"] == normalized)
                .ok_or_else(|| AgentError::missing(format!("unknown documented field {path}")))?;
            Ok(Envelope::new(
                "fields get",
                format!("Field {normalized}"),
                found,
            ))
        }
    }
}

fn collect_schema_fields(schema: &Value, prefix: &str, out: &mut Vec<Value>) {
    let Some(obj) = schema.as_object() else {
        return;
    };
    if let Some(properties) = obj.get("properties").and_then(Value::as_object) {
        for (name, child) in properties {
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}.{name}")
            };
            out.push(json!({
                "path": path,
                "description": child.get("description").and_then(Value::as_str).unwrap_or(""),
                "type": child.get("type").cloned().unwrap_or(Value::Null),
                "unit": child.get("x-unit").cloned().unwrap_or(Value::Null),
                "minimum": child.get("minimum").cloned().unwrap_or(Value::Null),
                "maximum": child.get("maximum").cloned().unwrap_or(Value::Null),
                "required": obj.get("required").and_then(Value::as_array).is_some_and(|r| r.iter().any(|x| x == name)),
            }));
            collect_schema_fields(child, &path, out);
            if let Some(items) = child.get("items") {
                collect_schema_fields(items, &format!("{path}[]"), out);
            }
        }
    }
    if let Some(defs) = obj.get("$defs").and_then(Value::as_object) {
        for (name, child) in defs {
            collect_schema_fields(child, &format!("{prefix}<{name}>"), out);
        }
    }
    for key in ["input", "output"] {
        if let Some(child) = obj.get(key) {
            let path = if prefix.is_empty() {
                key.into()
            } else {
                format!("{prefix}.{key}")
            };
            collect_schema_fields(child, &path, out);
        }
    }
}

fn snapshot_command(data_path: &Path, select: Option<&str>) -> Result<Envelope, AgentError> {
    let (bytes, value, _) = read_fleet(data_path)?;
    let selected = match select {
        Some(pointer) => value
            .pointer(pointer)
            .cloned()
            .ok_or_else(|| AgentError::missing(format!("JSON Pointer {pointer} not found")))?,
        None => value,
    };
    let mut out = Envelope::new("snapshot", "Fleet snapshot", selected);
    out.revision = Some(model::revision(&bytes));
    Ok(out)
}

fn calculate_command(
    calculator: &str,
    input: &Path,
    workspaces: &Path,
) -> Result<Envelope, AgentError> {
    let value = read_json_input(input)?;
    let result = dispatch_calculation(calculator, value)?;
    let encoded = serde_json::to_vec(&result).map_err(|error| AgentError::io(error.to_string()))?;
    let mut out = Envelope::new(
        format!("calculate {calculator}"),
        format!("{calculator} calculation complete"),
        result.clone(),
    );
    if encoded.len() > 32 * 1024 {
        let artifacts = workspaces.join("artifacts");
        fs::create_dir_all(&artifacts).map_err(|error| AgentError::io(error.to_string()))?;
        let path = artifacts.join(format!(
            "{}-{}-{}.json",
            slug(calculator),
            unix_seconds(),
            &model::revision(&encoded)[..12]
        ));
        write_json_atomic(&path, &result)?;
        out.data = compact_value(&result, 0);
        out.summary = format!(
            "{calculator} calculation complete; full {} byte result written as an artifact",
            encoded.len()
        );
        out.artifacts.push(path.display().to_string());
    }
    Ok(out)
}

fn compact_value(value: &Value, depth: usize) -> Value {
    match value {
        Value::Array(values) if values.len() > 12 => json!({
            "kind": "array_summary",
            "length": values.len(),
            "first": values.first().map(|value| compact_value(value, depth + 1)),
            "last": values.last().map(|value| compact_value(value, depth + 1)),
        }),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| compact_value(value, depth + 1))
                .collect(),
        ),
        Value::Object(object) if depth < 8 => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), compact_value(value, depth + 1)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

macro_rules! calculate {
    ($value:expr, $input:ty, $func:path) => {{
        let input: $input = serde_json::from_value($value)
            .map_err(|e| AgentError::validation(format!("calculator input: {e}")))?;
        let output = $func(&input).map_err(AgentError::validation)?;
        serde_json::to_value(output).map_err(|e| AgentError::io(e.to_string()))
    }};
}

pub(crate) fn dispatch_calculation(name: &str, value: Value) -> Result<Value, AgentError> {
    match name {
        "gear" => calculate!(value, physics::GearIn, physics::gear),
        "drive_curve" => calculate!(value, physics::DriveCurveIn, physics::drive_curve),
        "deltav" => calculate!(value, physics::DeltavIn, physics::deltav),
        "travel" => calculate!(value, physics::TravelIn, physics::travel),
        "burn" => calculate!(value, physics::BurnIn, physics::timed_burn),
        "sprint" => calculate!(value, physics::SprintIn, physics::sprint),
        "autosize" => calculate!(value, physics::AutosizeIn, physics::autosize),
        "designer" => calculate!(value, physics::DesignerIn, physics::designer),
        "laser" => calculate!(value, physics::LaserIn, physics::laser),
        "laser_profiles" => calculate!(value, physics::LaserProfilesIn, physics::laser_profiles),
        "radiator" => calculate!(value, physics::RadiatorIn, physics::radiator),
        "vent" => calculate!(value, physics::VentIn, physics::vent),
        "missile" => calculate!(value, physics::MissileIn, physics::missile),
        "missile_optimize" => {
            calculate!(value, physics::MissileOptimizeIn, physics::optimize_missile)
        }
        "intercept" => calculate!(value, physics::InterceptIn, physics::intercept),
        "design_report" => calculate!(value, physics::ReportIn, physics::design_report),
        "nav_tick" => calculate!(value, physics::NavTickIn, physics::nav_tick),
        "orbit_v" => calculate!(value, physics::OrbitVIn, physics::orbit_v),
        "burn_for_dv" => calculate!(value, physics::BurnForDvIn, physics::burn_for_dv),
        "nav_intercept" => calculate!(value, physics::NavInterceptIn, physics::nav_intercept),
        "lidar_pd" => calculate!(value, physics::LidarPdIn, physics::lidar_pd),
        "missile_engagement" => calculate!(
            value,
            physics::MissileEngagementIn,
            physics::missile_engagement
        ),
        _ => Err(AgentError::missing(format!("unknown calculator {name}"))),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DraftManifest {
    id: String,
    name: String,
    created_unix_s: u64,
    base_revision: String,
    current_revision: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PatchOperation {
    op: String,
    path: String,
    #[serde(default)]
    value: Value,
}

fn draft_command(data_path: &Path, root: &Path, args: DraftArgs) -> Result<Envelope, AgentError> {
    match args.command {
        DraftCommand::Create { name } => draft_create(data_path, root, &name),
        DraftCommand::Get { id, select } => draft_get(root, &id, select.as_deref()),
        DraftCommand::Patch { id, input } => draft_patch(root, &id, &input),
        DraftCommand::Validate { id } => draft_validate(root, &id),
        DraftCommand::Diff { id } => draft_diff(data_path, root, &id),
        DraftCommand::Rollback { id, steps } => draft_rollback(root, &id, steps),
        DraftCommand::Commit {
            id,
            selections,
            apply,
        } => draft_commit(data_path, root, &id, &selections, apply),
    }
}

fn draft_create(data_path: &Path, root: &Path, name: &str) -> Result<Envelope, AgentError> {
    let (bytes, value, _) = read_fleet(data_path)?;
    let now = unix_seconds();
    let id = format!("{}-{now}", slug(name));
    let dir = root.join(&id);
    if dir.exists() {
        return Err(AgentError::conflict(format!("draft {id} already exists")));
    }
    fs::create_dir_all(dir.join("history")).map_err(|e| AgentError::io(e.to_string()))?;
    fs::create_dir_all(dir.join("scenarios")).map_err(|e| AgentError::io(e.to_string()))?;
    fs::create_dir_all(dir.join("runs")).map_err(|e| AgentError::io(e.to_string()))?;
    let manifest = DraftManifest {
        id: id.clone(),
        name: name.into(),
        created_unix_s: now,
        base_revision: model::revision(&bytes),
        current_revision: 0,
    };
    write_json_atomic(
        &dir.join("manifest.json"),
        &serde_json::to_value(&manifest).unwrap(),
    )?;
    write_json_atomic(&dir.join("fleet.json"), &value)?;
    fs::write(dir.join("operations.jsonl"), "").map_err(|e| AgentError::io(e.to_string()))?;
    let mut out = Envelope::new(
        "draft create",
        format!("Created isolated draft {id}"),
        json!({"id": id, "name": name, "path": dir}),
    );
    out.revision = Some(manifest.base_revision);
    out.artifacts.push(dir.display().to_string());
    Ok(out)
}

fn draft_get(root: &Path, id: &str, select: Option<&str>) -> Result<Envelope, AgentError> {
    let dir = draft_dir(root, id)?;
    let value = read_json_file(&dir.join("fleet.json"))?;
    let manifest = read_manifest(&dir)?;
    let selected = match select {
        Some(pointer) => value
            .pointer(pointer)
            .cloned()
            .ok_or_else(|| AgentError::missing(format!("JSON Pointer {pointer} not found")))?,
        None => value,
    };
    let mut out = Envelope::new(
        "draft get",
        format!("Draft {id} revision {}", manifest.current_revision),
        selected,
    );
    out.revision = Some(format!("draft:{}", manifest.current_revision));
    Ok(out)
}

fn draft_patch(root: &Path, id: &str, input: &Path) -> Result<Envelope, AgentError> {
    let dir = draft_dir(root, id)?;
    let mut manifest = read_manifest(&dir)?;
    let mut value = read_json_file(&dir.join("fleet.json"))?;
    let patch_value = read_json_input(input)?;
    let operations_value = patch_value.get("patch").cloned().unwrap_or(patch_value);
    let operations: Vec<PatchOperation> =
        serde_json::from_value(operations_value).map_err(|e| {
            AgentError::validation(format!("patch must be an RFC 6902 operation array: {e}"))
        })?;
    if operations.is_empty() {
        return Err(AgentError::validation("patch contains no operations"));
    }
    let before = value.clone();
    for operation in &operations {
        apply_patch_operation(&mut value, operation)?;
    }
    let history = dir
        .join("history")
        .join(format!("{}.json", manifest.current_revision));
    write_json_atomic(&history, &before)?;
    write_json_atomic(&dir.join("fleet.json"), &value)?;
    manifest.current_revision += 1;
    write_json_atomic(
        &dir.join("manifest.json"),
        &serde_json::to_value(&manifest).unwrap(),
    )?;
    append_json_line(
        &dir.join("operations.jsonl"),
        &json!({
            "revision": manifest.current_revision,
            "unix_s": unix_seconds(),
            "operations": operations,
        }),
    )?;
    let changes = diff_paths(&before, &value, "", 200);
    let mut out = Envelope::new(
        "draft patch",
        format!("Applied {} operations to draft {id}", operations.len()),
        json!({"id": id, "draft_revision": manifest.current_revision, "changes": changes}),
    );
    out.revision = Some(format!("draft:{}", manifest.current_revision));
    Ok(out)
}

fn draft_validate(root: &Path, id: &str) -> Result<Envelope, AgentError> {
    let dir = draft_dir(root, id)?;
    let value = read_json_file(&dir.join("fleet.json"))?;
    let fleet = match model::FleetDocument::from_value(value) {
        Ok(fleet) => fleet,
        Err(error) => {
            return Ok(Envelope::new(
                "draft validate",
                format!("Draft {id} is not schema-valid"),
                json!({"valid": false, "errors": [{"path": "", "kind": "schema", "message": error}], "design_evaluations": []}),
            ));
        }
    };
    let errors = fleet
        .validate()
        .into_iter()
        .map(|message| {
            let prefix = message.split(':').next().unwrap_or("");
            let path = if prefix == "schema_version" || prefix.starts_with("settings.") {
                format!("/{}", prefix.replace('.', "/"))
            } else if prefix.starts_with("system") {
                "/system".into()
            } else if prefix.starts_with("material ") {
                "/materials".into()
            } else if prefix.starts_with("missile ") {
                "/missiles".into()
            } else if prefix.starts_with("design ") {
                "/designs".into()
            } else if prefix.starts_with("ship state ") {
                "/states".into()
            } else {
                "/".into()
            };
            json!({"path": path, "kind": "domain", "message": message})
        })
        .collect::<Vec<_>>();
    let mut evaluations = Vec::new();
    let mut warnings = Vec::new();
    for (field, actual, canon) in [
        ("f_exh", fleet.settings.f_exh, 0.753),
        ("eta_noz", fleet.settings.eta_noz, 0.85),
        ("ve_max_m_s", fleet.settings.ve_max_m_s, 2_300_000.0),
        ("prop_mh_ve_m_s", fleet.settings.prop_mh_ve_m_s, 16_700.0),
    ] {
        if (actual - canon).abs() > canon.abs().max(1.0) * 1e-9 {
            warnings.push(format!(
                "/settings/{field}: value {actual} deviates from bundled canon {canon}"
            ));
        }
    }
    for design in &fleet.designs {
        match evaluate_design(&fleet, &design.id) {
            Ok(result) => evaluations.push(compact_design_evaluation(&result)),
            Err(error) => warnings.push(format!("design {}: {}", design.id, error.message)),
        }
    }
    let invalid_designs = evaluations
        .iter()
        .filter(|evaluation| evaluation["credible"] == false)
        .count();
    let valid = errors.is_empty() && invalid_designs == 0;
    let mut out = Envelope::new(
        "draft validate",
        if valid {
            format!("Draft {id} is valid")
        } else {
            format!(
                "Draft {id} has {} domain errors and {invalid_designs} non-credible designs",
                errors.len()
            )
        },
        json!({"valid": valid, "errors": errors, "design_evaluations": evaluations}),
    );
    out.warnings = warnings;
    Ok(out)
}

fn compact_design_evaluation(evaluation: &Value) -> Value {
    let laser_ranges = evaluation["weapon_performance"]["lasers"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|laser| laser["profiles"].as_array().into_iter().flatten())
        .filter_map(|profile| profile["kill_range_m"].as_f64())
        .collect::<Vec<_>>();
    let missile_delta_v = evaluation["weapon_performance"]["missiles"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|missile| missile["delta_v_m_s"].as_f64())
        .collect::<Vec<_>>();
    json!({
        "design_id": evaluation["design_id"],
        "credible": evaluation["credible"],
        "mass": evaluation["mass"],
        "mission_performance": {
            "acceleration_wet_m_s2": evaluation["report"]["accel_wet"],
            "acceleration_dry_m_s2": evaluation["report"]["accel_dry"],
            "delta_v_m_s": evaluation["report"]["dv_plasma"],
            "hot_radiator_margin_w": evaluation["report"]["hot_margin_w"],
            "sink_endurance_s": evaluation["report"]["sink_endurance_s"],
            "maximum_laser_kill_range_m": laser_ranges.into_iter().reduce(f64::max),
            "maximum_missile_delta_v_m_s": missile_delta_v.into_iter().reduce(f64::max),
        },
        "issues": evaluation["issues"],
    })
}

fn draft_diff(data_path: &Path, root: &Path, id: &str) -> Result<Envelope, AgentError> {
    let (_, live, _) = read_fleet(data_path)?;
    let dir = draft_dir(root, id)?;
    let draft = read_json_file(&dir.join("fleet.json"))?;
    let changes = diff_paths(&live, &draft, "", 500);
    Ok(Envelope::new(
        "draft diff",
        format!("Draft {id} differs at {} paths", changes.len()),
        json!({"id": id, "changes": changes}),
    ))
}

fn draft_rollback(root: &Path, id: &str, steps: u64) -> Result<Envelope, AgentError> {
    if steps == 0 {
        return Err(AgentError::validation("steps must be positive"));
    }
    let dir = draft_dir(root, id)?;
    let mut manifest = read_manifest(&dir)?;
    if steps > manifest.current_revision {
        return Err(AgentError::validation(format!(
            "cannot roll back {steps} steps from revision {}",
            manifest.current_revision
        )));
    }
    let target = manifest.current_revision - steps;
    let snapshot = read_json_file(&dir.join("history").join(format!("{target}.json")))?;
    let current = read_json_file(&dir.join("fleet.json"))?;
    write_json_atomic(
        &dir.join("history")
            .join(format!("{}.json", manifest.current_revision)),
        &current,
    )?;
    write_json_atomic(&dir.join("fleet.json"), &snapshot)?;
    manifest.current_revision = target;
    write_json_atomic(
        &dir.join("manifest.json"),
        &serde_json::to_value(&manifest).unwrap(),
    )?;
    append_json_line(
        &dir.join("operations.jsonl"),
        &json!({
            "revision": target, "unix_s": unix_seconds(), "rollback_steps": steps
        }),
    )?;
    Ok(Envelope::new(
        "draft rollback",
        format!("Rolled draft {id} back to revision {target}"),
        json!({"id": id, "draft_revision": target}),
    ))
}

fn draft_commit(
    data_path: &Path,
    root: &Path,
    id: &str,
    selections: &[String],
    apply: bool,
) -> Result<Envelope, AgentError> {
    if selections.is_empty() {
        return Err(AgentError::validation(
            "draft commit requires at least one --select entity",
        ));
    }
    let dir = draft_dir(root, id)?;
    let mut manifest = read_manifest(&dir)?;
    let (live_bytes, live, _) = read_fleet(data_path)?;
    let live_revision = model::revision(&live_bytes);
    if live_revision != manifest.base_revision {
        return Err(AgentError::conflict(format!(
            "live fleet changed: draft base {} but current {}; create or rebase a draft",
            manifest.base_revision, live_revision
        )));
    }
    let draft = read_json_file(&dir.join("fleet.json"))?;
    let fleet_selections = selections
        .iter()
        .filter(|selection| !selection.starts_with("scenarios:"))
        .cloned()
        .collect::<Vec<_>>();
    let scenario_selections = selections
        .iter()
        .filter_map(|selection| selection.strip_prefix("scenarios:"))
        .collect::<Vec<_>>();
    let mut scenario_copies = Vec::new();
    for scenario_id in scenario_selections {
        if !safe_identifier(scenario_id) {
            return Err(AgentError::validation(format!(
                "invalid scenario selector {scenario_id}"
            )));
        }
        let source = dir.join("scenarios").join(format!("{scenario_id}.json"));
        if !source.is_file() {
            return Err(AgentError::missing(format!(
                "draft scenario {scenario_id} not found"
            )));
        }
        let destination = data_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("scenarios")
            .join(format!("{scenario_id}.json"));
        scenario_copies.push((source, destination));
    }
    let merged = if fleet_selections.is_empty() {
        live.clone()
    } else {
        merge_selections(live.clone(), &draft, &fleet_selections)?
    };
    let typed = model::FleetDocument::from_value(merged.clone()).map_err(AgentError::validation)?;
    let errors = typed.validate();
    if !errors.is_empty() {
        return Err(AgentError::validation(errors.join("; ")));
    }
    for (source, _) in &scenario_copies {
        let scenario: combat::CombatScenario = serde_json::from_value(read_json_file(source)?)
            .map_err(|error| AgentError::validation(format!("scenario schema: {error}")))?;
        let errors = combat::validate_scenario(&typed, &scenario);
        if !errors.is_empty() {
            return Err(AgentError::validation(format!(
                "scenario {}: {}",
                scenario.name,
                errors.join("; ")
            )));
        }
    }
    let changes = diff_paths(&live, &merged, "", 500);
    let mut out = Envelope::new(
        "draft commit",
        if apply {
            format!("Committed draft {id}")
        } else {
            format!("Previewed commit for draft {id}; rerun with --apply")
        },
        json!({"id": id, "applied": apply, "selections": selections, "changes": changes}),
    );
    if apply {
        write_json_atomic(data_path, &merged)?;
        for (source, destination) in &scenario_copies {
            write_json_atomic(destination, &read_json_file(source)?)?;
            out.artifacts.push(destination.display().to_string());
        }
        let bytes = fs::read(data_path).map_err(|e| AgentError::io(e.to_string()))?;
        manifest.base_revision = model::revision(&bytes);
        write_json_atomic(
            &dir.join("manifest.json"),
            &serde_json::to_value(&manifest).unwrap(),
        )?;
        out.revision = Some(manifest.base_revision);
        append_json_line(
            &dir.join("operations.jsonl"),
            &json!({
                "unix_s": unix_seconds(),
                "commit": {"selections": selections, "live_revision": out.revision}
            }),
        )?;
    } else {
        out.revision = Some(live_revision);
    }
    Ok(out)
}

fn merge_selections(
    mut live: Value,
    draft: &Value,
    selections: &[String],
) -> Result<Value, AgentError> {
    for selection in selections {
        if selection == "settings" || selection == "system" {
            live[selection] = draft
                .get(selection)
                .cloned()
                .ok_or_else(|| AgentError::missing(format!("draft has no {selection}")))?;
            continue;
        }
        if !selection.contains(':') {
            live[selection] = draft
                .get(selection)
                .cloned()
                .ok_or_else(|| AgentError::missing(format!("draft has no {selection}")))?;
            continue;
        }
        let (collection, id) = selection.split_once(':').unwrap();
        let draft_items = draft
            .get(collection)
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AgentError::missing(format!("draft collection {collection} not found"))
            })?;
        let item = draft_items
            .iter()
            .find(|x| x.get("id").and_then(Value::as_str) == Some(id))
            .cloned()
            .ok_or_else(|| AgentError::missing(format!("draft {collection}:{id} not found")))?;
        let live_items = live
            .get_mut(collection)
            .and_then(Value::as_array_mut)
            .ok_or_else(|| {
                AgentError::missing(format!("live collection {collection} not found"))
            })?;
        if let Some(index) = live_items
            .iter()
            .position(|x| x.get("id").and_then(Value::as_str) == Some(id))
        {
            live_items[index] = item;
        } else {
            live_items.push(item);
        }
    }
    Ok(live)
}

fn evaluate_command(
    data_path: &Path,
    root: &Path,
    args: EvaluateArgs,
) -> Result<Envelope, AgentError> {
    let fleet = if let Some(id) = args.draft {
        let dir = draft_dir(root, &id)?;
        model::FleetDocument::from_value(read_json_file(&dir.join("fleet.json"))?)
            .map_err(AgentError::validation)?
    } else {
        read_fleet(data_path)?.2
    };
    let result = evaluate_design(&fleet, &args.design)?;
    Ok(Envelope::new(
        "evaluate design",
        format!("Evaluated design {}", args.design),
        result,
    ))
}

fn evaluate_design(fleet: &model::FleetDocument, design_id: &str) -> Result<Value, AgentError> {
    let (design_index, design) = fleet
        .designs
        .iter()
        .enumerate()
        .find(|(_, design)| design.id == design_id)
        .ok_or_else(|| AgentError::missing(format!("design {design_id} not found")))?;
    let mut component_mass_t = 0.0;
    let mut ordnance_t = 0.0;
    let mut tank_capacity_t = 0.0;
    let mut p_fusion = 0.0;
    let mut rad_load_frac = 0.0;
    let mut f_cap = 0.0;
    let mut sink_mj = 0.0;
    let mut flywheel_mj = 0.0;
    let mut rad_hot = Vec::new();
    let mut rad_low = Vec::new();
    let mut lasers = Vec::new();
    let mut issues = Vec::<Value>::new();
    for c in &design.components {
        let count = if c.kind == "laser" {
            c.count.unwrap_or(1) as f64
        } else {
            1.0
        };
        let mass = if let Some(mass) = c.mass_t {
            mass
        } else if c.kind == "radiator_hot" || c.kind == "radiator_low" {
            let power = c.area_m2.unwrap_or(0.0)
                * c.eps.unwrap_or(0.0)
                * fleet.settings.sigma
                * c.t_k.unwrap_or(0.0).powi(4);
            power / (c.mw_per_kg.unwrap_or(1.0) * 1e9)
        } else {
            c.mass_t.unwrap_or(0.0)
        };
        component_mass_t += mass * count;
        match c.kind.as_str() {
            "reactor" => {
                p_fusion += c.p_fusion_w.unwrap_or(0.0);
                rad_load_frac = c.rad_load_frac.unwrap_or(rad_load_frac);
            }
            "nozzle" => f_cap += c.f_max_n.unwrap_or(0.0),
            "radiator_hot" | "radiator_low" => {
                let spec = physics::RadiatorSpec {
                    area: c.area_m2.unwrap_or(0.0),
                    t_k: c.t_k.unwrap_or(0.0),
                    eps: c.eps.unwrap_or(0.0),
                    integrity_pct: 100.0,
                };
                if c.kind == "radiator_hot" {
                    rad_hot.push(spec);
                } else {
                    rad_low.push(spec);
                }
            }
            "heat_sink" => {
                sink_mj += c.li_t.unwrap_or(0.0)
                    * 1000.0
                    * c.energy_mj_per_kg
                        .unwrap_or(fleet.settings.li_sink_mj_per_kg)
            }
            "flywheel" => {
                flywheel_mj += c.mass_t.unwrap_or(0.0)
                    * c.energy_mj_per_kg
                        .filter(|value| *value > 0.0)
                        .map(|value| value * 1000.0)
                        .unwrap_or(fleet.settings.flywheel_mj_per_t)
            }
            "laser" => {
                for _ in 0..c.count.unwrap_or(1) {
                    lasers.push(physics::LaserSpec {
                        p_beam: c.p_beam_w.unwrap_or(0.0),
                        eta_wall: c.eta_wall.unwrap_or(fleet.settings.laser_eta_wall),
                        t_pulse: c.t_pulse_s.unwrap_or(fleet.settings.pulse_ship_s),
                    });
                }
            }
            "magazine" => {
                if let Some(missile) = c
                    .missile_id
                    .as_deref()
                    .and_then(|id| fleet.missiles.iter().find(|m| m.id == id))
                {
                    let wet = missile.payload_kg
                        + missile
                            .stages
                            .iter()
                            .map(|s| s.dry_mass_kg + s.propellant_kg)
                            .sum::<f64>();
                    ordnance_t += wet / 1000.0 * c.capacity.unwrap_or(0) as f64;
                }
            }
            "tank" => {
                tank_capacity_t += c.mass_t.unwrap_or(0.0)
                    / c.tank_structure_frac
                        .filter(|value| *value > 0.0)
                        .unwrap_or(1.0 / fleet.settings.tank_prop_per_mass)
            }
            _ => {}
        }
    }
    let dry_t = component_mass_t + ordnance_t + design.structure_t;
    let wet_t = dry_t * design.mr;
    let propellant_t = wet_t - dry_t;
    if tank_capacity_t + 1e-6 < propellant_t {
        issues.push(issue(
            "error",
            "/designs/*/components",
            format!(
                "tank capacity {:.3} t is below required propellant {:.3} t",
                tank_capacity_t, propellant_t
            ),
        ));
    }
    if p_fusion <= 0.0 {
        issues.push(issue(
            "error",
            "/designs/*/components",
            "design has no positive fusion power",
        ));
    }
    if f_cap <= 0.0 {
        issues.push(issue(
            "warning",
            "/designs/*/components",
            "design has no nozzle thrust cap",
        ));
    }
    let report = if p_fusion > 0.0 && dry_t > 0.0 && wet_t > dry_t {
        match physics::design_report(&physics::ReportIn {
            p_fusion,
            f_exh: fleet.settings.f_exh,
            eta_noz: fleet.settings.eta_noz,
            e_afterburner: 0.0,
            ve_max: fleet.settings.ve_max_m_s,
            f_cap: (f_cap > 0.0).then_some(f_cap),
            m_dry: dry_t * 1000.0,
            m_wet: wet_t * 1000.0,
            dv_reserve: 0.0,
            rad_load_frac,
            sigma: fleet.settings.sigma,
            rad_hot,
            rad_low,
            sink_mj,
            flywheel_mj,
            lasers,
        }) {
            Ok(report) => {
                serde_json::to_value(report).map_err(|error| AgentError::io(error.to_string()))?
            }
            Err(error) => {
                issues.push(issue("error", "/designs/*", error));
                Value::Null
            }
        }
    } else {
        Value::Null
    };
    if let Some(margin) = report.get("hot_margin_w").and_then(Value::as_f64) {
        if margin < 0.0 {
            issues.push(issue(
                "error",
                "/designs/*/components",
                format!("hot radiator deficit {:.3e} W", -margin),
            ));
        }
    }
    let mut laser_performance = Vec::new();
    let mut missile_performance = Vec::new();
    let mut evaluated_missiles = BTreeSet::new();
    for (component_index, component) in design.components.iter().enumerate() {
        let component_path = format!("/designs/{design_index}/components/{component_index}");
        if component.kind == "laser" {
            let profiles = component
                .profiles
                .iter()
                .filter_map(|profile| {
                    fleet
                        .materials
                        .iter()
                        .find(|material| material.name == profile.material)
                        .map(|material| physics::LaserProfileIn {
                            name: profile.name.clone(),
                            rho: material.rho,
                            e_vap_mj: material.e_vap_mj,
                            t_pulse_s: profile.t_pulse_s,
                            threshold_mm: profile.threshold_mm,
                        })
                })
                .collect::<Vec<_>>();
            if profiles.len() != component.profiles.len() {
                issues.push(issue(
                    "error",
                    &format!("{component_path}/profiles"),
                    "one or more laser profiles reference an unknown material",
                ));
            }
            if profiles.is_empty() {
                issues.push(issue(
                    "warning",
                    &format!("{component_path}/profiles"),
                    "laser has no target profile, so credible range and fluence cannot be evaluated",
                ));
            } else {
                match physics::laser_profiles(&physics::LaserProfilesIn {
                    p_beam: component.p_beam_w.unwrap_or(0.0),
                    aperture: component.aperture_m.unwrap_or(0.0),
                    lambda: component.lambda_m.unwrap_or(0.0),
                    eta_drill: fleet.settings.eta_drill,
                    open_fire_factor: fleet.settings.open_fire_factor,
                    profiles,
                    n: Some(32),
                }) {
                    Ok(output) => laser_performance.push(json!({
                        "component_id": component.id,
                        "beam_power_w": output.beam_power_w,
                        "profiles": output.profiles.into_iter().map(|profile| json!({
                            "name": profile.name,
                            "kill_range_m": profile.r_kill,
                            "open_fire_range_m": profile.r_open,
                            "pulse_energy_j": profile.pulse_energy_j,
                            "peak_fluence_j_m2": profile.fluence_j_m2.into_iter().reduce(f64::max),
                        })).collect::<Vec<_>>(),
                    })),
                    Err(error) => issues.push(issue("error", &component_path, error)),
                }
            }
        }
        if component.kind == "magazine" {
            if let Some(missile_id) = &component.missile_id {
                if evaluated_missiles.insert(missile_id.clone()) {
                    if let Some(missile) = fleet
                        .missiles
                        .iter()
                        .find(|missile| &missile.id == missile_id)
                    {
                        let stages = missile
                            .stages
                            .iter()
                            .map(|stage| physics::MissileStageIn {
                                id: stage.id.clone(),
                                name: stage.name.clone(),
                                dry_mass_kg: stage.dry_mass_kg,
                                propellant_kg: stage.propellant_kg,
                                ve: match stage.propulsion.as_str() {
                                    "am" => {
                                        stage.isp_s.unwrap_or(fleet.settings.prop_am_isp_s)
                                            * fleet.settings.g
                                    }
                                    "fusion" => {
                                        stage.isp_s.unwrap_or(fleet.settings.prop_fusion_isp_s)
                                            * fleet.settings.g
                                    }
                                    "custom" => {
                                        stage.ve_m_s.unwrap_or(fleet.settings.prop_mh_ve_m_s)
                                    }
                                    _ => fleet.settings.prop_mh_ve_m_s,
                                },
                                a0_g: stage.a0_g,
                                jettison: stage.jettison,
                            })
                            .collect();
                        match physics::missile(&physics::MissileIn {
                            payload_kg: missile.payload_kg,
                            stages,
                            g: fleet.settings.g,
                        }) {
                            Ok(output) => missile_performance.push(json!({
                                "missile_id": missile.id,
                                "wet_mass_kg": output.m_wet,
                                "dry_mass_kg": output.m_dry,
                                "delta_v_m_s": output.dv,
                                "burn_time_s": output.t_burn,
                                "burnout_acceleration_g": output.a_burnout_g,
                                "terminal_effect": missile.terminal_effect,
                            })),
                            Err(error) => issues.push(issue("error", &component_path, error)),
                        }
                    }
                }
            }
        }
    }
    let has_sensor_profile = design.components.iter().any(|component| {
        component.kind.contains("sensor")
            || component.kind.contains("lidar")
            || component
                .combat
                .as_ref()
                .is_some_and(|profile| profile.role == "sensor")
    });
    if !has_sensor_profile {
        issues.push(issue(
            "warning",
            &format!("/designs/{design_index}/components"),
            "no sensor combat profile is installed; combat detection requires a scenario Lidar/PD template or range-falloff assumption",
        ));
    }
    Ok(json!({
        "design_id": design.id,
        "mass": {"component_t": component_mass_t, "ordnance_t": ordnance_t, "dry_t": dry_t, "wet_t": wet_t, "propellant_t": propellant_t, "tank_capacity_t": tank_capacity_t},
        "report": report,
        "weapon_performance": {"lasers": laser_performance, "missiles": missile_performance},
        "issues": issues,
        "credible": !issues.iter().any(|x| x["severity"] == "error"),
    }))
}

fn issue(severity: &str, path: &str, message: impl Into<String>) -> Value {
    json!({"severity": severity, "path": path, "message": message.into()})
}

fn simulate_command(root: &Path, args: SimulateArgs) -> Result<Envelope, AgentError> {
    match args.command {
        SimulateCommand::Validate { draft, input } => simulate_validate(root, &draft, &input),
        SimulateCommand::Place {
            draft,
            ships,
            separation_m,
        } => simulate_place(root, &draft, &ships, separation_m),
        SimulateCommand::Template {
            draft,
            ships,
            engagement,
            separation_m,
        } => simulate_template(root, &draft, &ships, &engagement, separation_m),
        SimulateCommand::Quick { draft, input } => simulate_quick(root, &draft, &input),
        SimulateCommand::Run { draft, input } => simulate_run(root, &draft, &input),
        SimulateCommand::Summary { draft, run } => simulate_summary(root, &draft, &run),
        SimulateCommand::Events {
            draft,
            run,
            offset,
            limit,
        } => simulate_events(root, &draft, &run, offset, limit),
        SimulateCommand::Compare { draft, runs } => simulate_compare(root, &draft, &runs),
    }
}

fn simulate_validate(root: &Path, draft_id: &str, input: &Path) -> Result<Envelope, AgentError> {
    let draft = draft_dir(root, draft_id)?;
    let fleet = model::FleetDocument::from_value(read_json_file(&draft.join("fleet.json"))?)
        .map_err(AgentError::validation)?;
    let scenario: combat::CombatScenario = serde_json::from_value(read_json_input(input)?)
        .map_err(|e| AgentError::validation(format!("combat scenario: {e}")))?;
    let errors = combat::validate_scenario(&fleet, &scenario);
    let warnings = combat::preflight_warnings(&fleet, &scenario);
    let valid = errors.is_empty() && warnings.is_empty();
    let summary = if valid {
        "Scenario is valid and ready to run".to_string()
    } else if errors.is_empty() {
        format!(
            "Scenario is valid but has {} warning(s); review before running",
            warnings.len()
        )
    } else {
        format!(
            "Scenario has {} error(s) and {} warning(s)",
            errors.len(),
            warnings.len()
        )
    };
    let mut out = Envelope::new(
        "simulate validate",
        summary,
        json!({
            "valid": errors.is_empty(),
            "ready": valid,
            "errors": errors,
            "warnings": warnings,
        }),
    );
    out.warnings = warnings.clone();
    Ok(out)
}

fn simulate_place(
    root: &Path,
    draft_id: &str,
    ships_csv: &str,
    separation_m: f64,
) -> Result<Envelope, AgentError> {
    let draft = draft_dir(root, draft_id)?;
    let fleet = model::FleetDocument::from_value(read_json_file(&draft.join("fleet.json"))?)
        .map_err(AgentError::validation)?;
    let ship_ids: Vec<&str> = ships_csv.split(',').map(str::trim).collect();
    if ship_ids.len() < 2 {
        return Err(AgentError::validation("at least two ship IDs are required"));
    }
    for id in &ship_ids {
        if !fleet.states.iter().any(|s| s.id == *id) {
            return Err(AgentError::missing(format!("unknown ship {id}")));
        }
    }
    if !separation_m.is_finite() || separation_m <= 0.0 {
        return Err(AgentError::validation("separation_m must be positive"));
    }
    let max_body_extent = fleet
        .system
        .bodies
        .iter()
        .map(|b| b.a_m.unwrap_or(0.0) + b.radius_m * 2.0)
        .fold(0.0, f64::max);
    let safe_origin = max_body_extent.max(1e11);
    let mut nav = BTreeMap::new();
    for (i, id) in ship_ids.iter().enumerate() {
        let offset = if i == 0 {
            0.0
        } else {
            separation_m * i as f64 / (ship_ids.len() - 1) as f64
        };
        let team_sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        nav.insert(
            id.to_string(),
            json!({
                "x": safe_origin + offset,
                "y": 0.0,
                "vx": team_sign * 25_000.0,
                "vy": 0.0,
                "landed_on": null,
            }),
        );
    }
    let body_clearances: Vec<Value> = fleet
        .system
        .bodies
        .iter()
        .map(|b| {
            let center = b.a_m.unwrap_or(0.0);
            json!({
                "body": b.name,
                "center_m": center,
                "radius_m": b.radius_m,
                "clearance_m": safe_origin - center - b.radius_m,
            })
        })
        .collect();
    Ok(Envelope::new(
        "simulate place",
        format!(
            "Placed {} ships at {:.0} Mm separation, {:.0} Gm from origin",
            ship_ids.len(),
            separation_m / 1e6,
            safe_origin / 1e9,
        ),
        json!({
            "initial_nav": nav,
            "separation_m": separation_m,
            "safe_origin_m": safe_origin,
            "body_clearances": body_clearances,
        }),
    ))
}

fn simulate_template(
    root: &Path,
    draft_id: &str,
    ships_csv: &str,
    engagement: &str,
    separation_m: f64,
) -> Result<Envelope, AgentError> {
    let draft = draft_dir(root, draft_id)?;
    let fleet = model::FleetDocument::from_value(read_json_file(&draft.join("fleet.json"))?)
        .map_err(AgentError::validation)?;
    let ship_ids: Vec<&str> = ships_csv.split(',').map(str::trim).collect();
    if ship_ids.len() < 2 {
        return Err(AgentError::validation("at least two ship IDs are required"));
    }
    if !matches!(engagement, "duel" | "ambush" | "pursuit") {
        return Err(AgentError::validation(
            "engagement must be duel, ambush, or pursuit",
        ));
    }
    for id in &ship_ids {
        if !fleet.states.iter().any(|s| s.id == *id) {
            return Err(AgentError::missing(format!("unknown ship {id}")));
        }
    }
    let max_body_extent = fleet
        .system
        .bodies
        .iter()
        .map(|b| b.a_m.unwrap_or(0.0) + b.radius_m * 2.0)
        .fold(0.0, f64::max);
    let safe_origin = max_body_extent.max(1e11);
    let mut initial_nav = BTreeMap::new();
    let mut participants = Vec::new();
    for (i, &id) in ship_ids.iter().enumerate() {
        let state = fleet.states.iter().find(|s| s.id == id).unwrap();
        let design = fleet.designs.iter().find(|d| d.id == state.design_id);
        let is_blue = i % 2 == 0;
        let team = if is_blue { "blue" } else { "red" };
        let offset = if is_blue { 0.0 } else { separation_m };
        let (vx, vy) = match engagement {
            "duel" => {
                if is_blue {
                    (25_000.0, 0.0)
                } else {
                    (-25_000.0, 0.0)
                }
            }
            "pursuit" => {
                if is_blue {
                    (30_000.0, 0.0)
                } else {
                    (5_000.0, 0.0)
                }
            }
            "ambush" => {
                if is_blue {
                    (25_000.0, 0.0)
                } else {
                    (0.0, 0.0)
                }
            }
            _ => (0.0, 0.0),
        };
        initial_nav.insert(
            id.to_string(),
            json!({
                "x": safe_origin + offset,
                "y": 0.0,
                "vx": vx,
                "vy": vy,
                "landed_on": null,
            }),
        );
        let max_laser_range = design.and_then(|d| {
            d.components
                .iter()
                .filter(|c| c.kind == "laser")
                .filter_map(|c| {
                    c.profiles
                        .iter()
                        .filter_map(|p| p.chosen_range_m)
                        .max_by(f64::total_cmp)
                })
                .max_by(f64::total_cmp)
        });
        let sensor_range = separation_m * 2.0;
        let missile_range = separation_m;
        let magazine_count: u32 = design
            .map(|d| {
                d.components
                    .iter()
                    .filter(|c| c.kind == "magazine")
                    .filter_map(|c| c.capacity)
                    .sum()
            })
            .unwrap_or(0);
        let salvo = if magazine_count > 20 {
            20
        } else if magazine_count > 0 {
            magazine_count.min(10)
        } else {
            1
        };
        let defensive_reserve = magazine_count / 4;
        let target_ids: Vec<String> = ship_ids
            .iter()
            .enumerate()
            .filter(|(j, _)| (j % 2 == 0) != is_blue)
            .map(|(_, &tid)| tid.to_string())
            .collect();
        participants.push(json!({
            "ship_id": id,
            "team": team,
            "doctrine": {
                "rules_of_engagement": if engagement == "ambush" && !is_blue { "return_fire" } else { "weapons_free" },
                "sensor_range_m": sensor_range,
                "sensor_cadence_s": 5,
                "missile_range_m": missile_range,
                "missile_salvo": salvo,
                "defensive_reserve": defensive_reserve,
                "laser_fire": max_laser_range.is_some() || design.is_some_and(|d| d.components.iter().any(|c| c.kind == "laser")),
                "retreat_integrity": 0.2,
                "target_priority": target_ids,
            },
        }));
    }
    let duration = match engagement {
        "pursuit" => 28_800.0,
        _ => 14_400.0,
    };
    let scenario = json!({
        "schema_version": "1.0",
        "name": format!("{} engagement — {} ships", engagement, ship_ids.len()),
        "duration_s": duration,
        "step_s": 10,
        "seed": 7,
        "samples": 50,
        "objective": format!(
            "{} engagement at {:.0} Mm initial separation. Auto-generated template.",
            engagement, separation_m / 1e6
        ),
        "initial_nav": initial_nav,
        "participants": participants,
    });
    Ok(Envelope::new(
        "simulate template",
        format!(
            "Generated {} scenario for {} ships at {:.0} Mm",
            engagement,
            ship_ids.len(),
            separation_m / 1e6,
        ),
        scenario,
    ))
}

fn simulate_quick(root: &Path, draft_id: &str, input: &Path) -> Result<Envelope, AgentError> {
    let draft = draft_dir(root, draft_id)?;
    let fleet = model::FleetDocument::from_value(read_json_file(&draft.join("fleet.json"))?)
        .map_err(AgentError::validation)?;
    let scenario: combat::CombatScenario = serde_json::from_value(read_json_input(input)?)
        .map_err(|e| AgentError::validation(format!("combat scenario: {e}")))?;
    let result = combat::run(&fleet, &scenario).map_err(AgentError::simulation)?;
    let scenario_library_path =
        draft
            .join("scenarios")
            .join(format!("{}-{}.json", slug(&scenario.name), scenario.seed));
    write_json_atomic(
        &scenario_library_path,
        &serde_json::to_value(&scenario).unwrap(),
    )?;
    let run_id = format!(
        "{}-{}-{}",
        slug(&scenario.name),
        scenario.seed,
        unix_seconds()
    );
    let run_dir = draft.join("runs").join(&run_id);
    if run_dir.exists() {
        return Err(AgentError::conflict(format!(
            "run {run_id} already exists; retry after the current second"
        )));
    }
    fs::create_dir_all(&run_dir).map_err(|e| AgentError::io(e.to_string()))?;
    let summary = json!({
        "scenario": result.scenario,
        "representative": {
            "seed": result.representative.seed,
            "winner": result.representative.winner,
            "end_time_s": result.representative.end_time_s,
            "ammunition_expended": result.representative.ammunition_expended,
            "resources": result.representative.resources,
            "components": result.representative.components,
        },
        "ensemble": result.ensemble,
        "assumptions": result.assumptions,
        "warnings": result.warnings,
    });
    write_json_atomic(
        &run_dir.join("scenario.json"),
        &serde_json::to_value(&scenario).unwrap(),
    )?;
    write_json_atomic(&run_dir.join("summary.json"), &summary)?;
    write_json_atomic(
        &run_dir.join("ensemble.json"),
        &serde_json::to_value(&result.ensemble).unwrap(),
    )?;
    let timeline = result
        .representative
        .events
        .iter()
        .map(|e| serde_json::to_string(e).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(run_dir.join("timeline.jsonl"), timeline)
        .map_err(|e| AgentError::io(e.to_string()))?;
    fs::write(run_dir.join("report.md"), combat_report(&scenario, &result))
        .map_err(|e| AgentError::io(e.to_string()))?;
    let key_events: Vec<_> = result
        .representative
        .events
        .iter()
        .filter(|e| {
            matches!(
                e.kind.as_str(),
                "track_acquired"
                    | "missile_launch"
                    | "missile_hit"
                    | "laser_fire"
                    | "retreat"
                    | "ship_defeated"
                    | "point_defense_kill"
            )
        })
        .take(30)
        .collect();
    let artifacts = [
        "summary.json",
        "timeline.jsonl",
        "ensemble.json",
        "report.md",
        "scenario.json",
    ]
    .iter()
    .map(|name| run_dir.join(name).display().to_string())
    .collect::<Vec<_>>();
    let mut out = Envelope::new(
        "simulate quick",
        format!(
            "{}: {} wins {:.0}% of {} samples, representative ends at {:.0}s",
            scenario.name,
            result.representative.winner.as_deref().unwrap_or("draw"),
            result
                .ensemble
                .win_probability
                .values()
                .copied()
                .fold(0.0, f64::max)
                * 100.0,
            result.ensemble.samples,
            result.representative.end_time_s,
        ),
        json!({
            "draft": draft_id,
            "run": run_id,
            "outcome": {
                "representative_seed": result.representative.seed,
                "representative_winner": result.representative.winner,
                "representative_end_time_s": result.representative.end_time_s,
                "samples": result.ensemble.samples,
                "draws": result.ensemble.draws,
                "win_probability": result.ensemble.win_probability,
                "timing": result.ensemble.timing,
            },
            "components": result.representative.components,
            "resources": result.representative.resources,
            "key_events": key_events,
            "warnings": result.warnings,
        }),
    );
    out.warnings = result.warnings;
    out.artifacts = artifacts;
    Ok(out)
}

fn simulate_run(root: &Path, draft_id: &str, input: &Path) -> Result<Envelope, AgentError> {
    let draft = draft_dir(root, draft_id)?;
    let fleet = model::FleetDocument::from_value(read_json_file(&draft.join("fleet.json"))?)
        .map_err(AgentError::validation)?;
    let scenario: combat::CombatScenario = serde_json::from_value(read_json_input(input)?)
        .map_err(|e| AgentError::validation(format!("combat scenario: {e}")))?;
    let result = combat::run(&fleet, &scenario).map_err(AgentError::simulation)?;
    let scenario_library_path =
        draft
            .join("scenarios")
            .join(format!("{}-{}.json", slug(&scenario.name), scenario.seed));
    write_json_atomic(
        &scenario_library_path,
        &serde_json::to_value(&scenario).unwrap(),
    )?;
    let run_id = format!(
        "{}-{}-{}",
        slug(&scenario.name),
        scenario.seed,
        unix_seconds()
    );
    let run_dir = draft.join("runs").join(&run_id);
    if run_dir.exists() {
        return Err(AgentError::conflict(format!(
            "run {run_id} already exists; retry after the current second"
        )));
    }
    fs::create_dir_all(&run_dir).map_err(|e| AgentError::io(e.to_string()))?;
    let summary = json!({
        "scenario": result.scenario,
        "representative": {
            "seed": result.representative.seed,
            "winner": result.representative.winner,
            "end_time_s": result.representative.end_time_s,
            "ammunition_expended": result.representative.ammunition_expended,
            "resources": result.representative.resources,
            "components": result.representative.components,
        },
        "ensemble": result.ensemble,
        "assumptions": result.assumptions,
        "warnings": result.warnings,
    });
    write_json_atomic(
        &run_dir.join("scenario.json"),
        &serde_json::to_value(&scenario).unwrap(),
    )?;
    write_json_atomic(&run_dir.join("summary.json"), &summary)?;
    write_json_atomic(
        &run_dir.join("ensemble.json"),
        &serde_json::to_value(&result.ensemble).unwrap(),
    )?;
    let timeline = result
        .representative
        .events
        .iter()
        .map(|e| serde_json::to_string(e).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(run_dir.join("timeline.jsonl"), timeline)
        .map_err(|e| AgentError::io(e.to_string()))?;
    fs::write(run_dir.join("report.md"), combat_report(&scenario, &result))
        .map_err(|e| AgentError::io(e.to_string()))?;
    let artifacts = [
        "summary.json",
        "timeline.jsonl",
        "ensemble.json",
        "report.md",
        "scenario.json",
    ]
    .iter()
    .map(|name| run_dir.join(name).display().to_string())
    .collect::<Vec<_>>();
    let mut artifacts = artifacts;
    artifacts.push(scenario_library_path.display().to_string());
    let mut out = Envelope::new(
        "simulate run",
        format!(
            "Completed {} seeded combat samples for {}",
            scenario.samples, scenario.name
        ),
        json!({
            "draft": draft_id,
            "run": run_id,
            "outcome": {
                "representative_seed": result.representative.seed,
                "representative_winner": result.representative.winner,
                "representative_end_time_s": result.representative.end_time_s,
                "samples": result.ensemble.samples,
                "draws": result.ensemble.draws,
                "win_probability": result.ensemble.win_probability,
                "timing": result.ensemble.timing,
                "mean_ammunition_expended": result.ensemble.mean_ammunition_expended,
            },
            "warnings": result.warnings,
        }),
    );
    out.warnings = result.warnings;
    out.artifacts = artifacts;
    Ok(out)
}

fn simulate_summary(root: &Path, draft: &str, run: &str) -> Result<Envelope, AgentError> {
    let path = run_dir(root, draft, run)?.join("summary.json");
    Ok(Envelope::new(
        "simulate summary",
        format!("Summary for run {run}"),
        read_json_file(&path)?,
    ))
}

fn simulate_events(
    root: &Path,
    draft: &str,
    run: &str,
    offset: usize,
    limit: usize,
) -> Result<Envelope, AgentError> {
    let path = run_dir(root, draft, run)?.join("timeline.jsonl");
    let text = fs::read_to_string(&path).map_err(|e| AgentError::io(e.to_string()))?;
    let events = text
        .lines()
        .skip(offset)
        .take(limit.min(1000))
        .map(|line| serde_json::from_str::<Value>(line).map_err(|e| AgentError::io(e.to_string())))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Envelope::new(
        "simulate events",
        format!("{} events from run {run}", events.len()),
        json!({"offset": offset, "limit": limit, "events": events}),
    ))
}

fn simulate_compare(root: &Path, draft: &str, runs: &[String]) -> Result<Envelope, AgentError> {
    let mut summaries = Vec::new();
    for run in runs {
        let summary = read_json_file(&run_dir(root, draft, run)?.join("summary.json"))?;
        summaries.push(json!({
            "run": run,
            "scenario": summary["scenario"],
            "representative": {
                "seed": summary["representative"]["seed"],
                "winner": summary["representative"]["winner"],
                "end_time_s": summary["representative"]["end_time_s"],
            },
            "ensemble": summary["ensemble"],
        }));
    }
    Ok(Envelope::new(
        "simulate compare",
        format!("Compared {} combat runs", runs.len()),
        json!(summaries),
    ))
}

fn combat_report(scenario: &combat::CombatScenario, result: &combat::CombatRun) -> String {
    let winner = result
        .representative
        .winner
        .as_deref()
        .unwrap_or("none before the time limit");
    let mut text = format!(
        "# {}\n\n{}\n\n## Representative run\n\n- Seed: `{}`\n- Winner: **{}**\n- End time: {:.1} s\n- Recorded events: {}\n\n## Ensemble\n\n- Samples: {}\n- Draws: {}\n- Mean end time: {:.1} s\n",
        scenario.name,
        if scenario.objective.is_empty() { "Map-backed combat simulation." } else { &scenario.objective },
        result.representative.seed,
        winner,
        result.representative.end_time_s,
        result.representative.events.len(),
        result.ensemble.samples,
        result.ensemble.draws,
        result.ensemble.mean_end_time_s,
    );
    for (team, probability) in &result.ensemble.win_probability {
        text.push_str(&format!(
            "- {} win probability: {:.1}%\n",
            team,
            probability * 100.0
        ));
    }
    if !result.ensemble.timing.is_empty() {
        text.push_str("\n### Event timing\n\n");
        for (kind, timing) in &result.ensemble.timing {
            text.push_str(&format!(
                "- First {kind}: median {}, p90 {} ({} observed)\n",
                optional_seconds(timing.median_s),
                optional_seconds(timing.p90_s),
                timing.samples
            ));
        }
    }
    if !result.representative.resources.is_empty() {
        text.push_str("\n## Representative resources\n\n");
        for resources in &result.representative.resources {
            text.push_str(&format!(
                "- {}: {:.3} t propellant, {:.3} MJ stored heat, {:.3} MJ flywheel, {} tracks{}\n",
                resources.ship_id,
                resources.propellant_t,
                resources.heat_mj,
                resources.flywheel_mj,
                resources.tracks,
                if resources.retreated {
                    ", retreated"
                } else {
                    ""
                }
            ));
        }
    }
    text.push_str("\n## Assumptions\n\n");
    for assumption in &result.assumptions {
        text.push_str(&format!("- {assumption}\n"));
    }
    text.push_str("\n## Representative timeline\n\n");
    for event in result.representative.events.iter().take(200) {
        text.push_str(&format!("- T+{:.1}s — {}\n", event.time_s, event.message));
    }
    text
}

fn optional_seconds(value: Option<f64>) -> String {
    value.map_or_else(|| "not observed".into(), |value| format!("{value:.1} s"))
}

fn run_dir(root: &Path, draft: &str, run: &str) -> Result<PathBuf, AgentError> {
    if !safe_identifier(run) {
        return Err(AgentError::validation("invalid run id"));
    }
    let path = draft_dir(root, draft)?.join("runs").join(run);
    if !path.is_dir() {
        return Err(AgentError::missing(format!(
            "run {run} not found in draft {draft}"
        )));
    }
    Ok(path)
}

fn read_fleet(path: &Path) -> Result<(Vec<u8>, Value, model::FleetDocument), AgentError> {
    let bytes =
        fs::read(path).map_err(|e| AgentError::io(format!("read {}: {e}", path.display())))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|e| AgentError::validation(format!("fleet JSON: {e}")))?;
    let typed = model::FleetDocument::from_value(value.clone()).map_err(AgentError::validation)?;
    Ok((bytes, value, typed))
}

fn read_json_input(path: &Path) -> Result<Value, AgentError> {
    let mut text = String::new();
    if path == Path::new("-") {
        io::stdin()
            .read_to_string(&mut text)
            .map_err(|e| AgentError::io(e.to_string()))?;
    } else {
        text = fs::read_to_string(path)
            .map_err(|e| AgentError::io(format!("read {}: {e}", path.display())))?;
    }
    serde_json::from_str(&text).map_err(|e| AgentError::validation(format!("input JSON: {e}")))
}

fn read_json_file(path: &Path) -> Result<Value, AgentError> {
    let text = fs::read_to_string(path)
        .map_err(|e| AgentError::io(format!("read {}: {e}", path.display())))?;
    serde_json::from_str(&text)
        .map_err(|e| AgentError::validation(format!("{}: {e}", path.display())))
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), AgentError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AgentError::io(e.to_string()))?;
    }
    let temp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(value).map_err(|e| AgentError::io(e.to_string()))?;
    fs::write(&temp, text)
        .and_then(|_| fs::rename(&temp, path))
        .map_err(|e| AgentError::io(format!("write {}: {e}", path.display())))
}

fn append_json_line(path: &Path, value: &Value) -> Result<(), AgentError> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| AgentError::io(e.to_string()))?;
    writeln!(file, "{}", serde_json::to_string(value).unwrap())
        .map_err(|e| AgentError::io(e.to_string()))
}

fn draft_dir(root: &Path, id: &str) -> Result<PathBuf, AgentError> {
    if !safe_identifier(id) {
        return Err(AgentError::validation("invalid draft id"));
    }
    let dir = root.join(id);
    if !dir.is_dir() {
        return Err(AgentError::missing(format!("draft {id} not found")));
    }
    Ok(dir)
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn read_manifest(dir: &Path) -> Result<DraftManifest, AgentError> {
    serde_json::from_value(read_json_file(&dir.join("manifest.json"))?)
        .map_err(|e| AgentError::validation(format!("draft manifest: {e}")))
}

fn apply_patch_operation(root: &mut Value, op: &PatchOperation) -> Result<(), AgentError> {
    if !op.path.starts_with('/') || op.path == "/" {
        return Err(AgentError::validation(format!(
            "patch path must be a non-root JSON Pointer: {}",
            op.path
        )));
    }
    let mut tokens = op
        .path
        .split('/')
        .skip(1)
        .map(unescape_pointer)
        .collect::<Vec<_>>();
    let leaf = tokens.pop().unwrap();
    let mut parent = root;
    for token in tokens {
        parent = match parent {
            Value::Object(map) => map.get_mut(&token),
            Value::Array(array) => token.parse::<usize>().ok().and_then(|i| array.get_mut(i)),
            _ => None,
        }
        .ok_or_else(|| AgentError::missing(format!("patch parent not found for {}", op.path)))?;
    }
    match (op.op.as_str(), parent) {
        ("add", Value::Object(map)) => {
            map.insert(leaf, op.value.clone());
        }
        ("replace", Value::Object(map)) => {
            if !map.contains_key(&leaf) {
                return Err(AgentError::missing(format!(
                    "replace path not found: {}",
                    op.path
                )));
            }
            map.insert(leaf, op.value.clone());
        }
        ("remove", Value::Object(map)) => {
            map.remove(&leaf).ok_or_else(|| {
                AgentError::missing(format!("remove path not found: {}", op.path))
            })?;
        }
        ("add", Value::Array(array)) => {
            if leaf == "-" {
                array.push(op.value.clone());
            } else {
                let index = leaf.parse::<usize>().map_err(|_| {
                    AgentError::validation("array patch index must be an integer or -")
                })?;
                if index > array.len() {
                    return Err(AgentError::missing("array add index out of bounds"));
                }
                array.insert(index, op.value.clone());
            }
        }
        ("replace", Value::Array(array)) => {
            let index = leaf
                .parse::<usize>()
                .map_err(|_| AgentError::validation("array patch index must be an integer"))?;
            *array
                .get_mut(index)
                .ok_or_else(|| AgentError::missing("array replace index out of bounds"))? =
                op.value.clone();
        }
        ("remove", Value::Array(array)) => {
            let index = leaf
                .parse::<usize>()
                .map_err(|_| AgentError::validation("array patch index must be an integer"))?;
            if index >= array.len() {
                return Err(AgentError::missing("array remove index out of bounds"));
            }
            array.remove(index);
        }
        (other, _) => {
            return Err(AgentError::validation(format!(
                "unsupported patch operation {other}; use add, replace, or remove"
            )))
        }
    }
    Ok(())
}

fn unescape_pointer(value: &str) -> String {
    value.replace("~1", "/").replace("~0", "~")
}

fn diff_paths(before: &Value, after: &Value, path: &str, limit: usize) -> Vec<Value> {
    let mut out = Vec::new();
    diff_inner(before, after, path, limit, &mut out);
    out
}

fn diff_inner(before: &Value, after: &Value, path: &str, limit: usize, out: &mut Vec<Value>) {
    if out.len() >= limit || before == after {
        return;
    }
    match (before, after) {
        (Value::Object(a), Value::Object(b)) => {
            let keys: BTreeSet<_> = a.keys().chain(b.keys()).collect();
            for key in keys {
                let child = format!("{}/{}", path, key.replace('~', "~0").replace('/', "~1"));
                match (a.get(key), b.get(key)) {
                    (Some(x), Some(y)) => diff_inner(x, y, &child, limit, out),
                    (x, y) => out.push(json!({"path": child, "before": x, "after": y})),
                }
                if out.len() >= limit { break; }
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len().max(b.len()) <= 64 => {
            for i in 0..a.len().max(b.len()) {
                let child = format!("{path}/{i}");
                match (a.get(i), b.get(i)) {
                    (Some(x), Some(y)) => diff_inner(x, y, &child, limit, out),
                    (x, y) => out.push(json!({"path": child, "before": x, "after": y})),
                }
                if out.len() >= limit { break; }
            }
        }
        _ => out.push(json!({"path": if path.is_empty() { "/" } else { path }, "before": before, "after": after})),
    }
}

fn slug(value: &str) -> String {
    let s = value
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let trimmed = s.trim_matches('-');
    if trimmed.is_empty() {
        "draft".into()
    } else {
        trimmed.chars().take(48).collect()
    }
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("slipstick-{label}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn calculator_catalog_has_schemas() {
        for name in calculator_names() {
            assert!(domain_schema(name).is_some(), "missing {name}");
        }
    }

    #[test]
    fn every_exposed_field_has_description_and_numeric_unit() {
        for name in calculator_names()
            .into_iter()
            .chain(["fleet", "combat", "missile_design"])
        {
            let schema = domain_schema(name).unwrap();
            assert_schema_metadata(&schema, name);
        }
    }

    #[test]
    fn calculator_fields_are_discoverable_without_loading_every_schema() {
        let result = fields_command(FieldsArgs {
            command: FieldsCommand::Get {
                path: "gear.input.p_fusion".into(),
            },
        })
        .unwrap();
        assert_eq!(result.data["path"], "gear.input.p_fusion");
        assert_eq!(result.data["unit"], "W");
    }

    #[test]
    fn codex_and_claude_wrappers_are_identical() {
        assert_eq!(include_str!("../AGENTS.md"), include_str!("../CLAUDE.md"));
    }

    #[test]
    fn agent_error_exit_codes_are_stable() {
        assert_eq!(AgentError::validation("x").code, 2);
        assert_eq!(AgentError::conflict("x").code, 3);
        assert_eq!(AgentError::missing("x").code, 4);
        assert_eq!(AgentError::simulation("x").code, 5);
        assert_eq!(AgentError::io("x").code, 6);
    }

    fn assert_schema_metadata(schema: &Value, path: &str) {
        let Some(object) = schema.as_object() else {
            return;
        };
        if let Some(properties) = object.get("properties").and_then(Value::as_object) {
            for (name, child) in properties {
                let child_path = format!("{path}.{name}");
                assert!(
                    child
                        .get("description")
                        .and_then(Value::as_str)
                        .is_some_and(|description| !description.is_empty()),
                    "missing description at {child_path}"
                );
                let numeric = matches!(
                    child.get("type").and_then(Value::as_str),
                    Some("number" | "integer")
                );
                if numeric {
                    assert!(
                        child.get("x-unit").is_some(),
                        "missing unit at {child_path}"
                    );
                }
                assert_schema_metadata(child, &child_path);
            }
        }
        if let Some(defs) = object.get("$defs").and_then(Value::as_object) {
            for (name, child) in defs {
                assert_schema_metadata(child, &format!("{path}<{name}>"));
            }
        }
        for key in ["input", "output"] {
            if let Some(child) = object.get(key) {
                assert_schema_metadata(child, &format!("{path}.{key}"));
            }
        }
    }

    #[test]
    fn patch_add_replace_remove() {
        let mut value = json!({"a": [1, 2], "b": true});
        apply_patch_operation(
            &mut value,
            &PatchOperation {
                op: "add".into(),
                path: "/a/-".into(),
                value: json!(3),
            },
        )
        .unwrap();
        apply_patch_operation(
            &mut value,
            &PatchOperation {
                op: "replace".into(),
                path: "/b".into(),
                value: json!(false),
            },
        )
        .unwrap();
        apply_patch_operation(
            &mut value,
            &PatchOperation {
                op: "remove".into(),
                path: "/a/0".into(),
                value: Value::Null,
            },
        )
        .unwrap();
        assert_eq!(value, json!({"a": [2, 3], "b": false}));
    }

    #[test]
    fn every_persisted_top_level_field_is_documented() {
        let mut fields = Vec::new();
        collect_schema_fields(&json!(schema_for!(model::FleetDocument)), "", &mut fields);
        for name in [
            "schema_version",
            "settings",
            "materials",
            "missiles",
            "designs",
            "states",
            "events",
            "system",
        ] {
            let field = fields.iter().find(|f| f["path"] == name).unwrap();
            assert!(
                !field["description"].as_str().unwrap_or("").is_empty(),
                "{name}"
            );
        }
    }

    #[test]
    fn draft_allows_invalid_intermediate_then_rolls_back() {
        let root = temporary_directory("draft-rollback");
        let fleet_path = root.join("fleet.json");
        fs::write(&fleet_path, include_str!("default_fleet.json")).unwrap();
        let workspaces = root.join("workspaces");
        let created = draft_create(&fleet_path, &workspaces, "invalid intermediate").unwrap();
        let id = created.data["id"].as_str().unwrap();
        let patch_path = root.join("invalid.patch.json");
        fs::write(
            &patch_path,
            r#"[{"op":"replace","path":"/schema_version","value":"drafting"}]"#,
        )
        .unwrap();

        draft_patch(&workspaces, id, &patch_path).unwrap();
        let validation = draft_validate(&workspaces, id).unwrap();
        assert_eq!(validation.data["valid"], false);
        draft_rollback(&workspaces, id, 1).unwrap();
        let restored = draft_get(&workspaces, id, Some("/schema_version")).unwrap();
        assert_eq!(restored.data, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selective_commit_and_revision_conflict_are_enforced() {
        let root = temporary_directory("draft-commit");
        let fleet_path = root.join("fleet.json");
        fs::write(&fleet_path, include_str!("default_fleet.json")).unwrap();
        let workspaces = root.join("workspaces");
        let created = draft_create(&fleet_path, &workspaces, "selective commit").unwrap();
        let id = created.data["id"].as_str().unwrap();
        let patch_path = root.join("rename.patch.json");
        fs::write(
            &patch_path,
            r#"[{"op":"replace","path":"/designs/0/name","value":"Agent refit"}]"#,
        )
        .unwrap();
        draft_patch(&workspaces, id, &patch_path).unwrap();

        let empty = draft_commit(&fleet_path, &workspaces, id, &[], false).unwrap_err();
        assert_eq!(empty.code, 2);
        let selection = vec!["designs:battleship".to_string()];
        let preview = draft_commit(&fleet_path, &workspaces, id, &selection, false).unwrap();
        assert_eq!(preview.data["applied"], false);
        draft_commit(&fleet_path, &workspaces, id, &selection, true).unwrap();
        assert_eq!(
            read_json_file(&fleet_path).unwrap()["designs"][0]["name"],
            "Agent refit"
        );

        let stale = draft_create(&fleet_path, &workspaces, "stale base").unwrap();
        let stale_id = stale.data["id"].as_str().unwrap();
        let mut live = fs::read_to_string(&fleet_path).unwrap();
        live.push('\n');
        fs::write(&fleet_path, live).unwrap();
        let conflict =
            draft_commit(&fleet_path, &workspaces, stale_id, &selection, false).unwrap_err();
        assert_eq!(conflict.code, 3);
        fs::remove_dir_all(root).unwrap();
    }
}
