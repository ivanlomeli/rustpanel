use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

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

#[tokio::main]

async fn main() {

    // Initialize system info with specific refresh requirements

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

        .layer(CorsLayer::permissive())

        .with_state(state);



    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 RustPanel Core running on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();

}



async fn get_system_metrics(State(state): State<AppState>) -> Json<SystemMetrics> {

    let mut sys = state.sys.lock().unwrap();

    let mut disks = state.disks.lock().unwrap();

    

    // Refresh data

    sys.refresh_cpu();

    sys.refresh_memory();

    disks.refresh_list(); // Refresh list to catch mounted/unmounted drives

    disks.refresh();      // Refresh usage statistics



    let cpu_usage = sys.global_cpu_info().cpu_usage();

    let total_memory = sys.total_memory();

    let used_memory = sys.used_memory();

    

    let memory_percentage = if total_memory > 0 {

        (used_memory as f32 / total_memory as f32) * 100.0

    } else {

        0.0

    };



    // Calculate total disk usage across all mounted disks

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
