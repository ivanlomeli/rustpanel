use axum::{
    extract::{State, Request},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use std::time::{SystemTime, UNIX_EPOCH};

const SECRET_KEY: &[u8] = b"secret_key_change_me_in_production";

#[derive(Clone)]
struct AppState {
    sys: Arc<Mutex<System>>,
    disks: Arc<Mutex<Disks>>,
}

#[derive(Serialize)]
struct SystemMetrics {
    cpu_usage: f32,
    total_memory: u64,
    used_memory: u64,
    memory_percentage: f32,
    total_disk: u64,
    used_disk: u64,
    disk_percentage: f32,
    os_name: String,
    host_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[tokio::main]
async fn main() {
    let sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );
    let disks = Disks::new_with_refreshed_list();

    let state = AppState {
        sys: Arc::new(Mutex::new(sys)),
        disks: Arc::new(Mutex::new(disks)),
    };

    let app = Router::new()
        .route("/api/system", get(get_system_metrics))
        .route_layer(middleware::from_fn(auth_middleware)) // Protect system route
        .route("/api/login", post(login_handler))          // Public login route
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 RustPanel Core running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn login_handler(Json(payload): Json<LoginRequest>) -> Result<Json<LoginResponse>, StatusCode> {
    // TODO: Use a real database and hashed passwords
    if payload.username == "admin" && payload.password == "password" {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 3600; // 1 hour expiration

        let claims = Claims {
            sub: payload.username,
            exp: expiration,
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(LoginResponse { token }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn auth_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req.headers().get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &DecodingKey::from_secret(SECRET_KEY), &validation);

        if token_data.is_ok() {
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn get_system_metrics(State(state): State<AppState>) -> Json<SystemMetrics> {
    let mut sys = state.sys.lock().unwrap();
    let mut disks = state.disks.lock().unwrap();
    
    sys.refresh_cpu();
    sys.refresh_memory();
    disks.refresh_list();
    disks.refresh();

    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    
    let memory_percentage = if total_memory > 0 {
        (used_memory as f32 / total_memory as f32) * 100.0
    } else {
        0.0
    };

    let mut total_disk = 0;
    let mut used_disk = 0;
    for disk in disks.iter() {
        total_disk += disk.total_space();
        used_disk += disk.total_space() - disk.available_space();
    }

    let disk_percentage = if total_disk > 0 {
        (used_disk as f32 / total_disk as f32) * 100.0
    } else {
        0.0
    };

    let metrics = SystemMetrics {
        cpu_usage,
        total_memory,
        used_memory,
        memory_percentage,
        total_disk,
        used_disk,
        disk_percentage,
        os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
        host_name: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
    };

    Json(metrics)
}