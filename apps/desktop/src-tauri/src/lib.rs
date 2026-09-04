use yoube_core::AppContext;

#[cfg(feature = "contracts")]
use specta::Type;
#[cfg(feature = "contracts")]
use tauri_specta::{collect_commands, Builder as SpectaBuilder};

#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "contracts", derive(Type))]
pub struct PingResponse { pub value: String }

#[tauri::command]
#[cfg_attr(feature = "contracts", specta::specta)]
fn ping(ctx: tauri::State<'_, AppContext>) -> Result<PingResponse, yoube_core::AppError> {
    Ok(PingResponse { value: ctx.ping()?.to_string() })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(feature = "contracts")]
    {
        let builder = SpectaBuilder::<tauri::Wry>::default()
            .commands(collect_commands![ping]);

        #[cfg(debug_assertions)]
        {
            builder
                .export(specta_typescript::Typescript::default(), "../src/bindings.ts")
                .expect("failed to export typescript bindings");
        }

        tauri::Builder::default()
            .manage(AppContext::new())
            .invoke_handler(builder.invoke_handler())
            .setup(move |app| { builder.mount_events(app); Ok(()) })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }

    #[cfg(not(feature = "contracts"))]
    tauri::Builder::default()
        .manage(AppContext::new())
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
