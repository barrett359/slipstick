mod agent;
mod combat;
mod model;
mod physics;

use axum::{
    body::Bytes,
    extract::{rejection::JsonRejection, DefaultBodyLimit, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

const DEFAULT_FLEET: &str = include_str!("default_fleet.json");

struct AppState {
    data_path: PathBuf,
    lock: Mutex<()>,
}

#[derive(Debug, Parser)]
#[command(
    name = "slipstick",
    version,
    about = "Ship design, navigation, and combat analysis"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the local Slipstick web application.
    Serve,
    /// Use Slipstick through the stable JSON agent interface.
    Agent(agent::AgentArgs),
}

#[tokio::main]
async fn main() {
    match Cli::parse().command {
        Some(Command::Agent(args)) => {
            let (envelope, exit_code) = match agent::execute(args) {
                Ok(envelope) => (envelope, 0),
                Err(error) => {
                    eprintln!("{}: {}", error.kind, error.message);
                    (error.envelope(), error.code)
                }
            };
            println!(
                "{}",
                serde_json::to_string(&envelope).expect("serialize agent result envelope")
            );
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
            return;
        }
        Some(Command::Serve) | None => serve().await,
    }
}

async fn serve() {
    let data_dir = PathBuf::from("data");
    let data_path = data_dir.join("fleet.json");
    if !data_path.exists() {
        std::fs::create_dir_all(&data_dir).expect("create data dir");
        std::fs::write(&data_path, DEFAULT_FLEET).expect("write default fleet.json");
        println!("Initialized {}", data_path.display());
    }
    let state = Arc::new(AppState {
        data_path,
        lock: Mutex::new(()),
    });

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8017")
        .await
        .expect("bind 127.0.0.1:8017");
    println!("Slipstick running — open http://localhost:8017");
    axum::serve(listener, app).await.unwrap();
}

fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/style.css", get(style_css))
        .route("/plot.js", get(plot_js))
        .route("/map.js", get(map_js))
        .route("/app.js", get(app_js))
        .route("/api/data", get(get_data).put(put_data))
        .route("/api/calc/gear", post(calc_gear))
        .route("/api/calc/drive_curve", post(calc_drive_curve))
        .route("/api/calc/deltav", post(calc_deltav))
        .route("/api/calc/travel", post(calc_travel))
        .route("/api/calc/burn", post(calc_burn))
        .route("/api/calc/sprint", post(calc_sprint))
        .route("/api/calc/autosize", post(calc_autosize))
        .route("/api/calc/laser", post(calc_laser))
        .route("/api/calc/laser_profiles", post(calc_laser_profiles))
        .route("/api/calc/radiator", post(calc_radiator))
        .route("/api/calc/vent", post(calc_vent))
        .route("/api/calc/missile", post(calc_missile))
        .route("/api/calc/missile_optimize", post(calc_missile_optimize))
        .route("/api/calc/intercept", post(calc_intercept))
        .route("/api/calc/design_report", post(calc_design_report))
        .route("/api/calc/nav_tick", post(calc_nav_tick))
        .route("/api/calc/orbit_v", post(calc_orbit_v))
        .route("/api/calc/burn_for_dv", post(calc_burn_for_dv))
        .route("/api/calc/nav_intercept", post(calc_nav_intercept))
        .route(
            "/api/calc/lidar_pd",
            post(calc_lidar_pd).layer(DefaultBodyLimit::max(1024 * 1024)),
        )
        .route(
            "/api/calc/missile_engagement",
            post(calc_missile_engagement).layer(DefaultBodyLimit::max(4 * 1024 * 1024)),
        )
        .with_state(state)
}

// ---- static frontend (embedded: one binary, works offline) ----------------

async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn style_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../static/style.css"),
    )
}

async fn plot_js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../static/plot.js"),
    )
}

async fn map_js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../static/map.js"),
    )
}

async fn app_js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../static/app.js"),
    )
}

// ---- dumb JSON store: whole document in, whole document out ---------------

async fn get_data(State(st): State<Arc<AppState>>) -> Response {
    let _g = st.lock.lock().await;
    match std::fs::read_to_string(&st.data_path) {
        Ok(s) => ([(header::CONTENT_TYPE, "application/json")], s).into_response(),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("read fleet.json: {}", e),
        ),
    }
}

async fn put_data(State(st): State<Arc<AppState>>, body: Bytes) -> Response {
    let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&body);
    let doc = match parsed {
        Ok(v) => v,
        Err(e) => return err(StatusCode::BAD_REQUEST, &format!("invalid JSON: {}", e)),
    };
    let fleet = match model::FleetDocument::from_value(doc.clone()) {
        Ok(fleet) => fleet,
        Err(message) => return err(StatusCode::UNPROCESSABLE_ENTITY, &message),
    };
    let validation_errors = fleet.validate();
    if !validation_errors.is_empty() {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            &validation_errors.join("; "),
        );
    }
    let pretty = serde_json::to_string_pretty(&doc).unwrap();
    let _g = st.lock.lock().await;
    // Write via temp file so a crash mid-write can't destroy the fleet.
    let tmp = st.data_path.with_extension("json.tmp");
    let res = std::fs::write(&tmp, &pretty).and_then(|_| std::fs::rename(&tmp, &st.data_path));
    match res {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("write fleet.json: {}", e),
        ),
    }
}

fn err(code: StatusCode, msg: &str) -> Response {
    (code, Json(serde_json::json!({ "error": msg }))).into_response()
}

// ---- calc endpoints --------------------------------------------------------

fn calc_response<T: serde::Serialize>(r: physics::CalcResult<T>) -> Response {
    match r {
        Ok(v) => Json(v).into_response(),
        Err(msg) => err(StatusCode::UNPROCESSABLE_ENTITY, &msg),
    }
}

async fn calc_gear(Json(i): Json<physics::GearIn>) -> Response {
    calc_response(physics::gear(&i))
}
async fn calc_drive_curve(Json(i): Json<physics::DriveCurveIn>) -> Response {
    calc_response(physics::drive_curve(&i))
}
async fn calc_deltav(Json(i): Json<physics::DeltavIn>) -> Response {
    calc_response(physics::deltav(&i))
}
async fn calc_travel(Json(i): Json<physics::TravelIn>) -> Response {
    calc_response(physics::travel(&i))
}
async fn calc_burn(Json(i): Json<physics::BurnIn>) -> Response {
    calc_response(physics::timed_burn(&i))
}
async fn calc_sprint(Json(i): Json<physics::SprintIn>) -> Response {
    calc_response(physics::sprint(&i))
}
async fn calc_autosize(Json(i): Json<physics::AutosizeIn>) -> Response {
    calc_response(physics::autosize(&i))
}
async fn calc_laser_profiles(Json(i): Json<physics::LaserProfilesIn>) -> Response {
    calc_response(physics::laser_profiles(&i))
}
async fn calc_laser(Json(i): Json<physics::LaserIn>) -> Response {
    calc_response(physics::laser(&i))
}
async fn calc_radiator(Json(i): Json<physics::RadiatorIn>) -> Response {
    calc_response(physics::radiator(&i))
}
async fn calc_vent(Json(i): Json<physics::VentIn>) -> Response {
    calc_response(physics::vent(&i))
}
async fn calc_missile(Json(i): Json<physics::MissileIn>) -> Response {
    calc_response(physics::missile(&i))
}
async fn calc_missile_optimize(Json(i): Json<physics::MissileOptimizeIn>) -> Response {
    calc_response(physics::optimize_missile(&i))
}
async fn calc_intercept(Json(i): Json<physics::InterceptIn>) -> Response {
    calc_response(physics::intercept(&i))
}
async fn calc_design_report(Json(i): Json<physics::ReportIn>) -> Response {
    calc_response(physics::design_report(&i))
}
async fn calc_nav_tick(Json(i): Json<physics::NavTickIn>) -> Response {
    calc_response(physics::nav_tick(&i))
}
async fn calc_orbit_v(Json(i): Json<physics::OrbitVIn>) -> Response {
    calc_response(physics::orbit_v(&i))
}
async fn calc_burn_for_dv(Json(i): Json<physics::BurnForDvIn>) -> Response {
    calc_response(physics::burn_for_dv(&i))
}
async fn calc_nav_intercept(Json(i): Json<physics::NavInterceptIn>) -> Response {
    calc_response(physics::nav_intercept(&i))
}

async fn calc_lidar_pd(payload: Result<Json<physics::LidarPdIn>, JsonRejection>) -> Response {
    let Json(i) = match payload {
        Ok(value) => value,
        Err(rejection) => return err(rejection.status(), &rejection.body_text()),
    };
    match physics::lidar_pd(&i) {
        Ok(mut value) => {
            value.calculation_id = calculation_uuid();
            Json(value).into_response()
        }
        Err(message) => err(StatusCode::UNPROCESSABLE_ENTITY, &message),
    }
}

async fn calc_missile_engagement(
    payload: Result<Json<physics::MissileEngagementIn>, JsonRejection>,
) -> Response {
    let Json(input) = match payload {
        Ok(value) => value,
        Err(rejection) => return err(rejection.status(), &rejection.body_text()),
    };
    calc_response(physics::missile_engagement(&input))
}

fn calculation_uuid() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let salt = COUNTER.fetch_add(1, Ordering::Relaxed) ^ u64::from(std::process::id());
    let mix = |mut x: u64| {
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^ (x >> 31)
    };
    let high = mix(nanos as u64 ^ salt);
    let low = mix((nanos >> 64) as u64 ^ salt.rotate_left(29) ^ high);
    let mut bytes = (((high as u128) << 64) | low as u128).to_be_bytes();
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

#[cfg(test)]
mod api_tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    fn app() -> Router {
        build_router(Arc::new(AppState {
            data_path: PathBuf::from("/tmp/slipstick-api-test-unused.json"),
            lock: Mutex::new(()),
        }))
    }

    fn request(body: impl Into<Body>) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/calc/lidar_pd")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body.into())
            .unwrap()
    }

    #[tokio::test]
    async fn lidar_pd_api_accepts_default_and_adds_uuid() {
        let response = app()
            .oneshot(request(include_str!("../testdata/lidar_pd_baseline.json")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let id = value["calculation_id"].as_str().unwrap();
        assert_eq!(id.len(), 36);
        assert_eq!(value["schema_version"], "1.0");
    }

    #[tokio::test]
    async fn lidar_pd_api_distinguishes_json_and_validation_errors() {
        let malformed = app().oneshot(request("{")).await.unwrap();
        assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);

        let invalid = include_str!("../testdata/lidar_pd_baseline.json").replacen(
            "\"schema_version\": \"1.0\"",
            "\"schema_version\": \"2.0\"",
            1,
        );
        let response = app().oneshot(request(invalid)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(value["error"]
            .as_str()
            .unwrap()
            .starts_with("schema_version:"));
    }

    #[tokio::test]
    async fn lidar_pd_api_rejects_oversized_body() {
        let oversized = format!("{{\"padding\":\"{}\"}}", "x".repeat(1024 * 1024));
        let response = app().oneshot(request(oversized)).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn fleet_save_returns_the_validation_detail() {
        let mut fleet: serde_json::Value =
            serde_json::from_str(include_str!("default_fleet.json")).unwrap();
        fleet["system"]["bodies"][0]["mass_kg"] = serde_json::json!(-1.0);
        let response = app()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/data")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&fleet).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(value["error"]
            .as_str()
            .unwrap()
            .contains("system body sol"));
    }

    #[tokio::test]
    async fn embedded_frontend_guards_and_explains_failed_saves() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/app.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let javascript = std::str::from_utf8(&body).unwrap();
        assert!(javascript.contains("invalidNumberPath(DB)"));
        assert!(javascript.contains("await saveResponseError(res)"));
        assert!(javascript.contains("Save failed: "));
    }

    #[tokio::test]
    async fn missile_engagement_api_accepts_range_stepped_scenario() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/calc/missile_engagement")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(include_str!(
                        "../testdata/tf_sahara_missile_engagement.json"
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["summary"]["conservation_check_passed"], true);
        assert_eq!(
            value["checkpoints"].as_array().unwrap().last().unwrap()["range_m"],
            50_000.0
        );
    }

    #[tokio::test]
    async fn cli_and_http_calculator_results_are_identical() {
        let input = serde_json::json!({
            "p_fusion": 1.82e6,
            "f_exh": 0.753,
            "eta_noz": 0.85,
            "e_afterburner": 0.0,
            "ve": 2.3e6,
            "ve_max": 2.3e6,
            "f_cap": null,
            "mass_kg": 2300.0,
            "duration_s": 10.0
        });
        let input_bytes = serde_json::to_vec(&input).unwrap();
        let cli_input = serde_json::from_slice(&input_bytes).unwrap();
        let expected = agent::dispatch_calculation("gear", cli_input).unwrap();
        // Compare the public JSON wire representation on both surfaces. The
        // in-memory serde_json number retains an extra binary-float digit that
        // is intentionally normalized when stdout/HTTP serializes it.
        let expected: serde_json::Value =
            serde_json::from_slice(&serde_json::to_vec(&expected).unwrap()).unwrap();
        let response = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/calc/gear")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(input_bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let actual: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(actual, expected);
    }
}
