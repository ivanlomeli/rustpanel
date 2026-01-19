use axum::{
    extract::{Query, State, Request},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::fs;
use std::path::Path;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use std::time::{SystemTime, UNIX_EPOCH};
use sqlx::sqlite::SqlitePool;
use dotenvy::dotenv;
use std::env;
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Clone)]
struct AppState {
    sys: Arc<Mutex<System>>,
    disks: Arc<Mutex<Disks>>,
    db: SqlitePool,
    jwt_secret: String,
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

#[derive(Serialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory: u64,
}

#[derive(Serialize)]
struct ServiceInfo {
    name: String,
    status: String,
    description: String,
}

#[derive(Serialize)]
struct FileInfo {
    name: String,
    is_dir: bool,
    size: u64,
}

#[derive(Deserialize)]
struct FileQuery {
    path: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let pool = SqlitePool::connect(&database_url).await.expect("Failed to connect to database");
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL
        )"
    ).execute(&pool).await.unwrap();

    let admin_exists = sqlx::query("SELECT 1 FROM users WHERE username = 'admin'")
        .fetch_optional(&pool).await.unwrap();

    if admin_exists.is_none() {
        let hashed = hash("password", DEFAULT_COST).unwrap();
        sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
            .bind("admin")
            .bind(hashed)
            .execute(&pool).await.unwrap();
        println!("👤 Created default admin user (admin / password)");
    }

    let sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );
    let disks = Disks::new_with_refreshed_list();

    let state = AppState {
        sys: Arc::new(Mutex::new(sys)),
        disks: Arc::new(Mutex::new(disks)),
        db: pool,
        jwt_secret,
    };

    let app = Router::new()
        .route("/api/system", get(get_system_metrics))
        .route("/api/processes", get(get_processes))
        .route("/api/services", get(get_services))
        .route("/api/files", get(list_files))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .route("/api/login", post(login_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("🚀 RustPanel Core running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn list_files(Query(params): Query<FileQuery>) -> Result<Json<Vec<FileInfo>>, StatusCode> {
    let default_path = ".".to_string();
    let path_str = params.path.as_ref().unwrap_or(&default_path);
    let path = Path::new(path_str);

    if path_str.contains("..") {
        return Err(StatusCode::FORBIDDEN);
    }

    match fs::read_dir(path) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    files.push(FileInfo {
                        name: entry.file_name().to_string_lossy().to_string(),
                        is_dir: metadata.is_dir(),
                        size: metadata.len(),
                    });
                }
            }
            files.sort_by(|a, b| {
                if a.is_dir == b.is_dir {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                } else {
                    b.is_dir.cmp(&a.is_dir)
                }
            });
            Ok(Json(files))
        },
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_services() -> Json<Vec<ServiceInfo>> {
    let services_to_check = ["nginx", "mysql", "ssh", "docker", "cron"];
    let mut services = Vec::new();

    for service_name in services_to_check {
        let output = std::process::Command::new("systemctl")
            .arg("is-active")
            .arg(service_name)
            .output();

        let status = match output {
            Ok(out) => {
                if out.status.success() {
                    "active".to_string()
                } else {
                    "inactive".to_string()
                }
            }
            Err(_) => "unknown".to_string(),
        };

        services.push(ServiceInfo {
            name: service_name.to_string(),
            status,
            description: format!("System service: {}", service_name),
        });
    }

    Json(services)
}

async fn get_processes(State(state): State<AppState>) -> Json<Vec<ProcessInfo>> {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_processes();

    let mut processes: Vec<ProcessInfo> = sys.processes().iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string(),
            cpu_usage: process.cpu_usage(),
            memory: process.memory(),
        })
        .collect();

    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
    processes.truncate(20);

    Json(processes)
}

async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>
) -> Result<Json<LoginResponse>, StatusCode> {
    let user = sqlx::query_as::<_, (String,)>("SELECT password FROM users WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some((hashed_password,)) = user {
        if verify(&payload.password, &hashed_password).unwrap_or(false) {
            let expiration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize + 3600;

            let claims = Claims {
                sub: payload.username,
                exp: expiration,
            };

            let token = encode(
                &Header::default(), 
                &claims, 
                &EncodingKey::from_secret(state.jwt_secret.as_bytes())
            ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok(Json(LoginResponse { token }));
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: Request, 
    next: Next
) -> Result<Response, StatusCode> {
    let auth_header = req.headers().get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        let validation = Validation::default();
        let token_data = decode::<Claims>(
            token, 
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()), 
            &validation
        );

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
