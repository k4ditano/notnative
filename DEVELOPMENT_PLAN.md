# Plan de Desarrollo - NotNative v0.2

**Fecha**: Noviembre 2025  
**Estado**: En planificación  
**Versión objetivo**: 0.2.0

## 🎯 Objetivos v0.2

1. **Drag & Drop en Sidebar** - Reordenar y organizar notas visualmente
2. **Indexación con SQLite** - Base de datos para búsqueda rápida
3. **Búsqueda Full-Text** - Encontrar notas por contenido/tags/nombre
4. **Sistema de Tags** - Organizar con tags y autocompletado

---

## 📋 Orden de Implementación

### Fase 1: Base de Datos e Indexación (Fundación)
**Prioridad**: CRÍTICA - Todo depende de esto

#### 1.1 Crear esquema SQLite
**Archivos**: `src/core/database.rs`

```sql
-- Tabla principal de notas
CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    path TEXT NOT NULL UNIQUE,
    folder TEXT,
    content TEXT,
    order_index INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Tabla de tags
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    usage_count INTEGER DEFAULT 0
);

-- Relación many-to-many
CREATE TABLE note_tags (
    note_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (note_id, tag_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Full-text search (FTS5)
CREATE VIRTUAL TABLE notes_fts USING fts5(
    name,
    content,
    tokenize = 'porter unicode61'
);

-- Índices
CREATE INDEX idx_notes_folder ON notes(folder);
CREATE INDEX idx_notes_updated ON notes(updated_at DESC);
CREATE INDEX idx_tags_usage ON tags(usage_count DESC);
```

**Tasks**:
- [ ] Crear módulo `database.rs` con struct `NotesDatabase`
- [ ] Implementar conexión a SQLite con `rusqlite`
- [ ] Crear método `initialize_schema()` para setup inicial
- [ ] Implementar sistema de migraciones (version tracking)
- [ ] Añadir `rusqlite` a Cargo.toml

#### 1.2 Implementar operaciones CRUD
**Funciones necesarias**:

```rust
impl NotesDatabase {
    pub fn new(path: &Path) -> Result<Self>;
    pub fn index_note(&self, note: &NoteFile) -> Result<()>;
    pub fn update_note(&self, note: &NoteFile) -> Result<()>;
    pub fn delete_note(&self, name: &str) -> Result<()>;
    pub fn get_note(&self, name: &str) -> Result<Option<NoteMetadata>>;
    pub fn list_notes(&self, folder: Option<&str>) -> Result<Vec<NoteMetadata>>;
    pub fn search_notes(&self, query: &str) -> Result<Vec<SearchResult>>;
    pub fn get_tags(&self) -> Result<Vec<Tag>>;
    pub fn add_tag(&self, note_id: i64, tag_name: &str) -> Result<()>;
    pub fn remove_tag(&self, note_id: i64, tag_name: &str) -> Result<()>;
}
```

**Tasks**:
- [ ] Implementar todas las funciones CRUD
- [ ] Añadir manejo de errores con `thiserror`
- [ ] Tests unitarios para cada operación

#### 1.3 Indexación inicial y watcher
**Integración con el sistema existente**:

```rust
// En MainApp::init()
let db = NotesDatabase::new(&notes_dir.db_path())?;
db.index_all_notes(&notes_dir)?;

// Watcher de archivos
let watcher = NotesWatcher::new(notes_dir.path(), sender.clone());
```

**Tasks**:
- [ ] Indexar todas las notas al iniciar la app
- [ ] Implementar watcher con `notify` para detectar cambios
- [ ] Re-indexar automáticamente al guardar
- [ ] Manejo de conflictos (archivo vs DB)

---

### Fase 2: Sistema de Tags y Frontmatter
**Prioridad**: ALTA - Permite organización avanzada

#### 2.1 Parser de frontmatter YAML
**Archivo**: `src/core/frontmatter.rs`

```rust
#[derive(Debug, Clone)]
pub struct Frontmatter {
    pub tags: Vec<String>,
    pub title: Option<String>,
    pub date: Option<String>,
    pub custom: HashMap<String, String>,
}

impl Frontmatter {
    pub fn parse(content: &str) -> Result<(Self, &str)>;
    pub fn serialize(&self) -> String;
}
```

**Formato esperado**:
```markdown
---
tags: [rust, notas, gtk]
title: Mi nota importante
date: 2025-11-01
---

# Contenido de la nota...
```

**Tasks**:
- [ ] Añadir `serde_yaml` a Cargo.toml
- [ ] Implementar parser que detecte `---` al inicio
- [ ] Separar frontmatter de contenido
- [ ] Extraer tags y otros metadatos
- [ ] Método para serializar de vuelta a YAML
- [ ] Tests con casos edge (sin frontmatter, malformado, etc.)

#### 2.2 UI de tags
**Ubicación**: Footer o panel lateral en el editor

**Widgets GTK**:
```rust
// En MainApp widgets
tags_box: gtk::Box,          // Container horizontal de tags
tags_entry: gtk::Entry,      // Input para añadir tags
tags_completion: gtk::EntryCompletion, // Autocompletado
```

**Tasks**:
- [ ] Añadir box de tags en footer o header
- [ ] Entry con placeholder "Añadir tag..."
- [ ] EntryCompletion con sugerencias de tags existentes
- [ ] Mostrar tags actuales como botones/pills removibles
- [ ] Click en tag → filtrar notas en sidebar
- [ ] Persistir tags en frontmatter al guardar

#### 2.3 Vista de tags en sidebar
**Tasks**:
- [ ] Sección colapsable "Tags" en sidebar
- [ ] Listar tags con contador de uso
- [ ] Click en tag → filtrar notas
- [ ] Color coding opcional por tag

---

### Fase 3: Búsqueda Full-Text
**Prioridad**: ALTA - Funcionalidad core

#### 3.1 UI de búsqueda
**Ubicación**: Header del sidebar

```rust
// En MainApp widgets
search_bar: gtk::SearchBar,
search_entry: gtk::SearchEntry,
search_results: gtk::ListBox,
```

**Tasks**:
- [ ] Añadir SearchBar en header del sidebar
- [ ] SearchEntry con icono de lupa
- [ ] Botón para toggle search bar
- [ ] Shortcut `Ctrl+F` para activar búsqueda
- [ ] Placeholder "Buscar notas..."

#### 3.2 Motor de búsqueda
**Archivo**: `src/core/search.rs`

```rust
pub struct SearchQuery {
    pub text: Option<String>,
    pub tags: Vec<String>,
    pub folder: Option<String>,
    pub date_from: Option<DateTime>,
    pub date_to: Option<DateTime>,
}

pub struct SearchResult {
    pub note_id: i64,
    pub note_name: String,
    pub snippet: String,      // Fragmento con highlight
    pub relevance: f32,       // Score de FTS5
    pub matched_tags: Vec<String>,
}

impl NotesDatabase {
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
}
```

**Tasks**:
- [ ] Implementar query builder para FTS5
- [ ] Generar snippets con contexto (±50 chars)
- [ ] Resaltar coincidencias en snippets
- [ ] Ordenar por relevancia (FTS5 rank)
- [ ] Combinar búsqueda por texto + filtros (tags, fecha)

#### 3.3 Vista de resultados
**Tasks**:
- [ ] ListBox para mostrar resultados
- [ ] Cada fila: nombre, snippet, tags, fecha
- [ ] Click en resultado → cargar nota
- [ ] Resaltar términos buscados en el snippet
- [ ] Indicador de "X resultados encontrados"
- [ ] Botón para limpiar búsqueda y volver a vista normal

#### 3.4 Búsqueda en tiempo real
**Tasks**:
- [ ] Debounce de 300ms en el Entry
- [ ] Spinner mientras busca
- [ ] Búsqueda asíncrona (no bloquear UI)
- [ ] Cancelar búsqueda anterior si se escribe nuevo texto

---

### Fase 4: Drag & Drop en Sidebar
**Prioridad**: MEDIA - Mejora UX pero no bloqueante

#### 4.1 Drag source
**Implementación**: En cada fila del ListBox

```rust
// En populate_notes_list()
let drag_source = gtk::DragSource::new();
drag_source.set_actions(gtk::gdk::DragAction::MOVE);

drag_source.connect_prepare(|source, x, y| {
    // Obtener datos de la nota/carpeta
    let note_name = get_note_name_from_source(source);
    let content_provider = gtk::gdk::ContentProvider::for_value(&note_name);
    Some(content_provider)
});

drag_source.connect_drag_begin(|source, drag| {
    // Visual feedback (icono, etc.)
});

row.add_controller(drag_source);
```

**Tasks**:
- [ ] Añadir DragSource a cada fila de nota
- [ ] Añadir DragSource a cada fila de carpeta
- [ ] Serializar datos de la fila (nombre, tipo, path)
- [ ] Feedback visual: cambiar opacidad, mostrar icono
- [ ] Icono personalizado durante el drag

#### 4.2 Drop target
**Implementación**: En el ListBox y en filas de carpetas

```rust
let drop_target = gtk::DropTarget::new(glib::Type::STRING, gtk::gdk::DragAction::MOVE);

drop_target.connect_drop(|target, value, x, y| {
    let note_name = value.get::<String>().ok()?;
    let drop_row = target.widget().downcast::<gtk::ListBoxRow>().ok()?;
    
    // Determinar tipo de drop
    if is_drop_on_folder(&drop_row) {
        // Mover nota a carpeta
        move_note_to_folder(&note_name, &folder_name)?;
    } else {
        // Reordenar entre notas
        reorder_note(&note_name, new_index)?;
    }
    
    true
});

drop_target.connect_motion(|target, x, y| {
    // Highlight visual de dónde se va a soltar
});
```

**Tasks**:
- [ ] Añadir DropTarget al ListBox completo
- [ ] Añadir DropTarget a filas de carpetas
- [ ] Detectar tipo de drop (reordenar vs mover a carpeta)
- [ ] Calcular posición de inserción
- [ ] Highlight visual durante hover (línea, highlight de carpeta)
- [ ] Prevenir drops inválidos (carpeta sobre sí misma, etc.)

#### 4.3 Actualización de estructura
**Tasks**:
- [ ] Mover archivos físicamente en disco
- [ ] Actualizar paths en base de datos
- [ ] Actualizar `order_index` en DB para reordenamiento
- [ ] Refrescar sidebar después del drop
- [ ] Animación suave de reordenamiento
- [ ] Manejo de errores (permisos, IO)

#### 4.4 Casos especiales
**Tasks**:
- [ ] Drag de carpeta sobre carpeta (anidar)
- [ ] Prevenir anidación circular
- [ ] Drag múltiple (opcional, futuro)
- [ ] Undo para drag & drop (opcional)

---

## 🔧 Dependencias Nuevas

Añadir a `Cargo.toml`:

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled", "column_decltype"] }
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
```

---

## 🧪 Testing

### Tests por módulo

1. **database.rs**
   - Crear y conectar a DB
   - CRUD de notas
   - FTS5 queries
   - Manejo de errores

2. **frontmatter.rs**
   - Parse válido
   - Parse con errores
   - Serialización roundtrip

3. **search.rs**
   - Búsqueda simple
   - Búsqueda con filtros
   - Snippets correctos
   - Ranking de resultados

### Tests de integración

```rust
#[test]
fn test_search_workflow() {
    // 1. Crear notas con tags
    // 2. Indexar en DB
    // 3. Buscar por texto
    // 4. Verificar resultados
}

#[test]
fn test_drag_drop_workflow() {
    // 1. Crear estructura de carpetas
    // 2. Simular drag & drop
    // 3. Verificar archivos movidos
    // 4. Verificar DB actualizada
}
```

---

## 📊 Métricas de Éxito

- [ ] Búsqueda responde en < 100ms para 1000 notas
- [ ] Indexación inicial < 5s para 1000 notas
- [ ] Drag & drop funciona sin errores en GTK
- [ ] Autocompletado de tags muestra sugerencias
- [ ] UI responsive durante búsqueda (no bloquea)

---

## 🐛 Riesgos y Mitigaciones

### Riesgo 1: Performance de SQLite en Wayland
**Mitigación**: Usar queries asíncronas con `tokio::spawn_blocking`

### Riesgo 2: Drag & Drop complejo en GTK4
**Mitigación**: Empezar con caso simple (reordenar notas), luego añadir carpetas

### Riesgo 3: Conflictos entre archivos y DB
**Mitigación**: El archivo es la fuente de verdad, DB es solo índice (reconstruible)

### Riesgo 4: Parsing YAML malformado
**Mitigación**: Ignorar frontmatter inválido, continuar sin tags

---

## 📝 Orden de Trabajo Recomendado

1. ✅ **Actualizar README** (hecho)
2. 🔥 **Base de datos** (crítico, 3-4 días)
   - Esquema SQL
   - Módulo database.rs
   - CRUD operations
   - Tests

3. 🔥 **Indexación** (crítico, 2 días)
   - Indexar al inicio
   - Watcher de archivos
   - Re-indexación automática

4. 🔥 **Frontmatter y tags** (2 días)
   - Parser YAML
   - UI de tags
   - Integración con DB

5. 🔥 **Búsqueda** (3 días)
   - UI SearchBar
   - Motor FTS5
   - Vista de resultados
   - Debounce y async

6. ⚡ **Drag & Drop** (3-4 días)
   - DragSource en filas
   - DropTarget en ListBox
   - Mover archivos
   - Actualizar DB
   - Feedback visual

**Total estimado**: ~2-3 semanas de desarrollo

---

## 🚀 Plan de Releases

- **v0.2.0-alpha.1** - Base de datos e indexación
- **v0.2.0-alpha.2** - Tags y búsqueda básica
- **v0.2.0-beta.1** - Búsqueda avanzada + drag & drop
- **v0.2.0** - Release estable con todas las features

---

## 📚 Referencias Técnicas

- [GTK4 Drag and Drop](https://docs.gtk.org/gtk4/drag-and-drop.html)
- [SQLite FTS5 Documentation](https://www.sqlite.org/fts5.html)
- [rusqlite crate](https://docs.rs/rusqlite/)
- [serde_yaml crate](https://docs.rs/serde_yaml/)
