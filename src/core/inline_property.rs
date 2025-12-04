//! Parser de propiedades inline con sintaxis [campo::valor] o [campo:::valor]
//!
//! Soporta:
//! - Texto: [titulo::Mi título]
//! - Números: [precio::99.99]
//! - Booleanos: [completado::true]
//! - Fechas: [fecha::2025-11-29]
//! - Relaciones: [autor::@Cervantes]
//! - Listas: [tags::item1, item2]
//! - Propiedades agrupadas: [autor::Cervantes, libro::Quijote, año::1605]
//!   Las propiedades agrupadas comparten un group_id y forman un "registro"
//! - Propiedades ocultas: [campo:::valor] con triple dos puntos no se muestra visualmente
//!   pero sigue almacenándose en la base de datos

use regex::Regex;
use std::sync::LazyLock;

use super::property::PropertyValue;

/// Una propiedad inline extraída del contenido
#[derive(Debug, Clone)]
pub struct InlineProperty {
    /// Nombre del campo
    pub key: String,
    /// Valor tipado
    pub value: PropertyValue,
    /// Valor raw (como aparece en el texto)
    pub raw_value: String,
    /// Número de línea (1-indexed)
    pub line_number: usize,
    /// Posición de inicio en el contenido (bytes)
    pub char_start: usize,
    /// Posición de fin en el contenido (bytes)
    pub char_end: usize,
    /// Nombre de la nota referenciada (si es Link)
    pub linked_note: Option<String>,
    /// ID de grupo para propiedades agrupadas [a::1, b::2]
    /// None = propiedad individual, Some(n) = pertenece a grupo n
    pub group_id: Option<usize>,
    /// Si es true, la propiedad usa ::: y no se muestra visualmente
    /// pero sigue almacenándose en la base de datos
    pub hidden: bool,
}

impl InlineProperty {
    /// Obtener el texto completo de la propiedad como aparece en el archivo
    pub fn full_text(&self) -> String {
        let separator = if self.hidden { ":::" } else { "::" };
        format!("[{}{}{}]", self.key, separator, self.raw_value)
    }
}

// Regex para detectar [campo::valor] o [campo1::val1, campo2::val2]
// Captura todo el contenido entre [ y ]
// Soporta caracteres Unicode en nombres de campo (ej: año, título)
static INLINE_PROPERTY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]").unwrap());

// Regex para detectar un par campo::valor o campo:::valor dentro del contenido
// Soporta Unicode y permite espacios alrededor del :: o :::
// Grupo 1: nombre del campo
// Grupo 2: separador (:: o :::)
static PROPERTY_PAIR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\p{L}[\p{L}\p{N}_]*)\s*(:::?)\s*").unwrap());

/// Parser de propiedades inline
pub struct InlinePropertyParser;

impl InlinePropertyParser {
    /// Parsear todo el contenido y extraer propiedades inline
    /// Detecta tanto propiedades individuales [campo::valor] como
    /// propiedades agrupadas [campo1::val1, campo2::val2]
    pub fn parse(content: &str) -> Vec<InlineProperty> {
        let mut properties = Vec::new();
        let mut current_group_id: usize = 0;

        // Calcular offsets de línea para reportar line_number
        let line_offsets: Vec<usize> = std::iter::once(0)
            .chain(content.match_indices('\n').map(|(i, _)| i + 1))
            .collect();

        for cap in INLINE_PROPERTY_REGEX.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let inner_content = cap.get(1).unwrap().as_str();

            let char_start = full_match.start();
            let char_end = full_match.end();

            // Calcular número de línea
            let line_number = line_offsets
                .iter()
                .position(|&offset| offset > char_start)
                .unwrap_or(line_offsets.len());

            // Intentar parsear como propiedad(es)
            let parsed =
                Self::parse_bracket_content(inner_content, line_number, char_start, char_end);

            if !parsed.is_empty() {
                // Si hay múltiples propiedades, asignar group_id
                let group_id = if parsed.len() > 1 {
                    current_group_id += 1;
                    Some(current_group_id)
                } else {
                    None
                };

                for mut prop in parsed {
                    prop.group_id = group_id;
                    properties.push(prop);
                }
            }
        }

        properties
    }

    /// Parsear el contenido dentro de los corchetes [...]
    /// Detecta si es una propiedad simple o un grupo de propiedades
    fn parse_bracket_content(
        inner: &str,
        line_number: usize,
        char_start: usize,
        char_end: usize,
    ) -> Vec<InlineProperty> {
        let mut properties = Vec::new();

        // Verificar si contiene al menos un campo::valor
        if !PROPERTY_PAIR_REGEX.is_match(inner) {
            return properties; // No es una propiedad inline, ignorar
        }

        // Detectar si es agrupada: buscar ", campo::" patrón
        // Primero, reemplazar \, temporalmente para no confundir con separadores
        let escaped = inner.replace("\\,", "\x00ESCAPED_COMMA\x00");

        // Buscar todas las posiciones de campo::
        let pairs: Vec<_> = PROPERTY_PAIR_REGEX.find_iter(&escaped).collect();

        if pairs.len() == 1 {
            // Propiedad simple [campo::valor] o [campo:::valor]
            if let Some(cap) = PROPERTY_PAIR_REGEX.captures(&escaped) {
                let key = cap.get(1).unwrap().as_str().to_string();
                // Detectar si usa ::: (hidden) o :: (visible)
                let separator = cap.get(2).unwrap().as_str();
                let hidden = separator == ":::";
                let value_start = cap.get(0).unwrap().end();
                // Mantener el marcador para que parse_value no confunda con lista
                let value_with_marker = escaped[value_start..].trim().to_string();

                let (value, linked_note) = Self::parse_value(&value_with_marker);

                // raw_value sí tiene la coma restaurada (para display)
                let raw_value = value_with_marker.replace("\x00ESCAPED_COMMA\x00", ",");

                properties.push(InlineProperty {
                    key,
                    value,
                    raw_value,
                    line_number,
                    char_start,
                    char_end,
                    linked_note,
                    group_id: None,
                    hidden,
                });
            }
        } else {
            // Múltiples propiedades agrupadas [campo1::val1, campo2::val2]
            // Parsear cada par campo::valor
            for i in 0..pairs.len() {
                let key_match = PROPERTY_PAIR_REGEX
                    .captures(&escaped[pairs[i].start()..])
                    .unwrap();
                let key = key_match.get(1).unwrap().as_str().to_string();
                // Detectar si usa ::: (hidden) o :: (visible)
                let separator = key_match.get(2).unwrap().as_str();
                let hidden = separator == ":::";
                let value_start_in_inner = pairs[i].start() + key_match.get(0).unwrap().end();

                // El valor termina donde empieza el siguiente campo:: (menos la coma)
                // o al final del string
                let value_end = if i + 1 < pairs.len() {
                    // Buscar la coma antes del siguiente campo
                    let next_start = pairs[i + 1].start();
                    // Encontrar la última coma antes de next_start
                    escaped[value_start_in_inner..next_start]
                        .rfind(',')
                        .map(|pos| value_start_in_inner + pos)
                        .unwrap_or(next_start)
                } else {
                    escaped.len()
                };

                // Mantener el marcador para parse_value
                let value_with_marker = escaped[value_start_in_inner..value_end].trim().to_string();

                let (value, linked_note) = Self::parse_value(&value_with_marker);

                // raw_value sí tiene la coma restaurada
                let raw_value = value_with_marker.replace("\x00ESCAPED_COMMA\x00", ",");

                properties.push(InlineProperty {
                    key,
                    value,
                    raw_value,
                    line_number,
                    char_start,
                    char_end,
                    linked_note,
                    group_id: None, // Se asigna después en parse()
                    hidden,
                });
            }
        }

        properties
    }

    /// Parsear el valor y detectar su tipo
    /// El valor puede contener marcadores \x00ESCAPED_COMMA\x00 que se restauran a coma
    fn parse_value(raw: &str) -> (PropertyValue, Option<String>) {
        let trimmed = raw.trim();

        // Restaurar comas escapadas para el valor final
        let restore_comma = |s: &str| s.replace("\x00ESCAPED_COMMA\x00", ",");

        // 1. Relación (@nota)
        if trimmed.starts_with('@') {
            let note_name = restore_comma(&trimmed[1..]);
            return (PropertyValue::Link(note_name.clone()), Some(note_name));
        }

        // 2. Booleano
        if trimmed.eq_ignore_ascii_case("true") {
            return (PropertyValue::Checkbox(true), None);
        }
        if trimmed.eq_ignore_ascii_case("false") {
            return (PropertyValue::Checkbox(false), None);
        }

        // 3. Número (solo si no contiene comas que indiquen lista)
        // Nota: el marcador no es una coma real
        if !trimmed.contains(',') {
            let clean = restore_comma(trimmed);
            if let Ok(num) = clean.parse::<f64>() {
                return (PropertyValue::Number(num), None);
            }
        }

        // 4. Fecha (YYYY-MM-DD)
        if Self::is_date(trimmed) {
            return (PropertyValue::Date(trimmed.to_string()), None);
        }

        // 5. DateTime (YYYY-MM-DDTHH:MM:SS)
        if Self::is_datetime(trimmed) {
            return (PropertyValue::DateTime(trimmed.to_string()), None);
        }

        // 6. Lista: valores separados por coma
        if trimmed.contains(',') {
            let items: Vec<String> = trimmed
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if !items.is_empty() {
                // Detectar si son tags (#tag1, #tag2)
                if items.iter().all(|s| s.starts_with('#')) {
                    let tags = items
                        .iter()
                        .map(|s| s.trim_start_matches('#').to_string())
                        .collect();
                    return (PropertyValue::Tags(tags), None);
                }

                // Detectar si son relaciones (@nota1, @nota2)
                if items.iter().all(|s| s.starts_with('@')) {
                    let links = items
                        .iter()
                        .map(|s| s.trim_start_matches('@').to_string())
                        .collect();
                    return (PropertyValue::Links(links), None);
                }

                return (PropertyValue::List(items), None);
            }
        }

        // 7. Tags inline (#tag1 #tag2)
        if trimmed.starts_with('#') && trimmed.split_whitespace().all(|w| w.starts_with('#')) {
            let tags: Vec<String> = trimmed
                .split_whitespace()
                .map(|s| s.trim_start_matches('#').to_string())
                .collect();
            return (PropertyValue::Tags(tags), None);
        }

        // 8. Default: Texto (restaurar comas escapadas)
        (PropertyValue::Text(restore_comma(trimmed)), None)
    }

    /// Verificar si es fecha YYYY-MM-DD
    fn is_date(s: &str) -> bool {
        if s.len() != 10 {
            return false;
        }
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return false;
        }
        parts[0].len() == 4
            && parts[1].len() == 2
            && parts[2].len() == 2
            && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
    }

    /// Verificar si es datetime
    fn is_datetime(s: &str) -> bool {
        s.contains('T') && s.len() >= 19 && Self::is_date(&s[..10])
    }

    /// Reemplazar el valor de una propiedad en el contenido
    pub fn replace_property(content: &str, prop: &InlineProperty, new_value: &str) -> String {
        let mut result = String::with_capacity(content.len());
        result.push_str(&content[..prop.char_start]);
        result.push_str(&format!("[{}::{}]", prop.key, new_value));
        result.push_str(&content[prop.char_end..]);
        result
    }

    /// Insertar una nueva propiedad al final de una línea
    pub fn insert_property(content: &str, line: usize, key: &str, value: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::new();

        for (i, line_content) in lines.iter().enumerate() {
            result.push_str(line_content);
            if i + 1 == line {
                result.push_str(&format!(" [{}::{}]", key, value));
            }
            if i < lines.len() - 1 {
                result.push('\n');
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text() {
        let content = "Esto es [titulo::Mi Libro] de texto.";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 1);
        assert_eq!(props[0].key, "titulo");
        assert!(matches!(&props[0].value, PropertyValue::Text(s) if s == "Mi Libro"));
    }

    #[test]
    fn test_parse_number() {
        let content = "El precio es [precio::99.99] euros.";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 1);
        assert_eq!(props[0].key, "precio");
        assert!(matches!(&props[0].value, PropertyValue::Number(n) if (*n - 99.99).abs() < 0.001));
    }

    #[test]
    fn test_parse_boolean() {
        let content = "Estado: [completado::true] y [pendiente::false]";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 2);
        assert!(matches!(&props[0].value, PropertyValue::Checkbox(true)));
        assert!(matches!(&props[1].value, PropertyValue::Checkbox(false)));
    }

    #[test]
    fn test_parse_date() {
        let content = "Fecha: [fecha::2025-11-29]";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 1);
        assert!(matches!(&props[0].value, PropertyValue::Date(d) if d == "2025-11-29"));
    }

    #[test]
    fn test_parse_link() {
        let content = "Autor: [autor::@Cervantes]";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 1);
        assert!(matches!(&props[0].value, PropertyValue::Link(n) if n == "Cervantes"));
        assert_eq!(props[0].linked_note, Some("Cervantes".to_string()));
    }

    #[test]
    fn test_parse_list() {
        // Sintaxis de lista: valores separados por coma
        let content = "Items: [items::uno, dos, tres]";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 1);
        assert!(matches!(&props[0].value, PropertyValue::List(items) if items.len() == 3));
    }

    #[test]
    fn test_parse_multiple() {
        let content = r#"
# Mi Libro

[tipo::libro]
[titulo::Don Quijote]
[autor::@Cervantes]
[año::1605]
[paginas::863]
[leido::true]

Este es un gran libro.
"#;
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 6);
    }

    #[test]
    fn test_line_numbers() {
        let content = "Linea 1\n[campo::valor]\nLinea 3";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 1);
        assert_eq!(props[0].line_number, 2);
    }

    #[test]
    fn test_replace_property() {
        let content = "El [precio::100] es bajo.";
        let props = InlinePropertyParser::parse(content);
        let new_content = InlinePropertyParser::replace_property(content, &props[0], "200");

        assert_eq!(new_content, "El [precio::200] es bajo.");
    }

    #[test]
    fn test_grouped_properties() {
        // Propiedades agrupadas: [autor::Cervantes, libro::Quijote, año::1605]
        let content = "Referencia: [autor::Cervantes, libro::Quijote, año::1605]";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 3);

        // Verificar que todas tienen el mismo group_id
        assert!(props[0].group_id.is_some());
        assert_eq!(props[0].group_id, props[1].group_id);
        assert_eq!(props[1].group_id, props[2].group_id);

        // Verificar claves
        assert_eq!(props[0].key, "autor");
        assert_eq!(props[1].key, "libro");
        assert_eq!(props[2].key, "año");

        // Verificar valores
        assert!(matches!(&props[0].value, PropertyValue::Text(s) if s == "Cervantes"));
        assert!(matches!(&props[1].value, PropertyValue::Text(s) if s == "Quijote"));
        assert!(matches!(&props[2].value, PropertyValue::Number(n) if *n == 1605.0));
    }

    #[test]
    fn test_grouped_with_links() {
        // Grupo con enlaces a notas
        let content = "[tipo::libro, autor::@Cervantes, titulo::Quijote]";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 3);
        assert!(props[0].group_id.is_some());

        // El segundo es un link
        assert!(matches!(&props[1].value, PropertyValue::Link(n) if n == "Cervantes"));
        assert_eq!(props[1].linked_note, Some("Cervantes".to_string()));
    }

    #[test]
    fn test_escaped_comma() {
        // Coma escapada con \,
        let content = "[titulo::Cien años de soledad\\, novela]";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 1);
        assert_eq!(props[0].group_id, None); // No es grupo, solo una propiedad
        assert!(
            matches!(&props[0].value, PropertyValue::Text(s) if s == "Cien años de soledad, novela")
        );
    }

    #[test]
    fn test_individual_has_no_group() {
        // Propiedades individuales no tienen group_id
        let content = "[autor::Cervantes] escribió [libro::Quijote]";
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 2);
        assert_eq!(props[0].group_id, None);
        assert_eq!(props[1].group_id, None);
    }

    #[test]
    fn test_multiple_groups() {
        // Múltiples grupos en el mismo contenido
        let content = r#"
[autor::Cervantes, libro::Quijote]
[autor::Borges, libro::Ficciones]
"#;
        let props = InlinePropertyParser::parse(content);

        assert_eq!(props.len(), 4);

        // Grupo 1
        assert_eq!(props[0].group_id, Some(1));
        assert_eq!(props[1].group_id, Some(1));

        // Grupo 2
        assert_eq!(props[2].group_id, Some(2));
        assert_eq!(props[3].group_id, Some(2));

        // Grupos diferentes
        assert_ne!(props[0].group_id, props[2].group_id);
    }
}
