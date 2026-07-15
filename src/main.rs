mod physics;

use axum::{
    body::Bytes,
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

const DEFAULT_FLEET: &str = include_str!("default_fleet.json");

struct AppState {
    data_path: PathBuf,
    lock: Mutex<()>,
}

#[tokio::main]
async fn main() {
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

    let app = Router::new()
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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8017")
        .await
        .expect("bind 127.0.0.1:8017");
    println!("Slipstick running — open http://localhost:8017");
    axum::serve(listener, app).await.unwrap();
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
