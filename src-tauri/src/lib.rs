mod commands;

use commands::convert::{cleanup_temp_dir, convert_to_audio};
use commands::history::{save_history_entry, get_history_list, get_history_entry, delete_history_entry};
use commands::presets::{save_preset, get_presets, delete_preset};
use commands::settings::{delete_api_key, get_api_key, set_api_key};
use commands::transcribe::{poll_transcription, submit_transcription, upload_audio};
use commands::vocabulary::{
    load_vocabularies, create_vocabulary, update_vocabulary, delete_vocabulary,
    duplicate_vocabulary, create_vocabulary_category, export_vocabularies, import_vocabularies
};
use commands::vocabulary_extract::{extract_document_text, extract_vocabulary_terms};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging - logs will appear in terminal
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    log::info!("OHG Scribe starting...");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // FFmpeg conversion
            convert_to_audio,
            cleanup_temp_dir,
            // Settings
            get_api_key,
            set_api_key,
            delete_api_key,
            // AssemblyAI
            upload_audio,
            submit_transcription,
            poll_transcription,
            // History
            save_history_entry,
            get_history_list,
            get_history_entry,
            delete_history_entry,
            // Presets
            save_preset,
            get_presets,
            delete_preset,
            // Vocabulary
            load_vocabularies,
            create_vocabulary,
            update_vocabulary,
            delete_vocabulary,
            duplicate_vocabulary,
            create_vocabulary_category,
            export_vocabularies,
            import_vocabularies,
            // Vocabulary extraction
            extract_document_text,
            extract_vocabulary_terms,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
