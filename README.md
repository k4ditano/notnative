# NotNative

Una aplicación de notas **nativa** para escritorio Linux con soporte para markdown, comandos estilo vim y diseñada para máxima velocidad y eficiencia.

## 🎯 Características

### ✅ Implementado (v0.1)

#### Editor de texto con sistema modal vim
- **Buffer de texto ultrarrápido** basado en `ropey` con operaciones O(log n)
- **Sistema de comandos modal** inspirado en vim (Normal/Insert/Command/Visual)
- **Undo/Redo granular** con historial de 1000 operaciones
- **Navegación vim** completa: `h/j/k/l`, `0/$`, `gg/G`
- **Edición**: `x` (delete char), `dd` (delete line), `i` (insert mode)

#### Interfaz GTK4 + Temas
- **Interfaz nativa GTK4** sin libadwaita (GTK puro)
- **Integración con temas del sistema** - Omarchy theme auto-detectado
- **Watcher de temas** - Actualización automática al cambiar tema del sistema
- **Márgenes optimizados** - Espaciado visual mejorado en TextView y HeaderBar
- **Barra de estado** con indicador de modo y estadísticas en tiempo real

#### Sistema de archivos y persistencia
- **Sistema de archivos .md** - Cada nota se guarda como archivo markdown independiente
- **Persistencia automática** - Las notas se guardan en `~/.local/share/notnative/notes/`
- **Autoguardado inteligente** - Guarda cada 5 segundos y al cerrar (solo si hay cambios)
- **Indicadores visuales** - Muestra `●` en título cuando hay cambios sin guardar
- **Gestión de notas** - Crear, cargar, guardar y listar notas .md
- **Nota de bienvenida** - Se crea automáticamente la primera vez que se ejecuta la app
- **Título dinámico** - La ventana muestra el nombre de la nota actual

#### Renderizado Markdown
- **Renderizado markdown en tiempo real** - Vista limpia sin símbolos en modo Normal
- **Parser robusto** con `pulldown-cmark` - Maneja offsets correctamente
- **Soporte de sintaxis**: 
  - Headings (`#`, `##`, `###`)
  - Bold (`**texto**`) e Italic (`*texto*`)
  - Código inline (`` `código` ``) y bloques (` ``` `)
  - Links clickeables (`[texto](url)`)
  - Listas (`-` con bullets `•`)
  - Blockquotes (`>`)
- **Modo dual**: 
  - Modo Normal: Vista limpia sin símbolos markdown
  - Modo Insert: Texto crudo con todos los símbolos visibles
- **Estilos GTK TextTags** - Adaptados al tema del sistema
- **Links interactivos** - Click para abrir en navegador, cursor pointer al hover

#### Sidebar y navegación
- **Sidebar deslizante** con animaciones suaves de apertura/cierre
- **Sistema de carpetas** - Organización jerárquica de notas
- **Carpetas expandibles** - Click para expandir/colapsar
- **Navegación con teclado** - `j/k` para moverse, `l/Esc` para cerrar
- **Hover para cargar** - Pasa el mouse sobre una nota y se carga automáticamente
- **Menú contextual** - Click derecho para renombrar/eliminar (en desarrollo)
- **Shortcuts** - `Ctrl+E` para toggle, botón en header

#### Teclado y eventos
- **Eventos de teclado** integrados con el sistema de comandos
- **Composición de acentos** - Soporte completo para caracteres especiales (á, é, í, ó, ú, ñ)
- **Todos los caracteres especiales** funcionan correctamente (.,!?:;/etc)
- **Shortcuts globales**: `Ctrl+S` (guardar), `Ctrl+D` (cambiar tema), `Ctrl+E` (sidebar)

### 🚧 En desarrollo

- Drag & drop en sidebar (reordenar notas, mover entre carpetas)
- Sistema de indexación con SQLite
- Búsqueda full-text y filtrado de notas
- Sistema de tags con autocompletado

## 🚀 Instalación

### Requisitos

- Rust 1.70+
- GTK4
- libadwaita

### Fuentes (opcional, para Modo 8BIT)

Para usar el **Modo 8BIT** con fuentes retro, instala las fuentes incluidas:

```bash
./install-fonts.sh
```

Esto instalará VT323 (fuente de terminal retro) en tu sistema. Ver `fonts/README.md` para más detalles.

### Compilar

```bash
cargo build --release
```

### Ejecutar

```bash
cargo run --release
```

## ⌨️ Atajos de teclado

### Modo Normal (predeterminado)

- `i` - Entrar en modo INSERT
- `:` - Entrar en modo COMMAND
- `h/j/k/l` - Mover cursor (izq/abajo/arriba/der)
- `0` - Inicio de línea
- `$` - Fin de línea
- `gg` - Inicio del documento
- `G` - Fin del documento
- `x` - Eliminar carácter
- `dd` - Eliminar línea
- `u` - Deshacer
- `Ctrl+z` - Deshacer
- `Ctrl+r` - Rehacer
- `Ctrl+s` - Guardar
- `Ctrl+d` - Cambiar tema

### Modo Insert

- `Esc` - Volver a modo NORMAL
- `Ctrl+s` - Guardar
- Todas las teclas normales insertan texto

### Modo Command

- `:w` - Guardar
- `:q` - Salir
- `:wq` - Guardar y salir
- `:q!` - Salir sin guardar

### Interfaz

- **Botón 8BIT** (footer) - Activa/desactiva el modo retro con fuentes pixeladas
- **Menú Ajustes** (⚙️) - Acceso a preferencias y configuración
- **Indicador de modo** (footer izquierda) - Muestra el modo actual (NORMAL/INSERT)
- **Estadísticas** (footer derecha) - Líneas, palabras y cambios sin guardar

## 🏗️ Arquitectura

```
src/
├── main.rs              # Bootstrap, GTK init, carga de temas Omarchy
├── app.rs               # Lógica principal de UI con Relm4 (2500+ líneas)
└── core/
    ├── mod.rs           # Exports públicos del módulo
    ├── note_buffer.rs   # Buffer de texto con ropey + undo/redo
    ├── command.rs       # Parser de comandos vim y acciones
    ├── editor_mode.rs   # Modos: Normal, Insert, Command, Visual
    ├── note_file.rs     # Gestión de archivos .md y directorio de notas
    ├── markdown.rs      # Parser markdown con pulldown-cmark
    └── notes_config.rs  # Configuración (próximamente)
```

### Sistema de archivos

- **Directorio base**: `~/.local/share/notnative/notes/`
- **Formato**: Cada nota es un archivo `.md` independiente
- **Estructura**: Soporte básico para carpetas (mejoras pendientes)
- **Backup-friendly**: Los archivos son estándar markdown legible
- **Autoguardado**: Cada 5 segundos si hay cambios

### Stack tecnológico

- **Rust 2024 Edition** - Lenguaje base
- **GTK4** - Toolkit nativo (sin libadwaita)
- **Relm4 0.10** - Framework reactivo para GTK4
- **ropey 1.6** - Estructura de datos rope para edición de texto eficiente
- **pulldown-cmark 0.10** - Parser markdown robusto con offsets
- **notify 6** - Watcher para cambios de tema del sistema
- **serde + serde_json** - Serialización (para config futura)
- **dirs 5** - Manejo de directorios del sistema
- **anyhow + thiserror** - Error handling

## 🎨 Diseño

NotNative está diseñado para ser:

1. **Rápido**: Operaciones de edición en O(log n), sin bloqueos en la UI
2. **Nativo**: Integración completa con el escritorio (Wayland, portales, D-Bus)
3. **Minimalista**: Interfaz limpia, navegación solo con teclado
4. **Extensible**: Arquitectura modular preparada para plugins

## 🔧 Desarrollo

### Tests

```bash
cargo test
```

### Estructura del buffer

El `NoteBuffer` usa `ropey::Rope` internamente:
- Inserciones/eliminaciones: O(log n)
- Conversiones línea↔carácter: O(log n)
- Acceso a líneas: O(log n)
- Undo/redo con stack de operaciones (historial de 1000)

### Sistema de comandos

```rust
KeyPress → CommandParser → EditorAction → NoteBuffer → sync_to_view()
```

Flujo:
1. `EventControllerKey` captura teclas en `text_view`
2. `CommandParser` convierte tecla + modo en `EditorAction`
3. `MainApp::execute_action()` modifica el `NoteBuffer`
4. `sync_to_view()` actualiza GTK `TextBuffer`
5. En modo Normal: aplica estilos markdown y renderiza texto limpio
6. En modo Insert: muestra texto crudo con símbolos

### Renderizado Markdown

Modo dual de visualización:

- **Modo Normal**: Vista limpia
  - Los símbolos markdown se ocultan (`**`, `#`, `` ` ``, etc.)
  - Se aplican estilos GTK TextTags (negrita, cursiva, headings)
  - Links son clickeables con cursor pointer
  - Mapeo de posiciones buffer ↔ texto mostrado

- **Modo Insert**: Vista cruda
  - Todos los símbolos markdown visibles
  - Sin estilos aplicados (texto plano)
  - Permite editar el markdown directamente

### Integración con Tema del Sistema

NotNative se integra con el sistema de temas Omarchy:

1. **Carga inicial**: Lee CSS de `~/.config/omarchy/current/theme/*.css`
2. **Watcher**: Thread de `notify` detecta cambios en el symlink
3. **Recarga**: Aplica nuevo CSS y actualiza colores de TextTags
4. **Adaptación**: Los colores de links y código se extraen del tema

## � TODO - Próximos Pasos

### 🔥 Prioridad Alta (En Desarrollo Activo)

#### 1. Drag & Drop en Sidebar ⚡ NEXT
- [ ] Implementar `gtk::DragSource` en filas del ListBox
- [ ] Implementar `gtk::DropTarget` para recibir drops
- [ ] Detectar drop entre notas (reordenar)
- [ ] Detectar drop sobre carpetas (mover nota a carpeta)
- [ ] Detectar drop de carpeta sobre carpeta (anidar)
- [ ] Actualizar estructura de archivos en disco
- [ ] Animaciones visuales durante el drag
- [ ] Feedback visual (placeholder, highlight)
- [ ] Persistir nuevo orden en metadata

#### 2. Sistema de Indexación con SQLite ⚡ NEXT
- [ ] Crear esquema de base de datos:
  - Tabla `notes` (id, name, path, content, created_at, updated_at)
  - Tabla `tags` (id, name)
  - Tabla `note_tags` (note_id, tag_id)
  - Tabla virtual FTS5 para full-text search
- [ ] Implementar módulo `core/database.rs`
- [ ] Indexar notas existentes al iniciar
- [ ] Watcher para actualizar índice en cambios de archivos
- [ ] Re-indexar al guardar nota
- [ ] Migración y versionado de esquema

#### 3. Búsqueda Full-Text ⚡ NEXT
- [ ] Barra de búsqueda en header del sidebar
- [ ] Widget Entry con botón de búsqueda
- [ ] Query a SQLite FTS5
- [ ] Mostrar resultados en sidebar
- [ ] Resaltar coincidencias en resultados
- [ ] Búsqueda por:
  - Nombre de nota
  - Contenido
  - Tags
  - Fecha (creación/modificación)
- [ ] Filtrado en tiempo real (debounce)
- [ ] Mostrar snippets de contexto

#### 4. Sistema de Tags con Autocompletado ⚡ NEXT
- [ ] Parsear frontmatter YAML en notas:
  ```yaml
  ---
  tags: [tag1, tag2, tag3]
  ---
  ```
- [ ] Almacenar tags en base de datos
- [ ] Widget de entrada de tags en header/footer
- [ ] Autocompletado con `gtk::EntryCompletion`
- [ ] Sugerencias basadas en tags existentes
- [ ] Vista de tags más usados
- [ ] Filtrar sidebar por tag
- [ ] Color coding para tags (opcional)

#### 5. Completar Menú Contextual
- [ ] Implementar renombrado de notas (ya hay estructura, falta lógica)
- [ ] Implementar eliminación de notas (base implementada, refinar)
- [ ] Añadir confirmación de eliminación (dialog)
- [ ] Actualizar sidebar después de renombrar/eliminar
- [ ] Manejar carpetas en el menú contextual
- [ ] Crear nueva carpeta desde menú

#### 6. Mejorar Renderizado Markdown
- [ ] Syntax highlighting en bloques de código (usar `syntect` o similar)
- [ ] Soporte para imágenes inline
- [ ] Tablas markdown
- [ ] Listas anidadas y numeradas
- [ ] Checkboxes (`- [ ]` / `- [x]`)
- [ ] Mejorar colores de links según tema actual

### ⚡ Prioridad Media (UX y Pulido)

#### 5. Vista Previa Markdown Opcional
- [ ] Panel lateral con vista previa renderizada
- [ ] Toggle para mostrar/ocultar preview
- [ ] Scroll sincronizado entre editor y preview
- [ ] Usar WebKit o widget nativo para renderizado

#### 6. Atajos de Teclado Adicionales
- [ ] `Ctrl+N` - Nueva nota (alternativa al diálogo)
- [ ] `Ctrl+F` - Buscar en nota actual
- [ ] `Ctrl+Shift+F` - Buscar en todas las notas
- [ ] `/` en modo Normal - Quick search
- [ ] `:e <nombre>` - Abrir nota por nombre

#### 7. Configuración y Preferencias
- [ ] Diálogo de preferencias funcional (actualmente solo placeholder)
- [ ] Configurar directorio de notas
- [ ] Configurar intervalo de autoguardado
- [ ] Elegir tema (light/dark/system)
- [ ] Configurar fuente y tamaño
- [ ] Habilitar/deshabilitar markdown rendering

#### 8. Ventana "Acerca de"
- [ ] Diálogo con información del proyecto
- [ ] Versión actual
- [ ] Licencia (MIT)
- [ ] Créditos y enlaces

### 🎨 Prioridad Baja (Nice-to-Have)

#### 9. Modo 8BIT (Completar o Remover)
- [ ] Re-habilitar botón 8BIT (actualmente comentado)
- [ ] O eliminar completamente si no es necesario
- [ ] Considerar como Easter egg o feature opcional

#### 10. Exportación
- [ ] Exportar nota actual a HTML
- [ ] Exportar nota actual a PDF
- [ ] Exportar todas las notas (zip)

### 🚀 Futuro (v0.2+)

#### 11. Integración Hyprland
- [ ] Layer-shell para modo overlay
- [ ] IPC con Hyprland
- [ ] Shortcuts globales del compositor
- [ ] Modo "quick note" flotante

#### 12. API de IA (OpenRouter)
- [ ] Integración con OpenRouter API
- [ ] Resúmenes automáticos de notas largas
- [ ] Chat con contexto de notas
- [ ] Sugerencias y autocompletado inteligente

#### 13. MCP (Model Context Protocol)
- [ ] Server MCP para exponer notas
- [ ] Integración con herramientas MCP
- [ ] Extensiones vía MCP

#### 14. Sincronización (Opcional)
- [ ] Git sync básico
- [ ] Sync con servicios cloud (Nextcloud, Syncthing)
- [ ] Detección y resolución de conflictos

---

## 📝 Roadmap General

- [x] **v0.1** - Editor funcional con markdown, sidebar y carpetas ✅
- [ ] **v0.2** - Drag & drop, indexación SQLite, búsqueda, tags 🔥 **EN DESARROLLO**
- [ ] **v0.3** - Vista previa, exportación, preferencias
- [ ] **v0.4** - Integración Hyprland, shortcuts globales
- [ ] **v0.5** - API de IA (OpenRouter)
- [ ] **v0.6** - MCP integration
- [ ] **v0.7** - Sincronización cloud
- [ ] **v1.0** - Estabilización y release

## � Issues Conocidos y Mejoras Pendientes

### Bugs
- [ ] Renombrado de notas no implementado (estructura lista, falta lógica)
- [ ] Menú contextual: parent/unparent puede causar warnings en GTK
- [ ] Carpetas anidadas no se visualizan correctamente en sidebar
- [ ] Eliminar carpeta no está implementado

### Mejoras de Performance
- [ ] Renderizado markdown en thread separado para notas muy largas
- [ ] Lazy loading del sidebar (cargar solo notas visibles)
- [ ] Debounce en hover del sidebar (evitar cargas excesivas)

### UX/UI
- [ ] Animación de sidebar mejorable (considerar libadwaita AnimatedPane)
- [ ] Indicador visual cuando se guarda automáticamente
- [ ] Feedback visual al crear/eliminar notas
- [ ] Atajos de teclado no aparecen en diálogo (placeholder vacío)

### Refactoring
- [ ] `app.rs` es muy grande (2500+ líneas) - dividir en módulos
- [ ] Separar lógica de sidebar a componente Relm4 independiente
- [ ] Extraer renderizado markdown a módulo separado
- [ ] Mejorar manejo de errores (más mensajes informativos al usuario)

---

## �📜 Licencia

MIT

## 🤝 Contribuir

Las contribuciones son bienvenidas. Por favor, abre un issue primero para discutir cambios mayores.

---

## 📊 Estado del Proyecto

**Versión actual**: v0.1.0  
**Última actualización**: Noviembre 2025  
**Estado**: Alpha - Funcional pero en desarrollo activo  
**Líneas de código**: ~4000 líneas Rust  
**Tests**: Pendiente de implementar
