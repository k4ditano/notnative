use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Spanish,
    English,
}

impl Language {
    pub fn from_code(code: &str) -> Self {
        match code {
            "en" | "en_US" | "en_GB" => Language::English,
            "es" | "es_ES" | "es_MX" => Language::Spanish,
            _ => {
                // Detectar por prefijo
                if code.starts_with("en") {
                    Language::English
                } else if code.starts_with("es") {
                    Language::Spanish
                } else {
                    Language::Spanish // Default
                }
            }
        }
    }

    pub fn from_env() -> Self {
        std::env::var("LANG")
            .ok()
            .and_then(|lang| lang.split('.').next().map(String::from))
            .map(|code| Self::from_code(&code))
            .unwrap_or(Language::Spanish)
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::Spanish => "es",
            Language::English => "en",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Language::Spanish => "Español",
            Language::English => "English",
        }
    }
}

#[derive(Debug, Clone)]
pub struct I18n {
    language: Language,
    translations: HashMap<&'static str, (&'static str, &'static str)>,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        let mut translations = HashMap::new();

        // (key, (spanish, english))
        translations.insert("app_title", ("NotNative", "NotNative"));
        translations.insert("untitled", ("Sin título", "Untitled"));
        translations.insert("notes", ("Notas", "Notes"));
        translations.insert("new_note", ("Nueva nota", "New Note"));
        translations.insert("search", ("Buscar", "Search"));
        translations.insert("search_notes", ("Buscar (Ctrl+F)", "Search (Ctrl+F)"));
        translations.insert("search_placeholder", ("Buscar notas...", "Search notes..."));
        translations.insert(
            "show_hide_notes",
            ("Mostrar/ocultar lista de notas", "Show/hide notes list"),
        );
        translations.insert("preferences", ("Preferencias", "Preferences"));
        translations.insert(
            "keyboard_shortcuts",
            ("Atajos de teclado", "Keyboard Shortcuts"),
        );
        translations.insert("about", ("Acerca de", "About"));
        translations.insert("settings", ("Ajustes", "Settings"));
        translations.insert("tags", ("Tags", "Tags"));
        translations.insert("tags_note", ("Tags de la nota", "Note tags"));
        translations.insert("no_tags", ("No hay tags", "No tags"));
        translations.insert(
            "search_tag",
            ("Buscar notas con este tag", "Search notes with this tag"),
        );
        translations.insert("remove_tag", ("Eliminar tag", "Remove tag"));
        translations.insert("close", ("Cerrar", "Close"));

        // Diálogos
        translations.insert("create_note_title", ("Nueva nota", "New Note"));
        translations.insert(
            "note_name_hint",
            (
                "ejemplo: proyectos/nueva-idea",
                "example: projects/new-idea",
            ),
        );
        translations.insert(
            "create_folder_hint",
            (
                "Usa '/' para crear en carpetas",
                "Use '/' to create in folders",
            ),
        );
        translations.insert("create", ("Crear", "Create"));
        translations.insert("cancel", ("Cancelar", "Cancel"));
        translations.insert("rename", ("Renombrar", "Rename"));
        translations.insert("delete", ("Eliminar", "Delete"));
        translations.insert("change_icon", ("Cambiar icono", "Change icon"));
        translations.insert("view_history", ("Ver historial", "View history"));
        translations.insert(
            "open_in_file_manager",
            ("Abrir en explorador", "Open in file manager"),
        );
        translations.insert(
            "confirm_delete",
            (
                "¿Estás seguro de eliminar",
                "Are you sure you want to delete",
            ),
        );

        // Preferencias
        translations.insert("theme", ("Tema", "Theme"));
        translations.insert(
            "theme_sync",
            (
                "La aplicación sincroniza automáticamente con el tema Omarchy",
                "The app automatically syncs with Omarchy theme",
            ),
        );
        translations.insert(
            "markdown_rendering",
            ("Renderizado Markdown", "Markdown Rendering"),
        );
        translations.insert(
            "markdown_enabled",
            (
                "Activado por defecto en modo Normal",
                "Enabled by default in Normal mode",
            ),
        );
        translations.insert("language", ("Idioma", "Language"));
        translations.insert(
            "language_description",
            (
                "Elige el idioma de la interfaz",
                "Choose the interface language",
            ),
        );
        translations.insert(
            "restart_required",
            (
                "Se requiere reiniciar la aplicación",
                "Application restart required",
            ),
        );

        translations.insert(
            "start_in_background",
            ("Iniciar en segundo plano", "Start in background"),
        );
        translations.insert(
            "start_in_background_desc",
            (
                "Iniciar la aplicación minimizada en la bandeja del sistema",
                "Start the application minimized to the system tray",
            ),
        );

        // Format Toolbar
        translations.insert("format_toolbar", ("Barra de formato", "Format Toolbar"));
        translations.insert(
            "format_toolbar_desc",
            (
                "Mostrar barra de herramientas de formato en modo edición",
                "Show formatting toolbar in edit mode",
            ),
        );

        // Workspace
        translations.insert("workspace", ("Directorio de trabajo", "Workspace"));
        translations.insert(
            "workspace_description",
            (
                "Ubicación donde se guardan las notas y recursos",
                "Location where notes and resources are saved",
            ),
        );
        translations.insert(
            "open_workspace_folder",
            ("Abrir carpeta de trabajo", "Open workspace folder"),
        );
        translations.insert("change_workspace", ("Cambiar ubicación", "Change location"));
        translations.insert("workspace_location", ("Ubicación", "Location"));
        translations.insert(
            "select_workspace_folder",
            ("Seleccionar carpeta de trabajo", "Select workspace folder"),
        );
        translations.insert("select", ("Seleccionar", "Select"));

        // Audio
        translations.insert("audio_output", ("Salida de audio", "Audio Output"));
        translations.insert(
            "audio_output_description",
            (
                "Seleccionar dispositivo de salida de audio",
                "Select audio output device",
            ),
        );
        translations.insert(
            "audio_output_default",
            ("Por defecto del sistema", "System default"),
        );
        translations.insert(
            "audio_output_detecting",
            ("Detectando dispositivos...", "Detecting devices..."),
        );
        translations.insert(
            "audio_output_changed",
            ("Salida de audio cambiada", "Audio output changed"),
        );
        translations.insert(
            "audio_output_error",
            (
                "Error cambiando salida de audio",
                "Error changing audio output",
            ),
        );

        // Atajos de teclado
        translations.insert("shortcuts_general", ("General", "General"));
        translations.insert("shortcuts_modes", ("Modos de edición", "Editing Modes"));
        translations.insert("shortcuts_navigation", ("Navegación", "Navigation"));
        translations.insert("shortcuts_editing", ("Edición", "Editing"));

        translations.insert("shortcut_new_note", ("Nueva nota", "New note"));
        translations.insert("shortcut_save", ("Guardar nota", "Save note"));
        translations.insert("shortcut_search", ("Buscar notas", "Search notes"));
        translations.insert(
            "shortcut_toggle_sidebar",
            ("Alternar sidebar", "Toggle sidebar"),
        );
        translations.insert("shortcut_escape", ("Volver al editor", "Back to editor"));

        translations.insert("shortcut_insert_mode", ("Modo Insert", "Insert mode"));
        translations.insert("shortcut_normal_mode", ("Modo Normal", "Normal mode"));
        translations.insert("shortcut_command_mode", ("Modo Command", "Command mode"));
        translations.insert("shortcut_visual_mode", ("Modo Visual", "Visual mode"));

        translations.insert(
            "shortcut_movement",
            ("Izquierda/Abajo/Arriba/Derecha", "Left/Down/Up/Right"),
        );
        translations.insert("shortcut_next_word", ("Siguiente palabra", "Next word"));
        translations.insert("shortcut_prev_word", ("Palabra anterior", "Previous word"));
        translations.insert("shortcut_line_start", ("Inicio de línea", "Start of line"));
        translations.insert("shortcut_line_end", ("Fin de línea", "End of line"));
        translations.insert(
            "shortcut_doc_start",
            ("Inicio del documento", "Start of document"),
        );
        translations.insert("shortcut_doc_end", ("Fin del documento", "End of document"));

        translations.insert(
            "shortcut_delete_char",
            ("Eliminar carácter", "Delete character"),
        );
        translations.insert("shortcut_delete_line", ("Eliminar línea", "Delete line"));
        translations.insert("shortcut_undo", ("Deshacer", "Undo"));
        translations.insert("shortcut_redo", ("Rehacer", "Redo"));

        // About
        translations.insert(
            "app_description",
            (
                "Editor de notas markdown con estilo vim",
                "Vim-style markdown note editor",
            ),
        );
        translations.insert("website", ("Sitio web", "Website"));
        translations.insert("authors", ("Autores", "Authors"));
        translations.insert("version", ("Versión", "Version"));
        translations.insert("license", ("Licencia", "License"));

        // Búsqueda
        translations.insert(
            "no_results",
            ("No se encontraron resultados para", "No results found for"),
        );
        translations.insert("searching", ("Buscando", "Searching"));

        // Estados
        translations.insert("lines", ("líneas", "lines"));
        translations.insert("words", ("palabras", "words"));
        translations.insert("characters", ("caracteres", "characters"));
        translations.insert("saved", ("Guardado", "Saved"));
        translations.insert(
            "unsaved_changes",
            ("Cambios sin guardar", "Unsaved changes"),
        );

        // Mensajes
        translations.insert("note_created", ("Nota creada", "Note created"));
        translations.insert("note_deleted", ("Nota eliminada", "Note deleted"));
        translations.insert("note_renamed", ("Nota renombrada", "Note renamed"));
        translations.insert("error", ("Error", "Error"));
        translations.insert("success", ("Éxito", "Success"));

        // Visor de imágenes
        translations.insert("image_viewer", ("Visor de imagen", "Image Viewer"));
        translations.insert("open_file_location", ("Abrir ubicación", "Open Location"));

        // TODOs
        translations.insert("todos", ("TODOs", "TODOs"));
        translations.insert("todos_note", ("TODOs de la nota", "Note TODOs"));
        translations.insert(
            "no_todos",
            ("No hay TODOs en esta nota", "No TODOs in this note"),
        );
        translations.insert("completed", ("completo", "completed"));
        translations.insert("no_section", ("Sin sección", "No section"));

        // YouTube
        translations.insert(
            "transcribe_youtube",
            (
                "¿Transcribir video de YouTube?",
                "Transcribe YouTube video?",
            ),
        );
        translations.insert(
            "youtube_detected",
            (
                "Se ha detectado un enlace de YouTube",
                "A YouTube link has been detected",
            ),
        );
        translations.insert("only_link", ("Solo enlace", "Only link"));
        translations.insert(
            "transcribe_and_insert",
            ("Transcribir e insertar", "Transcribe and insert"),
        );
        translations.insert(
            "downloading_transcript",
            ("Descargando transcripción...", "Downloading transcript..."),
        );
        translations.insert(
            "loading_transcript",
            ("Cargando transcripción...", "Loading transcript..."),
        );
        translations.insert(
            "transcript_error",
            (
                "Error al obtener transcripción",
                "Error fetching transcript",
            ),
        );
        translations.insert(
            "transcript_unavailable",
            (
                "Transcripción no disponible para este video",
                "Transcript unavailable for this video",
            ),
        );
        translations.insert("transcript_section", ("📝 Transcripción", "📝 Transcript"));

        // Music Player
        translations.insert(
            "music_search_placeholder",
            ("Buscar música en YouTube...", "Search music on YouTube..."),
        );
        translations.insert(
            "no_music_playing",
            ("No hay música reproduciéndose", "No music playing"),
        );
        translations.insert(
            "music_play_pause",
            ("Reproducir/Pausar (Espacio)", "Play/Pause (Space)"),
        );
        translations.insert("music_stop", ("Detener", "Stop"));
        translations.insert("music_seek_back", ("Retroceder 5s", "Seek back 5s"));
        translations.insert("music_seek_forward", ("Avanzar 5s", "Seek forward 5s"));
        translations.insert("music_volume_down", ("Bajar volumen", "Lower volume"));
        translations.insert("music_volume_up", ("Subir volumen", "Raise volume"));
        translations.insert("music_previous_song", ("Canción anterior", "Previous song"));
        translations.insert("music_next_song", ("Siguiente canción", "Next song"));
        translations.insert("music_repeat_off", ("Repetir: OFF", "Repeat: OFF"));
        translations.insert("music_repeat_one", ("Repetir: UNA", "Repeat: ONE"));
        translations.insert("music_repeat_all", ("Repetir: TODAS", "Repeat: ALL"));
        translations.insert("music_shuffle_off", ("Aleatorio: OFF", "Shuffle: OFF"));
        translations.insert("music_shuffle_on", ("Aleatorio: ON", "Shuffle: ON"));
        translations.insert(
            "music_manage_playlists",
            ("Gestionar playlists", "Manage playlists"),
        );
        translations.insert(
            "music_playback_queue",
            ("Cola de reproducción", "Playback queue"),
        );
        translations.insert("music_loading", ("Cargando...", "Loading..."));
        translations.insert("music_add_to_queue", ("Añadir a cola", "Add to queue"));
        translations.insert(
            "music_remove_from_queue",
            ("Eliminar de cola", "Remove from queue"),
        );
        translations.insert("music_new_playlist", ("Nueva playlist", "New playlist"));
        translations.insert("music_load_playlist", ("Cargar playlist", "Load playlist"));
        translations.insert("music_save_playlist", ("Guardar playlist", "Save playlist"));
        translations.insert(
            "music_playlist_name",
            ("Nombre de la playlist", "Playlist name"),
        );

        // System Tray
        translations.insert("tray_show_window", ("Mostrar ventana", "Show window"));
        translations.insert("tray_hide_window", ("Ocultar ventana", "Hide window"));
        translations.insert("tray_quit", ("Salir", "Quit"));

        // AI Chat
        translations.insert("ai_chat", ("Chat IA", "AI Chat"));
        translations.insert(
            "chat_input_placeholder",
            ("Escribe tu mensaje aquí...", "Type your message here..."),
        );
        translations.insert("chat_send", ("Enviar", "Send"));
        translations.insert(
            "chat_model_default",
            ("Modelo: OpenAI GPT-4", "Model: OpenAI GPT-4"),
        );
        translations.insert(
            "chat_subtitle",
            (
                "Combina tus notas con el asistente en tiempo real",
                "Combine your notes with the assistant in real time",
            ),
        );
        translations.insert("chat_context", ("Contexto", "Context"));
        translations.insert(
            "chat_attach_note",
            ("Adjuntar nota actual", "Attach current note"),
        );
        translations.insert(
            "chat_attach_note_dialog_title",
            ("Adjuntar nota al contexto", "Attach note to context"),
        );
        translations.insert("chat_attach_button", ("Adjuntar", "Attach"));
        translations.insert("chat_clear_context", ("Limpiar contexto", "Clear context"));
        translations.insert("chat_clear_history", ("Borrar historial", "Clear history"));
        translations.insert(
            "chat_clear_history_confirm_title",
            (
                "¿Borrar todo el historial de chat?",
                "Delete all chat history?",
            ),
        );
        translations.insert(
            "chat_clear_history_confirm_message",
            (
                "Esta acción eliminará permanentemente todo el historial de conversaciones guardado. No se puede deshacer.",
                "This action will permanently delete all saved conversation history. This cannot be undone.",
            ),
        );
        translations.insert(
            "chat_history_cleared",
            ("Historial borrado", "History cleared"),
        );
        translations.insert(
            "chat_history_cleared_message",
            (
                "Se ha eliminado todo el historial de conversaciones",
                "All conversation history has been deleted",
            ),
        );

        // Chat AI - Modo Agente
        translations.insert("chat_mode_agent", ("Modo: Agente", "Mode: Agent"));
        translations.insert("chat_mode_normal", ("Modo: Normal", "Mode: Normal"));
        translations.insert("chat_new_session", ("Nueva sesión", "New Session"));
        translations.insert("chat_mode_chat", ("Modo: Chat", "Mode: Chat"));
        translations.insert(
            "chat_toggle_mode_tooltip",
            (
                "Alternar: Modo Agente (con herramientas) / Chat Normal (sin herramientas)",
                "Toggle: Agent Mode (with tools) / Normal Chat (without tools)",
            ),
        );
        translations.insert("chat_agent_thinking", ("💭 Pensamiento", "💭 Thought"));
        translations.insert("chat_agent_action", ("🔧 Acción", "🔧 Action"));
        translations.insert(
            "chat_agent_observation",
            ("👁️ Observación", "👁️ Observation"),
        );
        translations.insert("chat_agent_answer", ("✅ Respuesta", "✅ Answer"));

        translations.insert(
            "music_player_title",
            ("Reproductor de Música", "Music Player"),
        );
        translations.insert(
            "ai_chat_placeholder",
            (
                "Escribe un mensaje para el asistente IA...",
                "Type a message for the AI assistant...",
            ),
        );
        translations.insert("ai_send_message", ("Enviar mensaje", "Send message"));
        translations.insert("ai_thinking", ("Pensando...", "Thinking..."));
        translations.insert("ai_model", ("Modelo", "Model"));
        translations.insert("ai_temperature", ("Temperatura", "Temperature"));
        translations.insert("ai_api_key", ("API Key", "API Key"));
        translations.insert("ai_openai", ("OpenAI", "OpenAI"));
        translations.insert("ai_openrouter", ("OpenRouter", "OpenRouter"));
        translations.insert(
            "ai_api_key_placeholder",
            ("Ingresa tu API key...", "Enter your API key..."),
        );
        translations.insert("ai_save", ("Guardar", "Save"));
        translations.insert("ai_cancel", ("Cancelar", "Cancel"));
        translations.insert(
            "ai_no_key_configured",
            ("No hay API key configurada", "No API key configured"),
        );
        translations.insert(
            "ai_configure_key",
            (
                "Configurar API key en Preferencias",
                "Configure API key in Preferences",
            ),
        );
        translations.insert(
            "ai_free_models",
            ("═══ MODELOS GRATUITOS ═══", "═══ FREE MODELS ═══"),
        );
        translations.insert(
            "ai_paid_models",
            ("═══ MODELOS DE PAGO ═══", "═══ PAID MODELS ═══"),
        );

        // MCP Messages
        translations.insert(
            "mcp_note_created",
            (
                "✓ Nota '{}' creada exitosamente",
                "✓ Note '{}' created successfully",
            ),
        );
        translations.insert(
            "mcp_note_read",
            (
                "✓ Nota '{}' leída correctamente",
                "✓ Note '{}' read successfully",
            ),
        );
        translations.insert(
            "mcp_note_updated",
            (
                "✓ Nota '{}' actualizada exitosamente",
                "✓ Note '{}' updated successfully",
            ),
        );
        translations.insert(
            "mcp_note_deleted",
            (
                "✓ Nota '{}' eliminada exitosamente",
                "✓ Note '{}' deleted successfully",
            ),
        );
        translations.insert(
            "mcp_content_appended",
            (
                "✓ Contenido agregado a '{}' exitosamente",
                "✓ Content appended to '{}' successfully",
            ),
        );
        translations.insert(
            "mcp_notes_found",
            ("✓ {} notas encontradas", "✓ {} notes found"),
        );
        translations.insert(
            "mcp_search_results",
            ("✓ {} resultados para '{}'", "✓ {} results for '{}'"),
        );
        translations.insert(
            "mcp_notes_with_tag",
            ("✓ {} notas con tag #{}", "✓ {} notes with tag #{}"),
        );
        translations.insert(
            "mcp_tags_found",
            ("✓ {} tags encontrados", "✓ {} tags found"),
        );
        translations.insert(
            "mcp_tags_added",
            ("✓ Tags agregados a '{}'", "✓ Tags added to '{}'"),
        );
        translations.insert(
            "mcp_note_renamed",
            (
                "✓ Nota renombrada de '{}' a '{}'",
                "✓ Note renamed from '{}' to '{}'",
            ),
        );
        translations.insert(
            "mcp_note_duplicated",
            (
                "✓ Nota '{}' duplicada como '{}'",
                "✓ Note '{}' duplicated as '{}'",
            ),
        );
        translations.insert(
            "mcp_folder_created",
            (
                "✓ Carpeta '{}' creada exitosamente",
                "✓ Folder '{}' created successfully",
            ),
        );
        translations.insert(
            "mcp_note_not_found",
            ("Nota '{}' no encontrada", "Note '{}' not found"),
        );
        translations.insert(
            "mcp_folders_found",
            ("✓ {} carpetas encontradas", "✓ {} folders found"),
        );

        // === RECORDATORIOS / REMINDERS ===
        translations.insert("reminders_title", ("Recordatorios", "Reminders"));
        translations.insert("reminders_new", ("Nuevo recordatorio", "New reminder"));
        translations.insert("reminders_pending", ("Pendientes", "Pending"));
        translations.insert("reminders_completed", ("Completados", "Completed"));
        translations.insert("reminders_all", ("Todos", "All"));
        translations.insert("reminder_complete", ("Completar", "Complete"));
        translations.insert("reminder_snooze", ("Posponer", "Snooze"));
        translations.insert("reminder_edit", ("Editar", "Edit"));
        translations.insert("reminder_delete", ("Eliminar", "Delete"));
        translations.insert("reminders_empty", ("No hay recordatorios", "No reminders"));
        translations.insert("reminder_priority_low", ("Baja", "Low"));
        translations.insert("reminder_priority_medium", ("Media", "Medium"));
        translations.insert("reminder_priority_high", ("Alta", "High"));
        translations.insert("reminder_priority_urgent", ("Urgente", "Urgent"));
        translations.insert("reminder_snooze_5min", ("5 minutos", "5 minutes"));
        translations.insert("reminder_snooze_15min", ("15 minutos", "15 minutes"));
        translations.insert("reminder_snooze_1hour", ("1 hora", "1 hour"));
        translations.insert("reminder_snooze_tomorrow", ("Mañana", "Tomorrow"));
        translations.insert(
            "reminder_notification_title",
            ("🔔 Recordatorio", "🔔 Reminder"),
        );
        translations.insert(
            "reminder_create_title",
            ("Crear recordatorio", "Create reminder"),
        );
        translations.insert("reminder_title_label", ("Título", "Title"));
        translations.insert("reminder_description_label", ("Descripción", "Description"));
        translations.insert("reminder_date_label", ("Fecha y hora", "Date and time"));
        translations.insert("reminder_priority_label", ("Prioridad", "Priority"));
        translations.insert(
            "reminder_linked_note_label",
            ("Nota vinculada", "Linked note"),
        );
        translations.insert("reminder_repeat_label", ("Repetir", "Repeat"));
        translations.insert("reminder_repeat_none", ("No repetir", "Don't repeat"));
        translations.insert("reminder_repeat_daily", ("Diariamente", "Daily"));
        translations.insert("reminder_repeat_weekly", ("Semanalmente", "Weekly"));
        translations.insert("reminder_repeat_monthly", ("Mensualmente", "Monthly"));
        translations.insert("no_reminders", ("No hay recordatorios", "No reminders"));
        translations.insert("reminders_count", ("{} pendientes", "{} pending"));
        translations.insert("reminder_overdue", ("Vencido", "Overdue"));
        translations.insert("reminder_today", ("Hoy", "Today"));
        translations.insert("reminder_tomorrow", ("Mañana", "Tomorrow"));
        translations.insert(
            "reminder_created",
            ("Recordatorio creado", "Reminder created"),
        );
        translations.insert(
            "reminder_updated",
            ("Recordatorio actualizado", "Reminder updated"),
        );
        translations.insert(
            "reminder_deleted",
            ("Recordatorio eliminado", "Reminder deleted"),
        );
        translations.insert(
            "reminder_completed_msg",
            ("Recordatorio completado", "Reminder completed"),
        );
        translations.insert(
            "reminder_snoozed",
            ("Recordatorio pospuesto", "Reminder snoozed"),
        );
        translations.insert(
            "reminder_tooltip",
            ("Recordatorios (Alt+R)", "Reminders (Alt+R)"),
        );

        // MCP Tools para recordatorios
        translations.insert(
            "mcp_create_reminder_desc",
            (
                "Crea un nuevo recordatorio con fecha, hora y prioridad",
                "Create a new reminder with date, time and priority",
            ),
        );
        translations.insert(
            "mcp_list_reminders_desc",
            (
                "Lista todos los recordatorios o filtra por estado",
                "List all reminders or filter by status",
            ),
        );
        translations.insert(
            "mcp_update_reminder_desc",
            (
                "Actualiza un recordatorio existente",
                "Update an existing reminder",
            ),
        );
        translations.insert(
            "mcp_delete_reminder_desc",
            ("Elimina un recordatorio", "Delete a reminder"),
        );
        translations.insert(
            "mcp_snooze_reminder_desc",
            (
                "Pospone un recordatorio por un tiempo específico",
                "Snooze a reminder for a specific time",
            ),
        );
        translations.insert(
            "mcp_complete_reminder_desc",
            (
                "Marca un recordatorio como completado",
                "Mark a reminder as completed",
            ),
        );
        translations.insert(
            "mcp_reminder_title_desc",
            ("Título del recordatorio", "Reminder title"),
        );
        translations.insert(
            "mcp_reminder_date_desc",
            (
                "Fecha y hora del recordatorio (ej: '2025-11-20 15:00', 'hoy 18:00', 'mañana')",
                "Reminder date and time (e.g: '2025-11-20 15:00', 'today 18:00', 'tomorrow')",
            ),
        );

        // === QUICK NOTES ===
        translations.insert("quick_notes_title", ("Quick Note", "Quick Note"));
        translations.insert(
            "quick_note_back_to_list",
            ("Volver a la lista", "Back to list"),
        );
        translations.insert("quick_note_new", ("Nueva quick note", "New quick note"));
        translations.insert(
            "quick_note_keep_visible",
            ("Mantener visible", "Keep visible"),
        );
        translations.insert("quick_note_close", ("Cerrar (Esc)", "Close (Esc)"));
        translations.insert(
            "quick_note_no_notes",
            ("No hay quick notes aún", "No quick notes yet"),
        );
        translations.insert(
            "quick_note_press_to_create",
            ("Presiona + para crear una", "Press + to create one"),
        );
        translations.insert("quick_note_saved", ("💾 Guardado", "💾 Saved"));
        translations.insert("quick_note_unsaved", ("● Sin guardar", "● Unsaved"));
        translations.insert(
            "quick_note_autosaved",
            ("💾 Auto-guardado", "💾 Auto-saved"),
        );
        translations.insert(
            "quick_note_shortcut_hint",
            ("Ctrl+S: guardar | Esc: cerrar", "Ctrl+S: save | Esc: close"),
        );
        translations.insert(
            "quick_note_created",
            ("Quick note creada", "Quick note created"),
        );

        // === PLAYLIST EXTRAS ===
        translations.insert("playlist_new", ("🎵 Nueva", "🎵 New"));
        translations.insert("playlist_save", ("💾 Guardar", "💾 Save"));
        translations.insert("playlist_clear", ("🗑️ Limpiar", "🗑️ Clear"));
        translations.insert("playlist_queue", ("Cola de reproducción", "Playback queue"));
        translations.insert("playlist_queue_empty", ("Cola vacía", "Queue empty"));
        translations.insert(
            "playlist_saved_playlists",
            ("Playlists guardadas", "Saved playlists"),
        );
        translations.insert("playlist_save_title", ("Guardar Playlist", "Save Playlist"));
        translations.insert(
            "playlist_name_prompt",
            ("Nombre de la playlist:", "Playlist name:"),
        );
        translations.insert(
            "playlist_name_example",
            ("ej: Música relajante", "e.g.: Relaxing music"),
        );
        translations.insert(
            "playlist_no_loaded",
            ("No hay playlist cargada", "No playlist loaded"),
        );
        translations.insert(
            "playlist_no_saved",
            ("No hay playlists guardadas", "No saved playlists"),
        );
        translations.insert("playlist_add_to", ("Agregar a playlist", "Add to playlist"));
        translations.insert("playlist_delete", ("Eliminar playlist", "Delete playlist"));
        translations.insert("play", ("Reproducir", "Play"));
        translations.insert("remove", ("Eliminar", "Remove"));

        // === BÚSQUEDA Y RESULTADOS ===
        translations.insert("searching_ellipsis", ("🔄 Buscando...", "🔄 Searching..."));
        translations.insert(
            "no_results_found",
            ("❌ No se encontraron resultados", "❌ No results found"),
        );
        translations.insert(
            "found_relevant_notes",
            ("Encontré {} notas relevantes:", "Found {} relevant notes:"),
        );
        translations.insert(
            "semantic_results",
            (
                "Resultados por similitud semántica",
                "Results by semantic similarity",
            ),
        );
        translations.insert(
            "ai_analyzing",
            (
                "🔄 El asistente de IA está analizando los resultados...",
                "🔄 The AI assistant is analyzing the results...",
            ),
        );
        translations.insert(
            "assistant_response",
            ("🧠 Respuesta del Asistente", "🧠 Assistant Response"),
        );

        // === UI GENERAL ===
        translations.insert("images_filter", ("Imágenes", "Images"));
        translations.insert(
            "no_models_found",
            ("No se encontraron modelos", "No models found"),
        );
        translations.insert(
            "semantic_search_title",
            (
                "🧠 Búsqueda Semántica (Embeddings)",
                "🧠 Semantic Search (Embeddings)",
            ),
        );
        translations.insert(
            "semantic_search_description",
            (
                "Configura embeddings para búsqueda por significado conceptual usando OpenRouter",
                "Configure embeddings for conceptual meaning search using OpenRouter",
            ),
        );
        translations.insert(
            "enable_embeddings",
            ("Habilitar embeddings:", "Enable embeddings:"),
        );
        translations.insert(
            "index_all_notes",
            ("📄 Indexar todas las notas", "📄 Index all notes"),
        );
        translations.insert("indexing", ("⏳ Indexando...", "⏳ Indexing..."));
        translations.insert(
            "indexing_completed",
            ("Indexación completada", "Indexing completed"),
        );
        translations.insert("unknown_error", ("Error desconocido", "Unknown error"));
        translations.insert(
            "estimated_cost",
            (
                "Costo estimado: ~$0.01 por 10,000 notas",
                "Estimated cost: ~$0.01 per 10,000 notes",
            ),
        );
        translations.insert(
            "get_api_key_openrouter",
            ("Obtener API key en OpenRouter", "Get API key on OpenRouter"),
        );
        translations.insert("status_active", ("🟢 Activo", "🟢 Active"));
        translations.insert("copy_url", ("📋 Copiar URL", "📋 Copy URL"));
        translations.insert(
            "view_docs",
            ("📖 Ver Documentación", "📖 View Documentation"),
        );
        translations.insert("copied", ("✓ Copiado!", "✓ Copied!"));
        translations.insert("you_label", ("Tú", "You"));
        translations.insert(
            "no_notes_in_context",
            ("Sin notas en contexto", "No notes in context"),
        );
        translations.insert(
            "remove_from_context",
            ("Remover del contexto", "Remove from context"),
        );
        translations.insert(
            "operation_success",
            ("✓ Operación exitosa", "✓ Operation successful"),
        );
        translations.insert(
            "operation_failed",
            ("✗ Operación fallida", "✗ Operation failed"),
        );
        translations.insert(
            "youtube_unavailable",
            (
                "Transcripción de YouTube no disponible actualmente...",
                "YouTube transcription not currently available...",
            ),
        );
        translations.insert("music_player", ("Reproductor de música", "Music Player"));

        // === ATAJOS DE TECLADO (extras) ===
        translations.insert(
            "shortcut_open_note_search",
            ("Abrir búsqueda de notas", "Open note search"),
        );
        translations.insert(
            "shortcut_toggle_ai_chat",
            ("Alternar chat IA", "Toggle AI chat"),
        );
        translations.insert(
            "shortcut_delete_char_under",
            (
                "Eliminar carácter bajo el cursor",
                "Delete character under cursor",
            ),
        );
        translations.insert(
            "shortcut_delete_line_complete",
            ("Eliminar línea completa", "Delete entire line"),
        );
        translations.insert(
            "shortcut_delete_prev_char",
            ("Eliminar carácter anterior", "Delete previous character"),
        );
        translations.insert(
            "shortcut_delete_next_char",
            ("Eliminar carácter siguiente", "Delete next character"),
        );
        translations.insert("shortcut_new_line", ("Nueva línea", "New line"));
        translations.insert(
            "shortcut_search_sidebar",
            ("Búsqueda y Sidebar", "Search and Sidebar"),
        );
        translations.insert(
            "shortcut_activate_search",
            ("Activar búsqueda", "Activate search"),
        );
        translations.insert(
            "shortcut_close_search",
            (
                "Cerrar búsqueda / Volver al editor",
                "Close search / Back to editor",
            ),
        );
        translations.insert(
            "shortcut_open_music",
            ("Abrir reproductor de música", "Open music player"),
        );
        translations.insert(
            "shortcut_open_reminders",
            ("Abrir recordatorios", "Open reminders"),
        );

        // === ADDING CONTENT (AI) ===
        translations.insert(
            "adding_content",
            ("Añadiendo contenido...", "Adding content..."),
        );
        translations.insert("adding_tag", ("Añadiendo etiqueta...", "Adding tag..."));
        translations.insert("adding_tags", ("Añadiendo etiquetas...", "Adding tags..."));
        translations.insert(
            "generating_index",
            ("Generando índice...", "Generating index..."),
        );
        translations.insert(
            "extracting_code",
            ("Extrayendo código...", "Extracting code..."),
        );

        // === TOOLTIPS DEL FOOTER E INTERFAZ ===
        translations.insert(
            "tooltip_show_hide_notes",
            ("Mostrar/ocultar lista de notas", "Show/hide notes list"),
        );
        translations.insert("tooltip_new_note", ("Nueva nota", "New note"));
        translations.insert(
            "tooltip_change_search_mode",
            ("Ctrl para cambiar modo", "Ctrl to change mode"),
        );
        translations.insert("tooltip_close_esc", ("Cerrar (Esc)", "Close (Esc)"));
        translations.insert("tooltip_note_tags", ("Tags de la nota", "Note tags"));
        translations.insert("tooltip_note_todos", ("TODOs de la nota", "Note TODOs"));
        translations.insert(
            "tooltip_music_player",
            ("Reproductor de música", "Music player"),
        );
        translations.insert(
            "tooltip_reminders",
            ("Recordatorios (Alt+R)", "Reminders (Alt+R)"),
        );
        translations.insert("tooltip_settings", ("Ajustes", "Settings"));

        // === AI SETTINGS ===
        translations.insert("temperature_label", ("Temperatura:", "Temperature:"));
        translations.insert("max_tokens_label", ("Max Tokens:", "Max Tokens:"));
        translations.insert("unlimited", ("Ilimitado", "Unlimited"));
        translations.insert(
            "save_history_label",
            ("Guardar historial:", "Save history:"),
        );
        translations.insert("model_label", ("Modelo:", "Model:"));
        translations.insert(
            "refresh_models_tooltip",
            (
                "Actualizar lista de modelos desde OpenRouter",
                "Refresh model list from OpenRouter",
            ),
        );
        translations.insert(
            "search_model_placeholder",
            ("Buscar modelo...", "Search model..."),
        );

        // === ALERTAS DE MODO AGENTE/CHAT ===
        translations.insert(
            "agent_mode_activated",
            (
                "Modo Agente activado\nEl asistente puede buscar en notas y ejecutar acciones",
                "Agent Mode activated\nThe assistant can search notes and execute actions",
            ),
        );
        translations.insert(
            "chat_mode_activated",
            (
                "Chat Normal activado\nConversación directa sin acceso a herramientas",
                "Normal Chat activated\nDirect conversation without tool access",
            ),
        );
        translations.insert(
            "analyzing_task",
            ("Analizando tarea...", "Analyzing task..."),
        );

        // === MCP SERVER DIALOG ===
        translations.insert(
            "mcp_server_title",
            (
                "MCP Server - Model Context Protocol",
                "MCP Server - Model Context Protocol",
            ),
        );
        translations.insert(
            "mcp_server_active",
            ("MCP Server Activo", "MCP Server Active"),
        );
        translations.insert(
            "mcp_server_subtitle",
            (
                "Exponiendo herramientas de NotNative via HTTP",
                "Exposing NotNative tools via HTTP",
            ),
        );
        translations.insert("mcp_status", ("Estado", "Status"));
        translations.insert(
            "mcp_endpoints_available",
            ("Endpoints disponibles", "Available endpoints"),
        );

        // === KEYBOARD SHORTCUTS SECTIONS ===
        translations.insert("shortcuts_global", ("🌍 Globales", "🌍 Global"));
        translations.insert(
            "shortcuts_quick_notes",
            ("📝 Quick Notes", "📝 Quick Notes"),
        );
        translations.insert(
            "shortcuts_normal_navigation",
            ("📝 Modo Normal - Navegación", "📝 Normal Mode - Navigation"),
        );
        translations.insert(
            "shortcuts_normal_editing",
            (
                "📝 Modo Normal - Edición y Modos",
                "📝 Normal Mode - Editing and Modes",
            ),
        );
        translations.insert(
            "shortcuts_insert_mode",
            ("✍️ Modo Insertar", "✍️ Insert Mode"),
        );
        translations.insert("shortcuts_ai_chat", ("🤖 Modo Chat AI", "🤖 AI Chat Mode"));
        translations.insert(
            "shortcuts_sidebar",
            ("📂 Sidebar y Listas", "📂 Sidebar and Lists"),
        );
        translations.insert(
            "shortcuts_floating_search",
            ("🔍 Búsqueda Flotante", "🔍 Floating Search"),
        );

        // === KEYBOARD SHORTCUTS DESCRIPTIONS ===
        translations.insert(
            "shortcut_global_search",
            (
                "Abrir búsqueda global flotante",
                "Open floating global search",
            ),
        );
        translations.insert(
            "shortcut_note_search",
            (
                "Abrir búsqueda dentro de la nota actual",
                "Open search within current note",
            ),
        );
        translations.insert(
            "shortcut_enter_ai_chat",
            ("Entrar al modo Chat AI", "Enter AI Chat mode"),
        );
        translations.insert(
            "shortcut_back_or_close",
            (
                "Volver a lista / Cerrar ventana",
                "Back to list / Close window",
            ),
        );
        translations.insert(
            "shortcut_left",
            ("Mover cursor a la izquierda", "Move cursor left"),
        );
        translations.insert("shortcut_down", ("Mover cursor abajo", "Move cursor down"));
        translations.insert("shortcut_up", ("Mover cursor arriba", "Move cursor up"));
        translations.insert(
            "shortcut_right",
            ("Mover cursor a la derecha", "Move cursor right"),
        );
        translations.insert(
            "shortcut_ai_chat_mode",
            ("Entrar en Modo Chat AI", "Enter AI Chat Mode"),
        );
        translations.insert("shortcut_new_note", ("Crear nueva nota", "Create new note"));
        translations.insert(
            "shortcut_insert_table",
            ("Insertar tabla Markdown", "Insert Markdown table"),
        );
        translations.insert("shortcut_insert_image", ("Insertar imagen", "Insert image"));
        translations.insert(
            "shortcut_tab_autocomplete",
            (
                "Tabulación / Autocompletar Tag o @",
                "Tab / Autocomplete Tag or @",
            ),
        );
        translations.insert(
            "shortcut_exit_chat",
            (
                "Salir del Chat (volver a Modo Normal)",
                "Exit Chat (back to Normal Mode)",
            ),
        );
        translations.insert(
            "shortcut_exit_chat_insert",
            (
                "Salir del Chat y entrar a Modo Insertar",
                "Exit Chat and enter Insert Mode",
            ),
        );
        translations.insert("shortcut_send_message", ("Enviar mensaje", "Send message"));
        translations.insert(
            "shortcut_navigate_suggestions",
            ("Navegar sugerencias", "Navigate suggestions"),
        );
        translations.insert(
            "shortcut_accept_suggestion",
            ("Aceptar sugerencia", "Accept suggestion"),
        );
        translations.insert("shortcut_next_note", ("Siguiente nota", "Next note"));
        translations.insert("shortcut_prev_note", ("Nota anterior", "Previous note"));
        translations.insert(
            "shortcut_open_note",
            (
                "Abrir nota o carpeta seleccionada",
                "Open selected note or folder",
            ),
        );
        translations.insert(
            "shortcut_focus_editor",
            ("Devolver foco al editor", "Return focus to editor"),
        );
        translations.insert(
            "shortcut_toggle_semantic",
            (
                "Alternar búsqueda semántica (AI)",
                "Toggle semantic search (AI)",
            ),
        );
        translations.insert(
            "shortcut_navigate_results",
            ("Navegar resultados", "Navigate results"),
        );
        translations.insert(
            "shortcut_open_selected",
            ("Abrir nota seleccionada", "Open selected note"),
        );

        // === BASES DE DATOS ===
        translations.insert("base_add_filter", ("Añadir filtro", "Add filter"));
        translations.insert("base_sort", ("Ordenar", "Sort"));
        translations.insert("base_columns", ("Columnas", "Columns"));
        translations.insert(
            "base_columns_config",
            ("Configurar Columnas", "Configure Columns"),
        );
        translations.insert("base_data_source", ("Origen de datos", "Data source mode"));
        translations.insert("base_formula_rows", ("Filas con fórmulas", "Formula rows"));
        translations.insert(
            "base_formula_rows_title",
            ("Filas de Fórmulas", "Formula Rows"),
        );
        translations.insert(
            "base_add_formula_row",
            ("Añadir fila de totales", "Add totals row"),
        );
        translations.insert("base_formula_row_label", ("Etiqueta", "Label"));
        translations.insert(
            "base_formula_help",
            (
                "Usa fórmulas tipo Excel: =SUM(B:B), =AVG(C1:C10)",
                "Use Excel-like formulas: =SUM(B:B), =AVG(C1:C10)",
            ),
        );
        translations.insert("base_export_xlsx", ("Exportar a Excel", "Export to Excel"));
        translations.insert(
            "base_export_xlsx_success",
            ("Exportado correctamente", "Exported successfully"),
        );
        translations.insert(
            "base_export_xlsx_error",
            ("Error al exportar", "Export error"),
        );
        translations.insert(
            "base_show_graph",
            ("Mostrar grafo de relaciones", "Show relationships graph"),
        );
        translations.insert("base_no_filters", ("Sin filtros", "No filters"));
        translations.insert(
            "base_current_columns",
            ("Columnas visibles", "Visible Columns"),
        );
        translations.insert(
            "base_available_properties",
            ("Propiedades disponibles", "Available Properties"),
        );
        translations.insert(
            "base_toggle_visibility",
            ("Alternar visibilidad", "Toggle visibility"),
        );
        translations.insert("base_remove_column", ("Eliminar columna", "Remove column"));
        translations.insert(
            "base_add_column",
            ("Propiedades disponibles", "Available Properties"),
        );
        translations.insert(
            "base_properties_hint",
            (
                "Clic en + para añadir como columna",
                "Click + to add as column",
            ),
        );
        translations.insert(
            "base_add_as_column",
            ("Añadir como columna", "Add as column"),
        );
        translations.insert("base_move_up", ("Mover arriba", "Move up"));
        translations.insert("base_move_down", ("Mover abajo", "Move down"));
        translations.insert("base_data_source_title", ("Origen de datos", "Data Source"));
        translations.insert("base_notes_mode", ("Notas", "Notes"));
        translations.insert(
            "base_grouped_mode",
            ("Registros agrupados", "Grouped Records"),
        );
        translations.insert("base_grouped_hint", ("Los registros agrupados muestran propiedades\ninline como [game::X, bought::Y] como filas", "Grouped Records shows inline property\ngroups like [game::X, bought::Y] as rows"));
        translations.insert(
            "base_search_placeholder",
            ("Buscar en tabla...", "Search in table..."),
        );
        translations.insert(
            "base_no_notes",
            ("No se encontraron notas", "No notes found"),
        );
        translations.insert("base_items", ("elementos", "items"));
        translations.insert("base_items_of", ("de", "of"));
        translations.insert("base_title", ("Título", "Title"));
        translations.insert("base_created", ("Creado", "Created"));
        translations.insert("base_modified", ("Modificado", "Modified"));
        translations.insert("base_tags", ("Etiquetas", "Tags"));
        translations.insert(
            "base_search_properties",
            ("Buscar propiedades...", "Search properties..."),
        );
        translations.insert("base_move_up", ("Mover arriba", "Move up"));
        translations.insert("base_move_down", ("Mover abajo", "Move down"));
        translations.insert(
            "base_no_available_props",
            (
                "No hay más propiedades disponibles",
                "No more properties available",
            ),
        );

        // === SORT POPOVER ===
        translations.insert("base_sort_by", ("Ordenar por", "Sort by"));
        translations.insert("base_no_sorting", ("Sin ordenar", "No sorting"));
        translations.insert(
            "base_sort_ascending",
            ("Orden ascendente", "Sort ascending"),
        );
        translations.insert(
            "base_sort_descending",
            ("Orden descendente", "Sort descending"),
        );

        // === FILTER POPOVER ===
        translations.insert("base_add_filter_title", ("Añadir filtro", "Add Filter"));
        translations.insert("base_property", ("Propiedad", "Property"));
        translations.insert("base_operator", ("Operador", "Operator"));
        translations.insert("base_value", ("Valor", "Value"));
        translations.insert(
            "base_filter_value_placeholder",
            ("Valor del filtro...", "Filter value..."),
        );
        translations.insert("base_cancel", ("Cancelar", "Cancel"));
        translations.insert("base_apply_filter", ("Aplicar filtro", "Add Filter"));

        // Operadores de filtro
        translations.insert("filter_op_equals", ("igual a", "equals"));
        translations.insert("filter_op_not_equals", ("distinto de", "not equals"));
        translations.insert("filter_op_contains", ("contiene", "contains"));
        translations.insert("filter_op_not_contains", ("no contiene", "not contains"));
        translations.insert("filter_op_greater_than", ("mayor que", "greater than"));
        translations.insert(
            "filter_op_greater_or_equal",
            ("mayor o igual", "greater or equal"),
        );
        translations.insert("filter_op_less_than", ("menor que", "less than"));
        translations.insert(
            "filter_op_less_or_equal",
            ("menor o igual", "less or equal"),
        );
        translations.insert("filter_op_starts_with", ("empieza con", "starts with"));
        translations.insert("filter_op_ends_with", ("termina con", "ends with"));
        translations.insert("filter_op_is_empty", ("está vacío", "is empty"));
        translations.insert("filter_op_is_not_empty", ("no está vacío", "is not empty"));

        Self {
            language,
            translations,
        }
    }

    pub fn t(&self, key: &str) -> String {
        self.translations
            .get(key)
            .map(|(es, en)| match self.language {
                Language::Spanish => *es,
                Language::English => *en,
            })
            .unwrap_or(key)
            .to_string()
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    pub fn current_language(&self) -> Language {
        self.language
    }

    /// Obtiene todas las traducciones disponibles para una clave
    pub fn all_translations(&self, key: &str) -> Option<(String, String)> {
        self.translations
            .get(key)
            .map(|(es, en)| (es.to_string(), en.to_string()))
    }
}
