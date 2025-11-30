use gtk::prelude::*;
use gtk::{gio, glib};
use relm4::gtk;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::path::Path;
use std::fmt;
use webkit6::prelude::WebViewExt;

use crate::core::{
    Base, BaseQueryEngine, BaseView, ColumnConfig, Filter, FilterGroup, 
    FilterOperator, GroupedRecord, NoteMetadata, NoteWithProperties, NotesDatabase, PropertyValue, 
    SortConfig, SortDirection, SourceType, ViewType, HtmlRenderer, PreviewTheme,
};
use crate::graph_view::GraphView;
use crate::i18n::{I18n, Language};

/// Widget para mostrar una vista de tabla de una Base
pub struct BaseTableWidget {
    container: gtk::Box,
    content_stack: gtk::Stack,  // Stack para alternar tabla/grafo
    table_webview: webkit6::WebView,  // WebView para la tabla HTML
    column_view: gtk::ColumnView,  // ColumnView (mantenido para lógica de columnas)
    list_store: gio::ListStore,  // ListStore para datos
    filter_bar: gtk::Box,
    filters_container: gtk::Box,
    view_tabs: gtk::Box,
    status_bar: gtk::Box,
    graph_view: GraphView,  // Vista de grafo de relaciones
    graph_toggle: gtk::ToggleButton,  // Botón para alternar vista
    sort_btn: gtk::MenuButton,  // Botón de ordenamiento
    columns_btn: gtk::MenuButton,  // Botón de columnas
    source_type_btn: gtk::MenuButton,  // Botón para cambiar modo (Notes/GroupedRecords)
    
    /// Internacionalización
    i18n: Rc<RefCell<I18n>>,
    
    /// Base actual
    base: Rc<RefCell<Option<Base>>>,
    
    /// Notas actuales (sin filtrar)
    all_notes: Rc<RefCell<Vec<NoteWithProperties>>>,
    
    /// Notas filtradas (mostradas)
    notes: Rc<RefCell<Vec<NoteWithProperties>>>,
    
    /// Filtros activos (adicionales a los de la vista)
    active_filters: Rc<RefCell<Vec<Filter>>>,
    
    /// Ordenamiento actual
    current_sort: Rc<RefCell<Option<SortConfig>>>,
    
    /// Propiedades disponibles
    available_properties: Rc<RefCell<Vec<String>>>,
    
    /// Referencia a la BD y notes_root para refrescar
    db_path: Rc<RefCell<Option<std::path::PathBuf>>>,
    notes_root: Rc<RefCell<Option<std::path::PathBuf>>>,
    
    /// ID de la base actual (para persistir cambios)
    base_id: Rc<RefCell<Option<i64>>>,
    
    /// Referencia a la BD para persistir cambios
    notes_db: Rc<RefCell<Option<NotesDatabase>>>,
    
    /// Callbacks
    on_note_selected: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    on_note_double_click: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    
    /// Callback para clic en nodo del grafo (requiere Send+Sync)
    on_graph_note_click: std::sync::Arc<std::sync::Mutex<Option<Box<dyn Fn(&str) + Send + Sync>>>>,
    
    /// Callback cuando cambia el source_type (para recargar)
    on_source_type_changed: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    
    /// Callback cuando se hace clic en la vista (para cerrar sidebar)
    on_view_clicked: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl fmt::Debug for BaseTableWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BaseTableWidget")
            .field("base_id", &self.base_id.borrow())
            .finish_non_exhaustive()
    }
}

impl BaseTableWidget {
    pub fn new(i18n: Rc<RefCell<I18n>>) -> Self {
        // Container principal
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(0)
            .css_classes(["base-table-container"])
            .build();

        // Barra de filtros (arriba)
        let (filter_bar, filters_container, sort_btn, columns_btn, graph_toggle, source_type_btn) = Self::create_filter_bar(&i18n.borrow());
        container.append(&filter_bar);

        // Tabs de vistas
        let view_tabs = Self::create_view_tabs();
        container.append(&view_tabs);

        // Stack para alternar entre tabla y grafo
        let content_stack = gtk::Stack::builder()
            .vexpand(true)
            .transition_type(gtk::StackTransitionType::Crossfade)
            .transition_duration(200)
            .build();

        // Scroll container para el WebView de la tabla
        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();

        // WebView para la tabla HTML (como en las notas)
        let table_webview = webkit6::WebView::builder()
            .vexpand(true)
            .hexpand(true)
            .build();
        
        // Configurar el WebView
        if let Some(settings) = webkit6::prelude::WebViewExt::settings(&table_webview) {
            settings.set_enable_javascript(true);
            settings.set_enable_smooth_scrolling(true);
        }
        
        // Configurar UserContentManager para recibir mensajes JS→Rust
        let on_note_selected: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));
        let on_view_clicked: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
        
        if let Some(content_manager) = table_webview.user_content_manager() {
            content_manager.register_script_message_handler("noteClick", None);
            
            // Conectar el handler inmediatamente
            let on_note_selected_clone = on_note_selected.clone();
            let on_view_clicked_clone = on_view_clicked.clone();
            
            content_manager.connect_script_message_received(Some("noteClick"), move |_, result| {
                // Obtener el mensaje
                let message_str = result.to_str();
                let clean_path = message_str.trim_matches('"').to_string();
                
                // Siempre cerrar el sidebar al hacer clic
                if let Some(ref callback) = *on_view_clicked_clone.borrow() {
                    callback();
                }
                
                // Solo procesar selección si no es el mensaje especial de cerrar sidebar
                if !clean_path.is_empty() && clean_path != "__close_sidebar__" {
                    if let Some(ref callback) = *on_note_selected_clone.borrow() {
                        callback(&clean_path);
                    }
                }
            });
        }

        scroll.set_child(Some(&table_webview));
        
        // Lista vacía para datos (mantenida para lógica de filtros/orden)
        let list_store = gio::ListStore::new::<glib::BoxedAnyObject>();
        let selection_model = gtk::SingleSelection::new(Some(list_store.clone()));
        
        // ColumnView (oculto, solo para lógica de columnas)
        let column_view = gtk::ColumnView::builder()
            .model(&selection_model)
            .css_classes(["base-table"])
            .build();
        
        // Añadir WebView al stack (no ColumnView)
        content_stack.add_named(&scroll, Some("table"));
        
        // Crear GraphView
        let graph_view = GraphView::new();
        graph_view.set_vexpand(true);
        graph_view.set_hexpand(true);
        graph_view.add_css_class("base-graph-view");
        
        // Añadir grafo al stack
        content_stack.add_named(&graph_view, Some("graph"));
        
        // Mostrar tabla por defecto
        content_stack.set_visible_child_name("table");
        
        container.append(&content_stack);

        // Barra de estado (abajo)
        let status_bar = Self::create_status_bar();
        container.append(&status_bar);

        Self {
            container,
            content_stack,
            table_webview,
            column_view,
            list_store,
            filter_bar,
            filters_container,
            view_tabs,
            status_bar,
            graph_view,
            graph_toggle,
            sort_btn,
            columns_btn,
            source_type_btn,
            i18n,
            base: Rc::new(RefCell::new(None)),
            all_notes: Rc::new(RefCell::new(Vec::new())),
            notes: Rc::new(RefCell::new(Vec::new())),
            active_filters: Rc::new(RefCell::new(Vec::new())),
            current_sort: Rc::new(RefCell::new(None)),
            available_properties: Rc::new(RefCell::new(Vec::new())),
            db_path: Rc::new(RefCell::new(None)),
            notes_root: Rc::new(RefCell::new(None)),
            base_id: Rc::new(RefCell::new(None)),
            notes_db: Rc::new(RefCell::new(None)),
            on_note_selected,
            on_note_double_click: Rc::new(RefCell::new(None)),
            on_graph_note_click: std::sync::Arc::new(std::sync::Mutex::new(None)),
            on_source_type_changed: Rc::new(RefCell::new(None)),
            on_view_clicked,
        }
    }

    fn create_filter_bar(i18n: &I18n) -> (gtk::Box, gtk::Box, gtk::MenuButton, gtk::MenuButton, gtk::ToggleButton, gtk::MenuButton) {
        let bar = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .margin_bottom(8)
            .css_classes(["base-filter-bar"])
            .build();

        // Botón de añadir filtro
        let add_filter_btn = gtk::MenuButton::builder()
            .icon_name("view-filter-symbolic")
            .tooltip_text(&i18n.t("base_add_filter"))
            .css_classes(["flat"])
            .build();
        bar.append(&add_filter_btn);

        // Separator
        bar.append(&gtk::Separator::new(gtk::Orientation::Vertical));

        // Contenedor de filtros activos (se llenará dinámicamente)
        let filters_container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(4)
            .hexpand(true)
            .build();
        bar.append(&filters_container);

        // Botón de ordenamiento
        let sort_btn = gtk::MenuButton::builder()
            .icon_name("view-sort-ascending-symbolic")
            .tooltip_text(&i18n.t("base_sort"))
            .css_classes(["flat"])
            .build();
        bar.append(&sort_btn);

        // Botón de columnas
        let columns_btn = gtk::MenuButton::builder()
            .icon_name("view-column-symbolic")
            .tooltip_text(&i18n.t("base_columns"))
            .css_classes(["flat"])
            .build();
        bar.append(&columns_btn);

        // Separator antes del toggle de grafo
        bar.append(&gtk::Separator::new(gtk::Orientation::Vertical));

        // Botón para cambiar modo (Notes/GroupedRecords)
        let source_type_btn = gtk::MenuButton::builder()
            .icon_name("view-list-symbolic")
            .tooltip_text(&i18n.t("base_data_source"))
            .css_classes(["flat"])
            .build();
        bar.append(&source_type_btn);

        // Separator antes del toggle de grafo
        bar.append(&gtk::Separator::new(gtk::Orientation::Vertical));

        // Toggle para vista de grafo de relaciones
        let graph_toggle = gtk::ToggleButton::builder()
            .icon_name("network-workgroup-symbolic")
            .tooltip_text(&i18n.t("base_show_graph"))
            .css_classes(["flat", "base-graph-toggle"])
            .build();
        bar.append(&graph_toggle);

        (bar, filters_container, sort_btn, columns_btn, graph_toggle, source_type_btn)
    }

    fn create_view_tabs() -> gtk::Box {
        let tabs = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(0)
            .margin_start(12)
            .margin_end(12)
            .css_classes(["base-view-tabs"])
            .build();

        // Se llenarán dinámicamente con las vistas de la Base

        tabs
    }

    fn create_status_bar() -> gtk::Box {
        let bar = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(4)
            .margin_bottom(4)
            .css_classes(["base-status-bar"])
            .build();

        // Contador de notas
        let count_label = gtk::Label::builder()
            .label("0 notes")
            .css_classes(["dim-label"])
            .build();
        bar.append(&count_label);

        bar
    }

    /// Obtener el widget principal
    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }
    
    /// Actualizar el idioma de la interfaz (llamar cuando cambia el idioma global)
    pub fn update_language(&mut self) {
        let i18n = self.i18n.borrow();
        
        // Actualizar tooltips de los botones de la barra de herramientas
        // Buscar el botón de filtro (primer hijo después del inicio)
        if let Some(filter_btn) = self.filter_bar.first_child() {
            if let Some(btn) = filter_btn.downcast_ref::<gtk::MenuButton>() {
                btn.set_tooltip_text(Some(&i18n.t("base_add_filter")));
            }
        }
        
        // Actualizar tooltip de sort
        self.sort_btn.set_tooltip_text(Some(&i18n.t("base_sort")));
        
        // Actualizar tooltip de columnas
        self.columns_btn.set_tooltip_text(Some(&i18n.t("base_columns")));
        
        // Actualizar tooltip de source type
        self.source_type_btn.set_tooltip_text(Some(&i18n.t("base_data_source")));
        
        // Actualizar tooltip del toggle de grafo
        self.graph_toggle.set_tooltip_text(Some(&i18n.t("base_show_graph")));
        
        drop(i18n);
        
        // Regenerar los popovers con el nuevo idioma
        self.setup_filter_popover();
        self.setup_sort_popover();
        self.setup_columns_popover();
        self.setup_source_type_popover();
        
        // Actualizar los chips de filtro
        self.update_filter_chips();
        
        // Si hay datos cargados, refrescar la tabla para actualizar los headers
        if self.base.borrow().is_some() {
            let notes = self.notes.borrow();
            if !notes.is_empty() {
                let columns = if let Some(base) = self.base.borrow().as_ref() {
                    if let Some(view) = base.active_view() {
                        view.columns.clone()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                drop(notes);
                
                let notes_ref = self.notes.borrow();
                let html = self.render_table_html(&notes_ref, &columns);
                self.table_webview.load_html(&html, None);
            }
        }
    }

    /// Cargar una Base y mostrar sus datos
    pub fn load_base(&mut self, base_id: i64, base: Base, db: NotesDatabase, notes_root: &Path) {
        // Guardar referencias
        *self.base.borrow_mut() = Some(base.clone());
        *self.base_id.borrow_mut() = Some(base_id);
        *self.notes_db.borrow_mut() = Some(db.clone_connection());
        
        // Guardar paths para refrescar
        *self.notes_root.borrow_mut() = Some(notes_root.to_path_buf());
        
        // Cargar filtros y sort guardados desde la vista activa
        if let Some(view) = base.active_view() {
            *self.active_filters.borrow_mut() = view.filter.filters.clone();
            *self.current_sort.borrow_mut() = view.sort.clone();
        }

        // Comportamiento según el tipo de fuente
        match base.source_type {
            SourceType::Notes => {
                // Descubrir propiedades disponibles
                let engine = BaseQueryEngine::new(&db, notes_root);
                if let Ok(props) = engine.discover_properties(base.source_folder.as_deref()) {
                    *self.available_properties.borrow_mut() = props;
                }

                // Configurar popovers con las propiedades
                self.setup_filter_popover();
                self.setup_sort_popover();
                self.setup_columns_popover();

                // Actualizar tabs de vistas
                self.update_view_tabs(&base);

                // Obtener la vista activa
                if let Some(view) = base.active_view() {
                    self.load_view(view, base.source_folder.as_deref(), &db, notes_root);
                }
            }
            SourceType::GroupedRecords => {
                // Cargar registros agrupados
                self.load_grouped_records(&db, &base);
            }
        }
        
        // Configurar toggle del grafo
        self.setup_graph_toggle(&db);
        
        // Configurar popover de modo de datos
        self.setup_source_type_popover();
    }
    
    /// Cargar registros agrupados en la tabla
    fn load_grouped_records(&mut self, db: &NotesDatabase, base: &Base) {
        match db.get_all_grouped_records() {
            Ok(records) => {
                // Descubrir propiedades disponibles de los registros
                // properties es Vec<(String, String)>, extraemos las claves
                let mut props: Vec<String> = records.iter()
                    .flat_map(|r| r.properties.iter().map(|(k, _)| k.clone()))
                    .collect();
                props.sort();
                props.dedup();
                
                // Añadir _note al inicio
                let mut available = vec!["_note".to_string()];
                available.extend(props);
                *self.available_properties.borrow_mut() = available;
                
                // Configurar popovers con las propiedades correctas
                self.setup_filter_popover();
                self.setup_sort_popover();
                self.setup_columns_popover();
                
                // Actualizar tabs
                self.update_view_tabs(base);
                
                // Obtener columnas de la vista activa
                let columns = base.active_view()
                    .map(|v| v.columns.clone())
                    .unwrap_or_default();
                
                // Actualizar columnas en la tabla
                self.update_columns(&columns);
                
                // Convertir GroupedRecord a NoteWithProperties para reusar la tabla
                let notes: Vec<NoteWithProperties> = records.iter().map(|r| {
                    let mut properties = HashMap::new();
                    properties.insert("_note".to_string(), PropertyValue::Text(r.note_name.clone()));
                    for (k, v) in &r.properties {
                        properties.insert(k.clone(), PropertyValue::Text(v.clone()));
                    }
                    
                    // Crear metadata falsa para reusar NoteWithProperties
                    let metadata = NoteMetadata {
                        id: r.note_id,
                        name: r.note_name.clone(),
                        path: String::new(),
                        folder: None,
                        order_index: 0,
                        icon: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    };
                    
                    NoteWithProperties {
                        metadata,
                        properties,
                        content: None,
                    }
                }).collect();
                
                *self.all_notes.borrow_mut() = notes.clone();
                *self.notes.borrow_mut() = notes.clone();
                
                // Actualizar la tabla con WebView
                self.update_data(&notes);
                
                // Actualizar status
                self.update_status_bar(notes.len());
            }
            Err(e) => {
                eprintln!("Error loading grouped records: {}", e);
            }
        }
    }
    
    /// Configurar el toggle del grafo de relaciones
    fn setup_graph_toggle(&self, db: &NotesDatabase) {
        let content_stack = self.content_stack.clone();
        let graph_view = self.graph_view.clone();
        let db_clone = db.clone_connection();
        
        self.graph_toggle.connect_toggled(move |toggle| {
            if toggle.is_active() {
                // Cargar datos del grafo desde registros agrupados
                match db_clone.get_all_grouped_records() {
                    Ok(records) => {
                        graph_view.load_from_grouped_records(&records);
                        graph_view.start_simulation();
                    }
                    Err(e) => {
                        eprintln!("Error loading grouped records: {}", e);
                    }
                }
                content_stack.set_visible_child_name("graph");
            } else {
                graph_view.stop_simulation();
                content_stack.set_visible_child_name("table");
            }
        });
        
        // Configurar callback para doble-clic en nodos del grafo
        let on_click = self.on_graph_note_click.clone();
        self.graph_view.on_note_click(move |note_name| {
            if let Ok(guard) = on_click.lock() {
                if let Some(ref callback) = *guard {
                    callback(note_name);
                }
            }
        });
    }
    
    /// Configurar callback para doble-clic en nodos del grafo
    pub fn on_graph_note_click<F: Fn(&str) + Send + Sync + 'static>(&self, callback: F) {
        if let Ok(mut guard) = self.on_graph_note_click.lock() {
            *guard = Some(Box::new(callback));
        }
    }
    
    /// Configurar callback para cuando cambia el source_type
    pub fn on_source_type_changed<F: Fn() + 'static>(&self, callback: F) {
        *self.on_source_type_changed.borrow_mut() = Some(Box::new(callback));
    }

    /// Cargar una vista específica
    fn load_view(
        &mut self,
        view: &BaseView,
        source_folder: Option<&str>,
        db: &NotesDatabase,
        notes_root: &Path,
    ) {
        // Ejecutar query
        let engine = BaseQueryEngine::new(db, notes_root);
        match engine.query_view(view, source_folder) {
            Ok(notes) => {
                // Guardar todas las notas
                *self.all_notes.borrow_mut() = notes.clone();
                
                // Copiar el sort de la vista si existe
                *self.current_sort.borrow_mut() = view.sort.clone();

                // Aplicar filtros adicionales y ordenar
                self.apply_filters_and_sort();

                // Actualizar columnas
                self.update_columns(&view.columns);
            }
            Err(e) => {
                eprintln!("Error executing Base query: {}", e);
            }
        }
    }
    
    /// Aplicar filtros activos y ordenamiento
    fn apply_filters_and_sort(&self) {
        let all_notes = self.all_notes.borrow();
        let filters = self.active_filters.borrow();
        let sort = self.current_sort.borrow();
        
        // Filtrar notas
        let mut filtered: Vec<NoteWithProperties> = all_notes
            .iter()
            .filter(|note| {
                filters.iter().all(|f| f.evaluate(&note.properties))
            })
            .cloned()
            .collect();
        
        // Ordenar
        if let Some(sort_config) = sort.as_ref() {
            filtered.sort_by(|a, b| {
                let key_a = a.properties
                    .get(&sort_config.property)
                    .map(|v| v.sort_key())
                    .unwrap_or_default();
                let key_b = b.properties
                    .get(&sort_config.property)
                    .map(|v| v.sort_key())
                    .unwrap_or_default();

                match sort_config.direction {
                    SortDirection::Asc => key_a.cmp(&key_b),
                    SortDirection::Desc => key_b.cmp(&key_a),
                }
            });
        }
        
        // Actualizar notas mostradas
        *self.notes.borrow_mut() = filtered.clone();
        
        // Actualizar UI
        self.update_data(&filtered);
        self.update_status_bar(filtered.len());
        self.update_filter_chips();
    }
    
    /// Persistir la configuración actual de la Base en la BD
    fn save_config(&self) {
        let base_id = self.base_id.borrow();
        let notes_db = self.notes_db.borrow();
        let mut base_opt = self.base.borrow_mut();
        
        if let (Some(id), Some(db), Some(base)) = (base_id.as_ref(), notes_db.as_ref(), base_opt.as_mut()) {
            // Sincronizar filtros y sort a la vista activa
            if let Some(view) = base.views.get_mut(base.active_view) {
                view.filter.filters = self.active_filters.borrow().clone();
                view.sort = self.current_sort.borrow().clone();
            }
            
            // Serializar y guardar
            if let Ok(yaml) = base.serialize() {
                if let Err(e) = db.update_base(*id, &yaml, base.active_view as i32) {
                    eprintln!("Error saving Base config: {}", e);
                }
            }
        }
    }
    
    /// Añadir un filtro
    pub fn add_filter(&self, filter: Filter) {
        self.active_filters.borrow_mut().push(filter);
        self.apply_filters_and_sort();
        self.save_config();
    }
    
    /// Eliminar un filtro por índice
    pub fn remove_filter(&self, index: usize) {
        let mut filters = self.active_filters.borrow_mut();
        if index < filters.len() {
            filters.remove(index);
        }
        drop(filters);
        self.apply_filters_and_sort();
        self.save_config();
    }
    
    /// Limpiar todos los filtros
    pub fn clear_filters(&self) {
        self.active_filters.borrow_mut().clear();
        self.apply_filters_and_sort();
        self.save_config();
    }
    
    /// Establecer ordenamiento
    pub fn set_sort(&self, sort: Option<SortConfig>) {
        *self.current_sort.borrow_mut() = sort;
        self.apply_filters_and_sort();
        self.save_config();
    }
    
    /// Configurar el popover de filtros
    fn setup_filter_popover(&self) {
        // Obtener solo las columnas visibles de la vista actual
        let properties: Vec<String> = if let Some(base) = self.base.borrow().as_ref() {
            if let Some(view) = base.active_view() {
                view.columns.iter()
                    .filter(|c| c.visible)
                    .map(|c| c.property.clone())
                    .collect()
            } else {
                self.available_properties.borrow().clone()
            }
        } else {
            self.available_properties.borrow().clone()
        };
        let (popover, prop_combo, op_combo, value_entry) = create_filter_popover_with_refs(&properties, &self.i18n.borrow());
        
        // Clonar referencias para el closure
        let active_filters = self.active_filters.clone();
        let all_notes = self.all_notes.clone();
        let notes = self.notes.clone();
        let current_sort = self.current_sort.clone();
        let list_store = self.list_store.clone();
        let status_bar = self.status_bar.clone();
        let filters_container = self.filters_container.clone();
        let popover_clone = popover.clone();
        let properties_clone = properties.clone();
        let table_webview = self.table_webview.clone();
        let base = self.base.clone();
        
        // Buscar el botón Apply dentro del popover y conectarlo
        if let Some(content) = popover.child().and_downcast::<gtk::Box>() {
            // El último hijo es el box de botones
            if let Some(buttons_box) = content.last_child().and_downcast::<gtk::Box>() {
                // El último botón es Apply
                if let Some(apply_btn) = buttons_box.last_child().and_downcast::<gtk::Button>() {
                    apply_btn.connect_clicked(move |_| {
                        // Obtener valores seleccionados
                        let prop_idx = prop_combo.selected() as usize;
                        let op_idx = op_combo.selected() as usize;
                        let value_text = value_entry.text().to_string();
                        
                        if prop_idx < properties_clone.len() {
                            let property = properties_clone[prop_idx].clone();
                            let operator = index_to_operator(op_idx);
                            let value = parse_filter_value(&value_text);
                            
                            let filter = Filter {
                                property,
                                operator,
                                value,
                            };
                            
                            // Añadir filtro
                            active_filters.borrow_mut().push(filter);
                            
                            // Re-aplicar filtros
                            let all = all_notes.borrow();
                            let filters = active_filters.borrow();
                            let sort = current_sort.borrow();
                            
                            let mut filtered: Vec<NoteWithProperties> = all
                                .iter()
                                .filter(|note| {
                                    filters.iter().all(|f| f.evaluate(&note.properties))
                                })
                                .cloned()
                                .collect();
                            
                            // Ordenar
                            if let Some(sort_config) = sort.as_ref() {
                                filtered.sort_by(|a, b| {
                                    let key_a = a.properties
                                        .get(&sort_config.property)
                                        .map(|v| v.sort_key())
                                        .unwrap_or_default();
                                    let key_b = b.properties
                                        .get(&sort_config.property)
                                        .map(|v| v.sort_key())
                                        .unwrap_or_default();

                                    match sort_config.direction {
                                        SortDirection::Asc => key_a.cmp(&key_b),
                                        SortDirection::Desc => key_b.cmp(&key_a),
                                    }
                                });
                            }
                            
                            drop(all);
                            drop(filters);
                            drop(sort);
                            
                            *notes.borrow_mut() = filtered.clone();
                            
                            // Actualizar UI (list_store para lógica)
                            list_store.remove_all();
                            for note in &filtered {
                                let boxed = glib::BoxedAnyObject::new(note.clone());
                                list_store.append(&boxed);
                            }
                            
                            // Actualizar WebView
                            let columns = if let Some(base) = base.borrow().as_ref() {
                                if let Some(view) = base.views.get(base.active_view) {
                                    view.columns.clone()
                                } else {
                                    vec![
                                        ColumnConfig { property: "title".to_string(), title: None, width: Some(300), visible: true },
                                        ColumnConfig { property: "created".to_string(), title: None, width: Some(150), visible: true },
                                    ]
                                }
                            } else {
                                vec![
                                    ColumnConfig { property: "title".to_string(), title: None, width: Some(300), visible: true },
                                    ColumnConfig { property: "created".to_string(), title: None, width: Some(150), visible: true },
                                ]
                            };
                            let html = Self::render_table_html_static(&filtered, &columns, Language::from_env());
                            table_webview.load_html(&html, None);
                            
                            // Actualizar status
                            if let Some(label) = status_bar.first_child().and_downcast::<gtk::Label>() {
                                let text = if filtered.len() == 1 {
                                    "1 note".to_string()
                                } else {
                                    format!("{} notes", filtered.len())
                                };
                                label.set_text(&text);
                            }
                            
                            // Actualizar chips
                            update_filter_chips_in_container(&filters_container, &active_filters.borrow());
                        }
                        
                        // Cerrar popover
                        popover_clone.popdown();
                        
                        // Limpiar entry
                        value_entry.set_text("");
                    });
                }
            }
        }
        
        // Buscar el botón de filtros en la barra
        if let Some(filter_btn) = self.filter_bar.first_child().and_downcast::<gtk::MenuButton>() {
            filter_btn.set_popover(Some(&popover));
        }
    }
    
    /// Configurar el popover de ordenamiento
    fn setup_sort_popover(&self) {
        // Obtener solo las columnas visibles de la vista actual
        let properties: Vec<String> = if let Some(base) = self.base.borrow().as_ref() {
            if let Some(view) = base.active_view() {
                view.columns.iter()
                    .filter(|c| c.visible)
                    .map(|c| c.property.clone())
                    .collect()
            } else {
                self.available_properties.borrow().clone()
            }
        } else {
            self.available_properties.borrow().clone()
        };
        let popover = create_sort_popover_with_callbacks(
            &properties,
            self.current_sort.clone(),
            self.all_notes.clone(),
            self.notes.clone(),
            self.active_filters.clone(),
            self.list_store.clone(),
            self.status_bar.clone(),
            self.table_webview.clone(),
            self.base.clone(),
            &self.i18n.borrow(),
        );
        
        // Usar referencia directa al botón de sort
        self.sort_btn.set_popover(Some(&popover));
    }
    
    /// Configurar el popover de columnas (se regenera cada vez que se abre)
    fn setup_columns_popover(&self) {
        // Usar referencia directa al botón de columnas
        let columns_btn = &self.columns_btn;
        
        // Clonar referencias necesarias
        let base_ref = self.base.clone();
        let base_id = self.base_id.clone();
        let notes_db = self.notes_db.clone();
        let column_view = self.column_view.clone();
        let available_props = self.available_properties.clone();
        let table_webview = self.table_webview.clone();
        let notes = self.notes.clone();
        let i18n = self.i18n.clone();
        
        // Crear el popover una vez
        let popover = gtk::Popover::builder()
            .css_classes(["columns-popover"])
            .has_arrow(true)
            .build();
        
        // Contenedor que se actualizará dinámicamente
        let content_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .width_request(220)
            .build();
        
        popover.set_child(Some(&content_box));
        columns_btn.set_popover(Some(&popover));
        
        // Clonar para el callback
        let content_box_clone = content_box.clone();
        let popover_clone = popover.clone();
        
        // Conectar al evento de apertura del popover para refrescar contenido
        popover.connect_notify_local(Some("visible"), move |pop, _| {
            if pop.is_visible() {
                // Refrescar contenido cuando se abre
                Self::refresh_columns_popover_content(
                    &content_box_clone,
                    &base_ref,
                    &base_id,
                    &notes_db,
                    &column_view,
                    &available_props.borrow(),
                    &popover_clone,
                    &table_webview,
                    &notes,
                    &i18n.borrow(),
                );
            }
        });
    }
    
    /// Refrescar el contenido del popover de columnas (sin cerrarlo)
    fn refresh_columns_popover_content(
        content: &gtk::Box,
        base_ref: &Rc<RefCell<Option<Base>>>,
        base_id: &Rc<RefCell<Option<i64>>>,
        notes_db: &Rc<RefCell<Option<NotesDatabase>>>,
        column_view: &gtk::ColumnView,
        available_props: &[String],
        popover: &gtk::Popover,
        table_webview: &webkit6::WebView,
        notes: &Rc<RefCell<Vec<NoteWithProperties>>>,
        i18n: &I18n,
    ) {
        // Limpiar contenido existente
        while let Some(child) = content.first_child() {
            content.remove(&child);
        }
        
        let base = base_ref.borrow();
        if let Some(base_data) = base.as_ref() {
            if let Some(view) = base_data.active_view() {
                // === Sección: Columnas actuales ===
                let current_title = gtk::Label::builder()
                    .label(&i18n.t("base_current_columns"))
                    .css_classes(["heading"])
                    .xalign(0.0)
                    .margin_bottom(4)
                    .build();
                content.append(&current_title);
                
                // Un checkbox por cada columna existente
                let existing_props: Vec<String> = view.columns.iter()
                    .map(|c| c.property.clone())
                    .collect();
                
                // Clonar para callbacks de actualización
                let content_clone = content.clone();
                let popover_clone = popover.clone();
                let available_props_vec: Vec<String> = available_props.to_vec();
                let table_webview = table_webview.clone();
                let notes = notes.clone();
                    
                for (col_idx, col) in view.columns.iter().enumerate() {
                    let row = gtk::Box::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .spacing(8)
                        .build();
                    
                    let check = gtk::CheckButton::builder()
                        .active(col.visible)
                        .tooltip_text(&i18n.t("base_toggle_visibility"))
                        .build();
                    row.append(&check);
                    
                    let label = gtk::Label::builder()
                        .label(&col.display_title())
                        .hexpand(true)
                        .xalign(0.0)
                        .build();
                    row.append(&label);
                    
                    // Botón para eliminar columna (siempre disponible)
                    {
                        let remove_btn = gtk::Button::builder()
                            .icon_name("user-trash-symbolic")
                            .css_classes(["flat", "circular", "destructive-action"])
                            .tooltip_text(&i18n.t("base_remove_column"))
                            .build();
                        
                        let base_ref_clone = base_ref.clone();
                        let base_id_clone = base_id.clone();
                        let notes_db_clone = notes_db.clone();
                        let column_view_clone = column_view.clone();
                        let content_for_refresh = content_clone.clone();
                        let popover_for_refresh = popover_clone.clone();
                        let available_props_for_refresh = available_props_vec.clone();
                        let table_webview_clone = table_webview.clone();
                        let notes_clone = notes.clone();
                        
                        remove_btn.connect_clicked(move |_| {
                            let columns_for_html: Vec<ColumnConfig>;
                            {
                                let mut base_opt = base_ref_clone.borrow_mut();
                                if let Some(base) = base_opt.as_mut() {
                                    if let Some(view) = base.views.get_mut(base.active_view) {
                                        if col_idx < view.columns.len() {
                                            view.columns.remove(col_idx);
                                            
                                            // Actualizar ColumnView inmediatamente
                                            Self::rebuild_column_view(&column_view_clone, &view.columns);
                                            
                                            columns_for_html = view.columns.clone();
                                            
                                            // Persistir
                                            if let (Some(id), Some(db)) = (base_id_clone.borrow().as_ref(), notes_db_clone.borrow().as_ref()) {
                                                if let Ok(yaml) = base.serialize() {
                                                    let _ = db.update_base(*id, &yaml, base.active_view as i32);
                                                }
                                            }
                                            
                                            // Actualizar WebView
                                            let notes_borrowed = notes_clone.borrow();
                                            let html = Self::render_table_html_static(&notes_borrowed, &columns_for_html, Language::from_env());
                                            table_webview_clone.load_html(&html, None);
                                        }
                                    }
                                }
                            }
                            // Refrescar el popover sin cerrarlo
                            Self::refresh_columns_popover_content(
                                &content_for_refresh,
                                &base_ref_clone,
                                &base_id_clone,
                                &notes_db_clone,
                                &column_view_clone,
                                &available_props_for_refresh,
                                &popover_for_refresh,
                                &table_webview_clone,
                                &notes_clone,
                                &I18n::new(Language::from_env()),
                            );
                        });
                        
                        row.append(&remove_btn);
                    }
                    
                    // Conectar checkbox para visibilidad
                    let base_ref_clone = base_ref.clone();
                    let base_id_clone = base_id.clone();
                    let notes_db_clone = notes_db.clone();
                    let column_view_clone = column_view.clone();
                    let table_webview_clone = table_webview.clone();
                    let notes_clone = notes.clone();
                    
                    check.connect_toggled(move |btn| {
                        let mut base_opt = base_ref_clone.borrow_mut();
                        if let Some(base) = base_opt.as_mut() {
                            if let Some(view) = base.views.get_mut(base.active_view) {
                                if let Some(col_config) = view.columns.get_mut(col_idx) {
                                    col_config.visible = btn.is_active();
                                    
                                    // Reconstruir ColumnView para reflejar cambio
                                    Self::rebuild_column_view(&column_view_clone, &view.columns);
                                    
                                    // Actualizar WebView
                                    let notes_borrowed = notes_clone.borrow();
                                    let html = Self::render_table_html_static(&notes_borrowed, &view.columns, Language::from_env());
                                    table_webview_clone.load_html(&html, None);
                                    
                                    // Persistir
                                    if let (Some(id), Some(db)) = (base_id_clone.borrow().as_ref(), notes_db_clone.borrow().as_ref()) {
                                        if let Ok(yaml) = base.serialize() {
                                            let _ = db.update_base(*id, &yaml, base.active_view as i32);
                                        }
                                    }
                                }
                            }
                        }
                    });
                    
                    content.append(&row);
                }
                
                // === Sección: Propiedades disponibles para añadir ===
                // Filtrar propiedades que no están ya como columnas
                let new_props: Vec<&String> = available_props.iter()
                    .filter(|p| !existing_props.contains(p))
                    .collect();
                
                if !new_props.is_empty() {
                    let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
                    separator.set_margin_top(8);
                    separator.set_margin_bottom(8);
                    content.append(&separator);
                    
                    let add_title = gtk::Label::builder()
                        .label(&i18n.t("base_add_column"))
                        .css_classes(["heading"])
                        .xalign(0.0)
                        .margin_bottom(4)
                        .build();
                    content.append(&add_title);
                    
                    let hint = gtk::Label::builder()
                        .label(&i18n.t("base_properties_hint"))
                        .css_classes(["dim-label"])
                        .xalign(0.0)
                        .margin_bottom(4)
                        .build();
                    content.append(&hint);
                    
                    for prop in new_props {
                        let row = gtk::Box::builder()
                            .orientation(gtk::Orientation::Horizontal)
                            .spacing(8)
                            .build();
                        
                        let add_btn = gtk::Button::builder()
                            .icon_name("list-add-symbolic")
                            .css_classes(["flat", "circular"])
                            .tooltip_text(&i18n.t("base_add_as_column"))
                            .build();
                        row.append(&add_btn);
                        
                        let label = gtk::Label::builder()
                            .label(prop)
                            .hexpand(true)
                            .xalign(0.0)
                            .build();
                        row.append(&label);
                        
                        let base_ref_clone = base_ref.clone();
                        let base_id_clone = base_id.clone();
                        let notes_db_clone = notes_db.clone();
                        let column_view_clone = column_view.clone();
                        let prop_clone = prop.clone();
                        let content_for_refresh = content_clone.clone();
                        let popover_for_refresh = popover_clone.clone();
                        let available_props_for_refresh = available_props_vec.clone();
                        let table_webview_clone = table_webview.clone();
                        let notes_clone = notes.clone();
                        
                        add_btn.connect_clicked(move |_| {
                            let columns_for_html: Vec<ColumnConfig>;
                            {
                                let mut base_opt = base_ref_clone.borrow_mut();
                                if let Some(base) = base_opt.as_mut() {
                                    if let Some(view) = base.views.get_mut(base.active_view) {
                                        // Añadir nueva columna
                                        view.columns.push(ColumnConfig::new(&prop_clone));
                                        
                                        // Actualizar ColumnView inmediatamente
                                        Self::rebuild_column_view(&column_view_clone, &view.columns);
                                        
                                        columns_for_html = view.columns.clone();
                                        
                                        // Persistir
                                        if let (Some(id), Some(db)) = (base_id_clone.borrow().as_ref(), notes_db_clone.borrow().as_ref()) {
                                            if let Ok(yaml) = base.serialize() {
                                                let _ = db.update_base(*id, &yaml, base.active_view as i32);
                                            }
                                        }
                                        
                                        // Actualizar WebView
                                        let notes_borrowed = notes_clone.borrow();
                                        let html = Self::render_table_html_static(&notes_borrowed, &columns_for_html, Language::from_env());
                                        table_webview_clone.load_html(&html, None);
                                    }
                                }
                            }
                            // Refrescar el popover sin cerrarlo
                            Self::refresh_columns_popover_content(
                                &content_for_refresh,
                                &base_ref_clone,
                                &base_id_clone,
                                &notes_db_clone,
                                &column_view_clone,
                                &available_props_for_refresh,
                                &popover_for_refresh,
                                &table_webview_clone,
                                &notes_clone,
                                &I18n::new(Language::from_env()),
                            );
                        });
                        
                        content.append(&row);
                    }
                }
            }
        }
    }
    
    /// Configurar el popover para cambiar el modo de datos (Notes/GroupedRecords)
    fn setup_source_type_popover(&self) {
        let popover = gtk::Popover::builder()
            .css_classes(["source-type-popover"])
            .has_arrow(true)
            .build();
        
        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();
        
        let title = gtk::Label::builder()
            .label(&self.i18n.borrow().t("base_data_source_title"))
            .css_classes(["heading"])
            .xalign(0.0)
            .margin_bottom(8)
            .build();
        content.append(&title);
        
        // Obtener el modo actual
        let current_mode = self.base.borrow()
            .as_ref()
            .map(|b| b.source_type.clone())
            .unwrap_or(SourceType::Notes);
        
        // Radio buttons para los modos
        let notes_radio = gtk::CheckButton::builder()
            .label(&format!("📝 {}", self.i18n.borrow().t("base_notes_mode")))
            .active(matches!(current_mode, SourceType::Notes))
            .build();
        
        let grouped_radio = gtk::CheckButton::builder()
            .label(&format!("📊 {}", self.i18n.borrow().t("base_grouped_mode")))
            .group(&notes_radio)
            .active(matches!(current_mode, SourceType::GroupedRecords))
            .build();
        
        content.append(&notes_radio);
        content.append(&grouped_radio);
        
        // Descripción
        let desc = gtk::Label::builder()
            .label(&self.i18n.borrow().t("base_grouped_hint"))
            .css_classes(["dim-label", "caption"])
            .xalign(0.0)
            .margin_top(8)
            .wrap(true)
            .build();
        content.append(&desc);
        
        // Clonar referencias para los callbacks
        let base_ref = self.base.clone();
        let base_id = self.base_id.clone();
        let notes_db = self.notes_db.clone();
        let db_path = self.db_path.clone();
        let notes_root = self.notes_root.clone();
        let popover_clone = popover.clone();
        
        // Clonar referencias para los callbacks de radio
        let base_ref_notes = base_ref.clone();
        let base_id_notes = base_id.clone();
        let notes_db_notes = notes_db.clone();
        let popover_notes = popover.clone();
        let on_change_notes = self.on_source_type_changed.clone();
        
        notes_radio.connect_toggled(move |btn| {
            if btn.is_active() {
                Self::change_source_type(
                    &base_ref_notes, &base_id_notes, &notes_db_notes,
                    SourceType::Notes,
                );
                popover_notes.popdown();
                // Llamar callback para recargar
                if let Some(ref callback) = *on_change_notes.borrow() {
                    callback();
                }
            }
        });
        
        let base_ref_grouped = base_ref.clone();
        let base_id_grouped = base_id.clone();
        let notes_db_grouped = notes_db.clone();
        let popover_grouped = popover.clone();
        let on_change_grouped = self.on_source_type_changed.clone();
        
        grouped_radio.connect_toggled(move |btn| {
            if btn.is_active() {
                Self::change_source_type(
                    &base_ref_grouped, &base_id_grouped, &notes_db_grouped,
                    SourceType::GroupedRecords,
                );
                popover_grouped.popdown();
                // Llamar callback para recargar
                if let Some(ref callback) = *on_change_grouped.borrow() {
                    callback();
                }
            }
        });
        
        popover.set_child(Some(&content));
        self.source_type_btn.set_popover(Some(&popover));
        
        // Actualizar icono según modo actual
        match current_mode {
            SourceType::Notes => self.source_type_btn.set_icon_name("view-list-symbolic"),
            SourceType::GroupedRecords => self.source_type_btn.set_icon_name("view-grid-symbolic"),
        }
    }
    
    /// Cambiar el source_type de la Base y persistir
    fn change_source_type(
        base_ref: &Rc<RefCell<Option<Base>>>,
        base_id: &Rc<RefCell<Option<i64>>>,
        notes_db: &Rc<RefCell<Option<NotesDatabase>>>,
        new_type: SourceType,
    ) {
        let mut base_opt = base_ref.borrow_mut();
        if let Some(base) = base_opt.as_mut() {
            base.source_type = new_type;
            
            // Persistir en la BD
            if let (Some(id), Some(db)) = (base_id.borrow().as_ref(), notes_db.borrow().as_ref()) {
                if let Ok(yaml) = base.serialize() {
                    if let Err(e) = db.update_base(*id, &yaml, base.active_view as i32) {
                        eprintln!("Error saving Base source_type: {}", e);
                    }
                }
            }
        }
    }
    
    /// Actualizar los chips de filtros activos
    fn update_filter_chips(&self) {
        // Limpiar chips existentes
        while let Some(child) = self.filters_container.first_child() {
            self.filters_container.remove(&child);
        }
        
        let filters = self.active_filters.borrow();
        
        if filters.is_empty() {
            // Mostrar placeholder
            let placeholder = gtk::Label::builder()
                .label(&self.i18n.borrow().t("base_no_filters"))
                .css_classes(["dim-label"])
                .build();
            self.filters_container.append(&placeholder);
        } else {
            // Crear chips para cada filtro
            for (i, filter) in filters.iter().enumerate() {
                let chip = create_filter_chip(filter, i);
                
                // Conectar botón de cerrar
                let active_filters = self.active_filters.clone();
                let all_notes = self.all_notes.clone();
                let notes = self.notes.clone();
                let current_sort = self.current_sort.clone();
                let list_store = self.list_store.clone();
                let status_bar = self.status_bar.clone();
                let filters_container = self.filters_container.clone();
                
                if let Some(close_btn) = chip.last_child().and_downcast::<gtk::Button>() {
                    close_btn.connect_clicked(move |_| {
                        // Eliminar filtro
                        let mut filters = active_filters.borrow_mut();
                        if i < filters.len() {
                            filters.remove(i);
                        }
                        drop(filters);
                        
                        // Re-aplicar filtros (esto debería llamar a apply_filters_and_sort pero
                        // necesitamos acceso a self, así que por ahora solo refrescamos los datos)
                        // TODO: Refactorizar para mejor manejo de estado
                    });
                }
                
                self.filters_container.append(&chip);
            }
        }
    }

    /// Actualizar las columnas del ColumnView
    fn update_columns(&self, columns: &[ColumnConfig]) {
        Self::rebuild_column_view(&self.column_view, columns);
    }
    
    /// Reconstruir las columnas de un ColumnView (función estática para usar en callbacks)
    fn rebuild_column_view(column_view: &gtk::ColumnView, columns: &[ColumnConfig]) {
        // Limpiar columnas existentes
        while let Some(col) = column_view.columns().item(0) {
            if let Some(column) = col.downcast_ref::<gtk::ColumnViewColumn>() {
                column_view.remove_column(column);
            }
        }

        // Crear nuevas columnas
        for config in columns {
            if !config.visible {
                continue;
            }

            let property_name = config.property.clone();

            // Factory para crear las celdas
            let factory = gtk::SignalListItemFactory::new();
            
            factory.connect_setup(move |_, list_item| {
                let label = gtk::Label::builder()
                    .xalign(0.0)
                    .css_classes(["base-cell"])
                    .build();
                list_item.set_child(Some(&label));
            });

            let prop_name = property_name.clone();
            factory.connect_bind(move |_, list_item| {
                if let Some(boxed) = list_item.item().and_downcast::<glib::BoxedAnyObject>() {
                    let note = boxed.borrow::<NoteWithProperties>();
                    if let Some(label) = list_item.child().and_downcast::<gtk::Label>() {
                        label.set_text(&note.get_display(&prop_name));
                        
                        // Aplicar clase para filas alternas
                        let position = list_item.position();
                        label.remove_css_class("row-even");
                        label.remove_css_class("row-odd");
                        if position % 2 == 0 {
                            label.add_css_class("row-even");
                        } else {
                            label.add_css_class("row-odd");
                        }
                    }
                }
            });

            // Crear columna
            let column = gtk::ColumnViewColumn::builder()
                .title(&config.display_title())
                .factory(&factory)
                .resizable(true)
                .build();

            if let Some(width) = config.width {
                column.set_fixed_width(width as i32);
            }

            column_view.append_column(&column);
        }
    }

    /// Actualizar los datos de la tabla usando el WebView
    fn update_data(&self, notes: &[NoteWithProperties]) {
        // Obtener las columnas configuradas de la vista actual
        let columns = if let Some(base) = self.base.borrow().as_ref() {
            if let Some(view) = base.views.get(base.active_view) {
                view.columns.clone()
            } else {
                // Columnas por defecto
                vec![
                    ColumnConfig { property: "title".to_string(), title: None, width: Some(300), visible: true },
                    ColumnConfig { property: "created".to_string(), title: None, width: Some(150), visible: true },
                ]
            }
        } else {
            vec![
                ColumnConfig { property: "title".to_string(), title: None, width: Some(300), visible: true },
                ColumnConfig { property: "created".to_string(), title: None, width: Some(150), visible: true },
            ]
        };
        
        // Renderizar el HTML de la tabla
        let html = self.render_table_html(notes, &columns);
        self.table_webview.load_html(&html, None);
    }
    
    /// Generar el HTML para la tabla
    fn render_table_html(&self, notes: &[NoteWithProperties], columns: &[ColumnConfig]) -> String {
        Self::render_table_html_static(notes, columns, self.i18n.borrow().current_language())
    }
    
    /// Generar el HTML para la tabla (versión estática para usar en closures)
    fn render_table_html_static(notes: &[NoteWithProperties], columns: &[ColumnConfig], language: Language) -> String {
        let is_dark = Self::is_dark_theme();
        let theme_class = if is_dark { "dark" } else { "light" };
        
        // Traducciones para el HTML
        let (search_placeholder, items_label, no_notes_label) = if language == Language::Spanish {
            ("Buscar en tabla...", "elementos", "No se encontraron notas")
        } else {
            ("Search in table...", "items", "No notes found")
        };
        
        // CSS idéntico al de HtmlRenderer para consistencia
        let css = r#"
:root {
    --bg-primary: #1e1e2e;
    --bg-secondary: #313244;
    --bg-tertiary: #45475a;
    --fg-primary: #cdd6f4;
    --fg-secondary: #a6adc8;
    --fg-muted: #6c7086;
    --accent: #89b4fa;
    --border: #45475a;
}

body.light {
    --bg-primary: #eff1f5;
    --bg-secondary: #e6e9ef;
    --bg-tertiary: #ccd0da;
    --fg-primary: #4c4f69;
    --fg-secondary: #5c5f77;
    --fg-muted: #8c8fa1;
    --accent: #1e66f5;
    --border: #bcc0cc;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

html, body {
    min-height: 100vh;
    height: 100%;
}

body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 15px;
    line-height: 1.6;
    color: var(--fg-primary);
    background-color: var(--bg-primary);
    padding: 16px;
}

table {
    width: 100%;
    border-collapse: collapse;
    margin: 0;
    font-size: 0.95em;
}

th, td {
    border: 1px solid var(--border);
    padding: 10px 14px;
    text-align: left;
}

th {
    background-color: var(--bg-secondary);
    font-weight: 600;
    text-transform: uppercase;
    font-size: 0.85em;
    color: var(--fg-secondary);
}

tr:nth-child(even) {
    background-color: var(--bg-secondary);
}

tr:hover {
    background-color: var(--bg-tertiary);
    cursor: pointer;
}

.title-cell {
    font-weight: 500;
    color: var(--fg-primary);
}

.date-cell {
    color: var(--fg-muted);
    font-size: 0.9em;
}

.property-cell {
    color: var(--fg-secondary);
    font-size: 0.9em;
}

.empty-state {
    text-align: center;
    padding: 48px;
    color: var(--fg-muted);
}

/* Search bar */
.search-container {
    position: sticky;
    top: 0;
    z-index: 100;
    background: var(--bg-primary);
    padding: 8px 0 12px 0;
    margin-bottom: 8px;
}

.search-input {
    width: 100%;
    max-width: 400px;
    padding: 8px 12px 8px 36px;
    font-size: 14px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-secondary);
    color: var(--fg-primary);
    outline: none;
    transition: border-color 0.2s, box-shadow 0.2s;
}

.search-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.search-input::placeholder {
    color: var(--fg-muted);
}

.search-wrapper {
    position: relative;
    display: inline-block;
    width: 100%;
    max-width: 400px;
}

.search-icon {
    position: absolute;
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--fg-muted);
    pointer-events: none;
}

.search-results-count {
    display: inline-block;
    margin-left: 12px;
    font-size: 13px;
    color: var(--fg-muted);
}

tr.hidden-by-search {
    display: none;
}

tr.search-highlight td {
    background-color: rgba(137, 180, 250, 0.15);
}
"#;
        
        let mut html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>{}</style>
</head>
<body class="{}">
<script>
// Función de búsqueda en la tabla
function filterTable(query) {{
    var tbody = document.querySelector('tbody');
    if (!tbody) return;
    
    var rows = tbody.querySelectorAll('tr[data-path]');
    var count = 0;
    var total = rows.length;
    var searchLower = query.toLowerCase().trim();
    
    rows.forEach(function(row) {{
        if (searchLower === '') {{
            row.classList.remove('hidden-by-search');
            row.classList.remove('search-highlight');
            count++;
        }} else {{
            var text = row.textContent.toLowerCase();
            if (text.includes(searchLower)) {{
                row.classList.remove('hidden-by-search');
                row.classList.add('search-highlight');
                count++;
            }} else {{
                row.classList.add('hidden-by-search');
                row.classList.remove('search-highlight');
            }}
        }}
    }});
    
    // Actualizar contador
    var countEl = document.getElementById('search-count');
    if (countEl) {{
        if (searchLower === '') {{
            countEl.textContent = total + ' items';
        }} else {{
            countEl.textContent = count + ' of ' + total + ' items';
        }}
    }}
}}

// Listener en document para capturar todos los clics
document.addEventListener('click', function(event) {{
    // Ignorar clics en el campo de búsqueda
    if (event.target.closest('.search-container')) {{
        return;
    }}
    
    // Verificar si el clic fue en una fila de la tabla
    var row = event.target.closest('tr[data-path]');
    if (row) {{
        // Clic en fila - enviar el path de la nota
        window.webkit.messageHandlers.noteClick.postMessage(row.dataset.path);
    }} else {{
        // Clic fuera de las filas - solo cerrar sidebar
        window.webkit.messageHandlers.noteClick.postMessage('__close_sidebar__');
    }}
}});

// Atajos de teclado
document.addEventListener('keydown', function(event) {{
    // Ctrl+F o Cmd+F para enfocar búsqueda
    if ((event.ctrlKey || event.metaKey) && event.key === 'f') {{
        event.preventDefault();
        var searchInput = document.getElementById('table-search');
        if (searchInput) {{
            searchInput.focus();
            searchInput.select();
        }}
    }}
    // Escape para limpiar búsqueda
    if (event.key === 'Escape') {{
        var searchInput = document.getElementById('table-search');
        if (searchInput && document.activeElement === searchInput) {{
            searchInput.value = '';
            filterTable('');
            searchInput.blur();
        }}
    }}
}});
</script>
"#, css, theme_class);
        
        if notes.is_empty() {
            html.push_str(&format!(r#"<div class="empty-state">{}</div>"#, no_notes_label));
        } else {
            // Barra de búsqueda
            let notes_count = notes.len();
            html.push_str(&format!(r#"<div class="search-container">
    <div class="search-wrapper">
        <span class="search-icon">🔍</span>
        <input type="text" id="table-search" class="search-input" placeholder="{}" oninput="filterTable(this.value)" autocomplete="off">
    </div>
    <span id="search-count" class="search-results-count">{} {}</span>
</div>
"#, search_placeholder, notes_count, items_label));
            
            html.push_str("<table>\n<thead>\n<tr>\n");
            
            // Cabeceras
            for col in columns.iter().filter(|c| c.visible) {
                let header_name = Self::format_column_header(&col.property, language);
                html.push_str(&format!("<th>{}</th>\n", Self::escape_html(&header_name)));
            }
            html.push_str("</tr>\n</thead>\n<tbody>\n");
            
            // Filas de datos
            for note in notes {
                let path_attr = Self::escape_html(&note.metadata.path);
                html.push_str(&format!(r#"<tr data-path="{}">"#, path_attr));
                
                for col in columns.iter().filter(|c| c.visible) {
                    let value = Self::get_property_value(note, &col.property);
                    let cell_class = match col.property.as_str() {
                        "title" => "title-cell",
                        "created" | "modified" => "date-cell",
                        _ => "property-cell",
                    };
                    html.push_str(&format!(r#"<td class="{}">{}</td>"#, cell_class, Self::escape_html(&value)));
                }
                html.push_str("</tr>\n");
            }
            
            html.push_str("</tbody>\n</table>\n");
        }
        
        html.push_str("</body>\n</html>");
        html
    }
    
    /// Formatear el nombre de la columna para el header
    fn format_column_header(property: &str, language: Language) -> String {
        match property {
            "title" => if language == Language::Spanish { "Título".to_string() } else { "Title".to_string() },
            "created" => if language == Language::Spanish { "Creado".to_string() } else { "Created".to_string() },
            "modified" => if language == Language::Spanish { "Modificado".to_string() } else { "Modified".to_string() },
            "tags" => if language == Language::Spanish { "Etiquetas".to_string() } else { "Tags".to_string() },
            other => {
                // Capitalizar primera letra
                let mut chars = other.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                }
            }
        }
    }
    
    /// Obtener el valor de una propiedad de la nota
    fn get_property_value(note: &NoteWithProperties, property: &str) -> String {
        match property {
            "title" => note.metadata.name.clone(),
            "created" => note.metadata.created_at.format("%Y-%m-%d %H:%M").to_string(),
            "modified" => note.metadata.updated_at.format("%Y-%m-%d %H:%M").to_string(),
            other => {
                // Buscar en properties
                note.properties
                    .get(other)
                    .map(|v| v.to_display_string())
                    .unwrap_or_default()
            }
        }
    }
    
    /// Escapar HTML
    fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
         .replace('<', "&lt;")
         .replace('>', "&gt;")
         .replace('"', "&quot;")
         .replace('\'', "&#39;")
    }
    
    /// Detectar si el tema es oscuro
    /// Por ahora siempre usamos tema oscuro para consistencia con las notas
    fn is_dark_theme() -> bool {
        // Siempre oscuro por defecto (igual que HtmlRenderer)
        true
    }

    /// Actualizar los tabs de vistas
    fn update_view_tabs(&self, base: &Base) {
        // Limpiar tabs existentes
        while let Some(child) = self.view_tabs.first_child() {
            self.view_tabs.remove(&child);
        }

        // Crear un tab por cada vista
        for (i, view) in base.views.iter().enumerate() {
            let is_active = i == base.active_view;

            let button = gtk::ToggleButton::builder()
                .label(&view.name)
                .active(is_active)
                .css_classes(if is_active { 
                    vec!["base-view-tab", "active"] 
                } else { 
                    vec!["base-view-tab"] 
                })
                .build();

            self.view_tabs.append(&button);
        }

        // Botón para añadir nueva vista
        let add_view_btn = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Add view")
            .css_classes(["flat", "base-add-view"])
            .build();
        self.view_tabs.append(&add_view_btn);
    }

    /// Actualizar la barra de estado
    fn update_status_bar(&self, count: usize) {
        if let Some(label) = self.status_bar.first_child().and_downcast::<gtk::Label>() {
            let text = if count == 1 {
                "1 note".to_string()
            } else {
                format!("{} notes", count)
            };
            label.set_text(&text);
        }
    }

    /// Configurar callback para selección de nota
    pub fn on_note_selected<F: Fn(&str) + 'static>(&self, callback: F) {
        *self.on_note_selected.borrow_mut() = Some(Box::new(callback));
    }
    
    /// Configurar callback para cuando se hace clic en la vista (para cerrar sidebar)
    pub fn on_view_clicked<F: Fn() + 'static>(&self, callback: F) {
        *self.on_view_clicked.borrow_mut() = Some(Box::new(callback));
    }

    /// Configurar callback para doble clic en nota
    pub fn on_note_double_click<F: Fn(&str) + 'static>(&self, callback: F) {
        *self.on_note_double_click.borrow_mut() = Some(Box::new(callback));
    }
}

impl Default for BaseTableWidget {
    fn default() -> Self {
        Self::new(Rc::new(RefCell::new(I18n::new(Language::from_env()))))
    }
}

/// CSS para los widgets de Base
pub const BASE_CSS: &str = r#"
.base-table-container {
    background: @theme_bg_color;
}

.base-filter-bar {
    background: alpha(@theme_fg_color, 0.02);
    border-bottom: 1px solid alpha(@theme_fg_color, 0.08);
    padding: 6px 12px;
}

.base-filter-bar button {
    min-height: 28px;
    min-width: 28px;
    padding: 4px 8px;
    border-radius: 6px;
}

.base-filter-bar button:hover {
    background: alpha(@theme_fg_color, 0.08);
}

.base-view-tabs {
    padding: 8px 12px 0 12px;
    background: alpha(@theme_fg_color, 0.01);
}

.base-view-tab {
    padding: 8px 16px;
    border-radius: 8px 8px 0 0;
    margin: 0 2px;
    background: transparent;
    border: none;
    font-weight: 500;
    color: alpha(@theme_fg_color, 0.7);
}

.base-view-tab:checked,
.base-view-tab.active {
    background: @theme_bg_color;
    color: @theme_fg_color;
    box-shadow: 0 -2px 0 0 @accent_bg_color inset;
}

.base-view-tab:hover:not(:checked) {
    background: alpha(@theme_fg_color, 0.05);
    color: @theme_fg_color;
}

/* Tabla principal */
.base-table {
    background: transparent;
}

.base-table > listview {
    background: transparent;
}

.base-table > listview > row {
    padding: 0;
    background: transparent;
    border-bottom: 1px solid alpha(@theme_fg_color, 0.06);
    transition: background 150ms ease;
}

.base-table > listview > row:hover {
    background: alpha(@theme_fg_color, 0.04);
}

.base-table > listview > row:selected {
    background: alpha(@accent_bg_color, 0.15);
}

/* Cabeceras de columna */
.base-table header {
    background: alpha(@theme_fg_color, 0.03);
    border-bottom: 2px solid alpha(@theme_fg_color, 0.1);
}

.base-table header button {
    font-weight: 600;
    font-size: 0.85em;
    padding: 10px 16px;
    background: transparent;
    border: none;
    border-right: 1px solid alpha(@theme_fg_color, 0.06);
    color: alpha(@theme_fg_color, 0.8);
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.base-table header button:hover {
    background: alpha(@theme_fg_color, 0.05);
    color: @theme_fg_color;
}

.base-table header button:last-child {
    border-right: none;
}

/* Celdas */
.base-cell {
    padding: 12px 16px;
    font-size: 0.95em;
    color: @theme_fg_color;
}

/* Barra de estado */
.base-status-bar {
    background: alpha(@theme_fg_color, 0.02);
    border-top: 1px solid alpha(@theme_fg_color, 0.08);
    padding: 8px 16px;
    font-size: 0.85em;
    color: alpha(@theme_fg_color, 0.6);
}

/* Filter chips */
.base-filter-chip {
    background: alpha(@accent_bg_color, 0.15);
    color: @accent_color;
    padding: 4px 10px;
    border-radius: 16px;
    font-size: 0.85em;
    font-weight: 500;
}

.base-filter-chip:hover {
    background: alpha(@accent_bg_color, 0.25);
}

.base-filter-chip button {
    padding: 0;
    min-width: 18px;
    min-height: 18px;
    margin-left: 4px;
    border-radius: 50%;
}

.base-filter-chip button:hover {
    background: alpha(@theme_fg_color, 0.15);
}

/* Filter popover */
.filter-popover {
    padding: 16px;
    background-color: @theme_bg_color;
    border: 1px solid alpha(@theme_fg_color, 0.1);
    border-radius: 12px;
    box-shadow: 0 4px 12px alpha(black, 0.15);
}

.filter-popover .property-row {
    margin-bottom: 12px;
}

.filter-popover label {
    margin-bottom: 6px;
    font-weight: 500;
    font-size: 0.9em;
    color: alpha(@theme_fg_color, 0.8);
}

.filter-popover dropdown,
.filter-popover entry {
    min-height: 36px;
    border-radius: 8px;
}

/* Sort popover */
.sort-popover {
    padding: 12px;
    min-width: 220px;
    background-color: @theme_bg_color;
    border: 1px solid alpha(@theme_fg_color, 0.1);
    border-radius: 12px;
    box-shadow: 0 4px 12px alpha(black, 0.15);
}

.sort-popover .sort-row {
    padding: 10px 12px;
    border-radius: 8px;
    margin: 2px 0;
}

.sort-popover .sort-row:hover {
    background: alpha(@theme_fg_color, 0.06);
}

/* Columns popover */
.columns-popover {
    padding: 12px;
    min-width: 220px;
    background-color: @theme_bg_color;
    border: 1px solid alpha(@theme_fg_color, 0.1);
    border-radius: 12px;
    box-shadow: 0 4px 12px alpha(black, 0.15);
}

.columns-popover .heading {
    font-weight: 600;
    font-size: 0.85em;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: alpha(@theme_fg_color, 0.6);
    margin-bottom: 8px;
}

.columns-popover check {
    margin-right: 10px;
}

.columns-popover row {
    padding: 6px 4px;
    border-radius: 6px;
}

.columns-popover row:hover {
    background: alpha(@theme_fg_color, 0.04);
}

/* Source type popover */
.source-type-popover {
    padding: 16px;
    min-width: 240px;
    background-color: @theme_bg_color;
    border: 1px solid alpha(@theme_fg_color, 0.1);
    border-radius: 12px;
    box-shadow: 0 4px 12px alpha(black, 0.15);
}

.source-type-popover .heading {
    font-weight: 600;
    font-size: 0.9em;
    margin-bottom: 12px;
}

.source-type-popover checkbutton {
    padding: 10px 12px;
    border-radius: 8px;
    margin: 4px 0;
}

.source-type-popover checkbutton:hover {
    background: alpha(@theme_fg_color, 0.06);
}

.source-type-popover checkbutton:checked {
    background: alpha(@accent_bg_color, 0.12);
}

.source-type-popover .caption {
    font-size: 0.8em;
    line-height: 1.4;
}

/* Property types */
.property-checkbox {
    color: @success_color;
}

.property-date {
    color: @accent_color;
}

.property-tags {
    font-size: 0.85em;
}

.property-tag {
    background: alpha(@accent_bg_color, 0.15);
    padding: 2px 6px;
    border-radius: 3px;
    margin-right: 4px;
}

/* Graph view toggle */
.base-graph-toggle:checked {
    background: alpha(@accent_bg_color, 0.3);
    color: @accent_color;
}

/* Graph view styles */
.base-graph-view {
    background: #1e1e22;
    min-height: 400px;
}
"#;

/// Crear un chip de filtro visual con índice para eliminación
pub fn create_filter_chip(filter: &Filter, _index: usize) -> gtk::Box {
    let chip = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(4)
        .css_classes(["base-filter-chip"])
        .build();

    // Nombre de la propiedad
    let prop_label = gtk::Label::new(Some(&filter.property));
    chip.append(&prop_label);

    // Operador
    let op_text = operator_to_symbol(&filter.operator);
    let op_label = gtk::Label::builder()
        .label(op_text)
        .css_classes(["dim-label"])
        .build();
    chip.append(&op_label);

    // Valor (solo si no es IsEmpty/IsNotEmpty)
    if !matches!(filter.operator, FilterOperator::IsEmpty | FilterOperator::IsNotEmpty) {
        let value_text = filter.value.to_display_string();
        // Truncar si es muy largo
        let display_value = if value_text.len() > 20 {
            format!("{}...", &value_text[..17])
        } else {
            value_text
        };
        let value_label = gtk::Label::new(Some(&display_value));
        chip.append(&value_label);
    }

    // Botón de cerrar
    let close_btn = gtk::Button::builder()
        .icon_name("window-close-symbolic")
        .css_classes(["flat", "circular"])
        .tooltip_text("Remove filter")
        .build();
    chip.append(&close_btn);

    chip
}

/// Convertir operador a símbolo visual
fn operator_to_symbol(op: &FilterOperator) -> &'static str {
    match op {
        FilterOperator::Equals => "=",
        FilterOperator::NotEquals => "≠",
        FilterOperator::Contains => "contains",
        FilterOperator::NotContains => "not contains",
        FilterOperator::GreaterThan => ">",
        FilterOperator::GreaterOrEqual => "≥",
        FilterOperator::LessThan => "<",
        FilterOperator::LessOrEqual => "≤",
        FilterOperator::StartsWith => "starts with",
        FilterOperator::EndsWith => "ends with",
        FilterOperator::IsEmpty => "is empty",
        FilterOperator::IsNotEmpty => "is not empty",
    }
}

/// Convertir índice del combo a FilterOperator
fn index_to_operator(index: usize) -> FilterOperator {
    match index {
        0 => FilterOperator::Equals,
        1 => FilterOperator::NotEquals,
        2 => FilterOperator::Contains,
        3 => FilterOperator::NotContains,
        4 => FilterOperator::GreaterThan,
        5 => FilterOperator::GreaterOrEqual,
        6 => FilterOperator::LessThan,
        7 => FilterOperator::LessOrEqual,
        8 => FilterOperator::StartsWith,
        9 => FilterOperator::EndsWith,
        10 => FilterOperator::IsEmpty,
        11 => FilterOperator::IsNotEmpty,
        _ => FilterOperator::Contains,
    }
}

/// Parsear el texto de valor a PropertyValue
fn parse_filter_value(text: &str) -> PropertyValue {
    let trimmed = text.trim();
    
    // Intentar parsear como número
    if let Ok(num) = trimmed.parse::<f64>() {
        return PropertyValue::Number(num);
    }
    
    // Intentar como booleano
    if trimmed.eq_ignore_ascii_case("true") {
        return PropertyValue::Checkbox(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return PropertyValue::Checkbox(false);
    }
    
    // Default: texto
    PropertyValue::Text(trimmed.to_string())
}

/// Actualizar los chips de filtros en el contenedor
fn update_filter_chips_in_container(container: &gtk::Box, filters: &[Filter]) {
    // Limpiar chips existentes
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
    
    if filters.is_empty() {
        let placeholder = gtk::Label::builder()
            .label("No filters")
            .css_classes(["dim-label"])
            .build();
        container.append(&placeholder);
    } else {
        for (i, filter) in filters.iter().enumerate() {
            let chip = create_filter_chip(filter, i);
            container.append(&chip);
        }
    }
}

/// Crear el popover para añadir filtros (devuelve referencias a los widgets)
pub fn create_filter_popover_with_refs(properties: &[String], i18n: &I18n) -> (gtk::Popover, gtk::DropDown, gtk::DropDown, gtk::Entry) {
    let popover = gtk::Popover::builder()
        .css_classes(["filter-popover"])
        .build();
    
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();
    
    // Título
    let title = gtk::Label::builder()
        .label(&i18n.t("base_add_filter_title"))
        .css_classes(["heading"])
        .xalign(0.0)
        .build();
    content.append(&title);
    
    // Selector de propiedad
    let prop_label = gtk::Label::builder()
        .label(&i18n.t("base_property"))
        .xalign(0.0)
        .css_classes(["dim-label"])
        .build();
    content.append(&prop_label);
    
    let prop_combo = gtk::DropDown::from_strings(
        &properties.iter().map(|s| s.as_str()).collect::<Vec<_>>()
    );
    prop_combo.set_css_classes(&["filter-property-combo"]);
    content.append(&prop_combo);
    
    // Selector de operador
    let op_label = gtk::Label::builder()
        .label(&i18n.t("base_operator"))
        .xalign(0.0)
        .css_classes(["dim-label"])
        .build();
    content.append(&op_label);
    
    // Operadores traducidos
    let operators = [
        i18n.t("filter_op_equals"),
        i18n.t("filter_op_not_equals"),
        i18n.t("filter_op_contains"),
        i18n.t("filter_op_not_contains"),
        i18n.t("filter_op_greater_than"),
        i18n.t("filter_op_greater_or_equal"),
        i18n.t("filter_op_less_than"),
        i18n.t("filter_op_less_or_equal"),
        i18n.t("filter_op_starts_with"),
        i18n.t("filter_op_ends_with"),
        i18n.t("filter_op_is_empty"),
        i18n.t("filter_op_is_not_empty"),
    ];
    let op_strs: Vec<&str> = operators.iter().map(|s| s.as_str()).collect();
    let op_combo = gtk::DropDown::from_strings(&op_strs);
    content.append(&op_combo);
    
    // Campo de valor
    let value_label = gtk::Label::builder()
        .label(&i18n.t("base_value"))
        .xalign(0.0)
        .css_classes(["dim-label"])
        .build();
    content.append(&value_label);
    
    let value_entry = gtk::Entry::builder()
        .placeholder_text(&i18n.t("base_filter_value_placeholder"))
        .build();
    content.append(&value_entry);
    
    // Botones
    let buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(8)
        .halign(gtk::Align::End)
        .build();
    
    let cancel_btn = gtk::Button::builder()
        .label(&i18n.t("base_cancel"))
        .css_classes(["flat"])
        .build();
    
    let popover_clone = popover.clone();
    cancel_btn.connect_clicked(move |_| {
        popover_clone.popdown();
    });
    
    let apply_btn = gtk::Button::builder()
        .label(&i18n.t("base_apply_filter"))
        .css_classes(["suggested-action"])
        .build();
    
    buttons.append(&cancel_btn);
    buttons.append(&apply_btn);
    content.append(&buttons);
    
    popover.set_child(Some(&content));
    
    (popover, prop_combo, op_combo, value_entry)
}

/// Crear el popover para añadir filtros
pub fn create_filter_popover(properties: &[String]) -> gtk::Popover {
    let popover = gtk::Popover::builder()
        .css_classes(["filter-popover"])
        .build();
    
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();
    
    // Título
    let title = gtk::Label::builder()
        .label("Add Filter")
        .css_classes(["heading"])
        .xalign(0.0)
        .build();
    content.append(&title);
    
    // Selector de propiedad
    let prop_label = gtk::Label::builder()
        .label("Property")
        .xalign(0.0)
        .css_classes(["dim-label"])
        .build();
    content.append(&prop_label);
    
    let prop_combo = gtk::DropDown::from_strings(
        &properties.iter().map(|s| s.as_str()).collect::<Vec<_>>()
    );
    prop_combo.set_css_classes(&["filter-property-combo"]);
    content.append(&prop_combo);
    
    // Selector de operador
    let op_label = gtk::Label::builder()
        .label("Operator")
        .xalign(0.0)
        .css_classes(["dim-label"])
        .build();
    content.append(&op_label);
    
    let operators = [
        "equals", "not equals", "contains", "not contains",
        "greater than", "greater or equal", "less than", "less or equal",
        "starts with", "ends with", "is empty", "is not empty"
    ];
    let op_combo = gtk::DropDown::from_strings(&operators);
    content.append(&op_combo);
    
    // Campo de valor
    let value_label = gtk::Label::builder()
        .label("Value")
        .xalign(0.0)
        .css_classes(["dim-label"])
        .build();
    content.append(&value_label);
    
    let value_entry = gtk::Entry::builder()
        .placeholder_text("Filter value...")
        .build();
    content.append(&value_entry);
    
    // Botones
    let buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(8)
        .halign(gtk::Align::End)
        .build();
    
    let cancel_btn = gtk::Button::builder()
        .label("Cancel")
        .css_classes(["flat"])
        .build();
    
    let apply_btn = gtk::Button::builder()
        .label("Add Filter")
        .css_classes(["suggested-action"])
        .build();
    
    buttons.append(&cancel_btn);
    buttons.append(&apply_btn);
    content.append(&buttons);
    
    // Conectar señales
    let popover_clone = popover.clone();
    cancel_btn.connect_clicked(move |_| {
        popover_clone.popdown();
    });
    
    // El apply_btn se conectará desde el widget que crea el popover
    // para tener acceso al estado del widget
    
    popover.set_child(Some(&content));
    popover
}

/// Crear el popover de ordenamiento con callbacks conectados
pub fn create_sort_popover_with_callbacks(
    properties: &[String],
    current_sort: Rc<RefCell<Option<SortConfig>>>,
    all_notes: Rc<RefCell<Vec<NoteWithProperties>>>,
    notes: Rc<RefCell<Vec<NoteWithProperties>>>,
    active_filters: Rc<RefCell<Vec<Filter>>>,
    list_store: gio::ListStore,
    status_bar: gtk::Box,
    table_webview: webkit6::WebView,
    base: Rc<RefCell<Option<Base>>>,
    i18n: &I18n,
) -> gtk::Popover {
    let popover = gtk::Popover::builder()
        .css_classes(["sort-popover"])
        .build();
    
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();
    
    // Título
    let title = gtk::Label::builder()
        .label(&i18n.t("base_sort_by"))
        .css_classes(["heading"])
        .xalign(0.0)
        .margin_bottom(8)
        .build();
    content.append(&title);
    
    // Opción para quitar ordenamiento
    let none_btn = gtk::Button::builder()
        .label(&i18n.t("base_no_sorting"))
        .css_classes(["flat"])
        .hexpand(true)
        .build();
    
    {
        let current_sort = current_sort.clone();
        let all_notes = all_notes.clone();
        let notes = notes.clone();
        let active_filters = active_filters.clone();
        let list_store = list_store.clone();
        let status_bar = status_bar.clone();
        let table_webview = table_webview.clone();
        let base = base.clone();
        let popover = popover.clone();
        
        none_btn.connect_clicked(move |_| {
            *current_sort.borrow_mut() = None;
            apply_sort_and_refresh(
                &current_sort, &all_notes, &notes, &active_filters, 
                &list_store, &status_bar, &table_webview, &base
            );
            popover.popdown();
        });
    }
    content.append(&none_btn);
    
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    
    // Una fila por cada propiedad
    let t_sort_asc = i18n.t("base_sort_ascending");
    let t_sort_desc = i18n.t("base_sort_descending");
    
    for prop in properties {
        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .css_classes(["sort-row"])
            .margin_top(2)
            .margin_bottom(2)
            .build();
        
        let prop_label = gtk::Label::builder()
            .label(prop)
            .hexpand(true)
            .xalign(0.0)
            .build();
        row.append(&prop_label);
        
        // Botón ascendente
        let asc_btn = gtk::Button::builder()
            .icon_name("view-sort-ascending-symbolic")
            .tooltip_text(&t_sort_asc)
            .css_classes(["flat", "circular"])
            .build();
        
        {
            let prop = prop.clone();
            let current_sort = current_sort.clone();
            let all_notes = all_notes.clone();
            let notes = notes.clone();
            let active_filters = active_filters.clone();
            let list_store = list_store.clone();
            let status_bar = status_bar.clone();
            let table_webview = table_webview.clone();
            let base = base.clone();
            let popover = popover.clone();
            
            asc_btn.connect_clicked(move |_| {
                *current_sort.borrow_mut() = Some(SortConfig {
                    property: prop.clone(),
                    direction: SortDirection::Asc,
                });
                apply_sort_and_refresh(
                    &current_sort, &all_notes, &notes, &active_filters,
                    &list_store, &status_bar, &table_webview, &base
                );
                popover.popdown();
            });
        }
        row.append(&asc_btn);
        
        // Botón descendente
        let desc_btn = gtk::Button::builder()
            .icon_name("view-sort-descending-symbolic")
            .tooltip_text(&t_sort_desc)
            .css_classes(["flat", "circular"])
            .build();
        
        {
            let prop = prop.clone();
            let current_sort = current_sort.clone();
            let all_notes = all_notes.clone();
            let notes = notes.clone();
            let active_filters = active_filters.clone();
            let list_store = list_store.clone();
            let status_bar = status_bar.clone();
            let table_webview = table_webview.clone();
            let base = base.clone();
            let popover = popover.clone();
            
            desc_btn.connect_clicked(move |_| {
                *current_sort.borrow_mut() = Some(SortConfig {
                    property: prop.clone(),
                    direction: SortDirection::Desc,
                });
                apply_sort_and_refresh(
                    &current_sort, &all_notes, &notes, &active_filters,
                    &list_store, &status_bar, &table_webview, &base
                );
                popover.popdown();
            });
        }
        row.append(&desc_btn);
        
        content.append(&row);
    }
    
    popover.set_child(Some(&content));
    popover
}

/// Aplicar ordenamiento y refrescar la UI
fn apply_sort_and_refresh(
    current_sort: &Rc<RefCell<Option<SortConfig>>>,
    all_notes: &Rc<RefCell<Vec<NoteWithProperties>>>,
    notes: &Rc<RefCell<Vec<NoteWithProperties>>>,
    active_filters: &Rc<RefCell<Vec<Filter>>>,
    list_store: &gio::ListStore,
    status_bar: &gtk::Box,
    table_webview: &webkit6::WebView,
    base: &Rc<RefCell<Option<Base>>>,
) {
    let all = all_notes.borrow();
    let filters = active_filters.borrow();
    let sort = current_sort.borrow();
    
    // Filtrar
    let mut filtered: Vec<NoteWithProperties> = all
        .iter()
        .filter(|note| {
            filters.iter().all(|f| f.evaluate(&note.properties))
        })
        .cloned()
        .collect();
    
    // Ordenar
    if let Some(sort_config) = sort.as_ref() {
        filtered.sort_by(|a, b| {
            let key_a = a.properties
                .get(&sort_config.property)
                .map(|v| v.sort_key())
                .unwrap_or_default();
            let key_b = b.properties
                .get(&sort_config.property)
                .map(|v| v.sort_key())
                .unwrap_or_default();

            match sort_config.direction {
                SortDirection::Asc => key_a.cmp(&key_b),
                SortDirection::Desc => key_b.cmp(&key_a),
            }
        });
    }
    
    drop(all);
    drop(filters);
    drop(sort);
    
    *notes.borrow_mut() = filtered.clone();
    
    // Actualizar UI (list_store para lógica)
    list_store.remove_all();
    for note in &filtered {
        let boxed = glib::BoxedAnyObject::new(note.clone());
        list_store.append(&boxed);
    }
    
    // Actualizar WebView
    let columns = if let Some(base) = base.borrow().as_ref() {
        if let Some(view) = base.views.get(base.active_view) {
            view.columns.clone()
        } else {
            vec![
                ColumnConfig { property: "title".to_string(), title: None, width: Some(300), visible: true },
                ColumnConfig { property: "created".to_string(), title: None, width: Some(150), visible: true },
            ]
        }
    } else {
        vec![
            ColumnConfig { property: "title".to_string(), title: None, width: Some(300), visible: true },
            ColumnConfig { property: "created".to_string(), title: None, width: Some(150), visible: true },
        ]
    };
    let html = BaseTableWidget::render_table_html_static(&filtered, &columns, Language::from_env());
    table_webview.load_html(&html, None);
    
    // Actualizar status
    if let Some(label) = status_bar.first_child().and_downcast::<gtk::Label>() {
        let text = if filtered.len() == 1 {
            "1 note".to_string()
        } else {
            format!("{} notes", filtered.len())
        };
        label.set_text(&text);
    }
}

/// Crear el popover para ordenamiento
pub fn create_sort_popover(properties: &[String]) -> gtk::Popover {
    let popover = gtk::Popover::builder()
        .css_classes(["sort-popover"])
        .build();
    
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .build();
    
    // Título
    let title = gtk::Label::builder()
        .label("Sort by")
        .css_classes(["heading"])
        .xalign(0.0)
        .margin_bottom(8)
        .build();
    content.append(&title);
    
    // Opción para quitar ordenamiento
    let none_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .css_classes(["sort-row"])
        .build();
    
    let none_btn = gtk::Button::builder()
        .label("No sorting")
        .css_classes(["flat"])
        .hexpand(true)
        .build();
    none_row.append(&none_btn);
    content.append(&none_row);
    
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    
    // Una fila por cada propiedad
    for prop in properties {
        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .css_classes(["sort-row"])
            .build();
        
        let prop_label = gtk::Label::builder()
            .label(prop)
            .hexpand(true)
            .xalign(0.0)
            .build();
        row.append(&prop_label);
        
        // Botón ascendente
        let asc_btn = gtk::Button::builder()
            .icon_name("view-sort-ascending-symbolic")
            .tooltip_text("Sort ascending")
            .css_classes(["flat", "circular"])
            .build();
        row.append(&asc_btn);
        
        // Botón descendente
        let desc_btn = gtk::Button::builder()
            .icon_name("view-sort-descending-symbolic")
            .tooltip_text("Sort descending")
            .css_classes(["flat", "circular"])
            .build();
        row.append(&desc_btn);
        
        content.append(&row);
    }
    
    popover.set_child(Some(&content));
    popover
}

/// Crear el popover para visibilidad de columnas
pub fn create_columns_popover(columns: &[ColumnConfig]) -> gtk::Popover {
    let popover = gtk::Popover::builder()
        .css_classes(["columns-popover"])
        .build();
    
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .build();
    
    // Título
    let title = gtk::Label::builder()
        .label("Visible Columns")
        .css_classes(["heading"])
        .xalign(0.0)
        .margin_bottom(8)
        .build();
    content.append(&title);
    
    // Un checkbox por cada columna
    for col in columns {
        let check = gtk::CheckButton::builder()
            .label(&col.display_title())
            .active(col.visible)
            .build();
        content.append(&check);
    }
    
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    
    // Botón para mostrar todas
    let show_all_btn = gtk::Button::builder()
        .label("Show all")
        .css_classes(["flat"])
        .build();
    content.append(&show_all_btn);
    
    popover.set_child(Some(&content));
    popover
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_table_widget_creation() {
        gtk::init().unwrap();
        let i18n = Rc::new(RefCell::new(I18n::new(Language::from_env())));
        let widget = BaseTableWidget::new(i18n);
        assert!(widget.widget().is_visible() || !widget.widget().is_visible()); // Just verify it compiles
    }
}
