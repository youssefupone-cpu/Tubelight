use specta::Type;
use tauri_specta::{collect_commands, collect_events, Builder};
use yoube_core::AppContext;

#[derive(serde::Serialize, serde::Deserialize, Type)]
pub struct PingResponse { pub value: String }

#[tauri::command]
#[specta::specta]
fn ping(ctx: tauri::State<'_, AppContext>) -> Result<PingResponse, yoube_core::AppError> {
    Ok(PingResponse { value: ctx.ping()?.to_string() })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::default()
        .commands(collect_commands![ping]);

    #[cfg(debug_assertions)]
    builder
        .export(specta_typescript::Typescript::default(), "../src/bindings.ts")
        .expect("failed to export typescript bindings");

    tauri::Builder::default()
        .manage(AppContext::new())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| { builder.mount_events(app); Ok(()) })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
