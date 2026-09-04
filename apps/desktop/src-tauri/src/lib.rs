use yoube_core::AppContext;

#[tauri::command]
fn ping(ctx: tauri::State<'_, AppContext>) -> Result<String, yoube_core::AppError> {
    Ok(ctx.ping()?.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppContext::new())
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
