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

#[derive(Debug)]
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
        translations.insert("show_hide_notes", ("Mostrar/ocultar lista de notas", "Show/hide notes list"));
        translations.insert("preferences", ("Preferencias", "Preferences"));
        translations.insert("keyboard_shortcuts", ("Atajos de teclado", "Keyboard Shortcuts"));
        translations.insert("about", ("Acerca de", "About"));
        translations.insert("settings", ("Ajustes", "Settings"));
        translations.insert("tags", ("Tags", "Tags"));
        translations.insert("tags_note", ("Tags de la nota", "Note tags"));
        translations.insert("no_tags", ("No hay tags", "No tags"));
        translations.insert("search_tag", ("Buscar notas con este tag", "Search notes with this tag"));
        translations.insert("remove_tag", ("Eliminar tag", "Remove tag"));
        translations.insert("close", ("Cerrar", "Close"));
        
        // Diálogos
        translations.insert("create_note_title", ("Nueva nota", "New Note"));
        translations.insert("note_name_hint", ("ejemplo: proyectos/nueva-idea", "example: projects/new-idea"));
        translations.insert("create_folder_hint", ("Usa '/' para crear en carpetas", "Use '/' to create in folders"));
        translations.insert("create", ("Crear", "Create"));
        translations.insert("cancel", ("Cancelar", "Cancel"));
        translations.insert("rename", ("Renombrar", "Rename"));
        translations.insert("delete", ("Eliminar", "Delete"));
        translations.insert("confirm_delete", ("¿Estás seguro de eliminar", "Are you sure you want to delete"));
        
        // Preferencias
        translations.insert("theme", ("Tema", "Theme"));
        translations.insert("theme_sync", ("La aplicación sincroniza automáticamente con el tema Omarchy", "The app automatically syncs with Omarchy theme"));
        translations.insert("markdown_rendering", ("Renderizado Markdown", "Markdown Rendering"));
        translations.insert("markdown_enabled", ("Activado por defecto en modo Normal", "Enabled by default in Normal mode"));
        translations.insert("language", ("Idioma", "Language"));
        translations.insert("language_description", ("Elige el idioma de la interfaz", "Choose the interface language"));
        translations.insert("restart_required", ("Se requiere reiniciar la aplicación", "Application restart required"));
        
        // Atajos de teclado
        translations.insert("shortcuts_general", ("General", "General"));
        translations.insert("shortcuts_modes", ("Modos de edición", "Editing Modes"));
        translations.insert("shortcuts_navigation", ("Navegación", "Navigation"));
        translations.insert("shortcuts_editing", ("Edición", "Editing"));
        
        translations.insert("shortcut_new_note", ("Nueva nota", "New note"));
        translations.insert("shortcut_save", ("Guardar nota", "Save note"));
        translations.insert("shortcut_search", ("Buscar notas", "Search notes"));
        translations.insert("shortcut_toggle_sidebar", ("Alternar sidebar", "Toggle sidebar"));
        translations.insert("shortcut_escape", ("Volver al editor", "Back to editor"));
        
        translations.insert("shortcut_insert_mode", ("Modo Insert", "Insert mode"));
        translations.insert("shortcut_normal_mode", ("Modo Normal", "Normal mode"));
        translations.insert("shortcut_command_mode", ("Modo Command", "Command mode"));
        translations.insert("shortcut_visual_mode", ("Modo Visual", "Visual mode"));
        
        translations.insert("shortcut_movement", ("Izquierda/Abajo/Arriba/Derecha", "Left/Down/Up/Right"));
        translations.insert("shortcut_next_word", ("Siguiente palabra", "Next word"));
        translations.insert("shortcut_prev_word", ("Palabra anterior", "Previous word"));
        translations.insert("shortcut_line_start", ("Inicio de línea", "Start of line"));
        translations.insert("shortcut_line_end", ("Fin de línea", "End of line"));
        translations.insert("shortcut_doc_start", ("Inicio del documento", "Start of document"));
        translations.insert("shortcut_doc_end", ("Fin del documento", "End of document"));
        
        translations.insert("shortcut_delete_char", ("Eliminar carácter", "Delete character"));
        translations.insert("shortcut_delete_line", ("Eliminar línea", "Delete line"));
        translations.insert("shortcut_undo", ("Deshacer", "Undo"));
        translations.insert("shortcut_redo", ("Rehacer", "Redo"));
        
        // About
        translations.insert("app_description", ("Editor de notas markdown con estilo vim", "Vim-style markdown note editor"));
        translations.insert("website", ("Sitio web", "Website"));
        translations.insert("authors", ("Autores", "Authors"));
        translations.insert("version", ("Versión", "Version"));
        translations.insert("license", ("Licencia", "License"));
        
        // Búsqueda
        translations.insert("no_results", ("No se encontraron resultados para", "No results found for"));
        translations.insert("searching", ("Buscando", "Searching"));
        
        // Estados
        translations.insert("lines", ("líneas", "lines"));
        translations.insert("words", ("palabras", "words"));
        translations.insert("characters", ("caracteres", "characters"));
        translations.insert("saved", ("Guardado", "Saved"));
        translations.insert("unsaved_changes", ("Cambios sin guardar", "Unsaved changes"));
        
        // Mensajes
        translations.insert("note_created", ("Nota creada", "Note created"));
        translations.insert("note_deleted", ("Nota eliminada", "Note deleted"));
        translations.insert("note_renamed", ("Nota renombrada", "Note renamed"));
        translations.insert("error", ("Error", "Error"));
        translations.insert("success", ("Éxito", "Success"));
        
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
