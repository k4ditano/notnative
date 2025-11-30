# Propiedades Inline en NotNative

## Descripción

NotNative soporta un sistema de propiedades inline que te permite añadir metadatos estructurados directamente en el contenido de tus notas usando una sintaxis simple.

## Sintaxis

### Propiedad Individual
```
[campo::valor]
```

### Propiedades Agrupadas (Registros)
```
[campo1::valor1, campo2::valor2, campo3::valor3]
```

Las propiedades agrupadas forman un "registro" relacionado. Esto es útil para representar relaciones complejas como autor-libro-año.

Las propiedades se pueden colocar en cualquier parte del contenido de la nota.

## Tipos de Valores Soportados

### Texto
```markdown
[titulo::Mi Libro Favorito]
[autor::Gabriel García Márquez]
```

### Números
```markdown
[precio::29.99]
[paginas::432]
[año::2024]
```

### Booleanos (Checkbox)
```markdown
[completado::true]
[pendiente::false]
```

### Fechas
```markdown
[fecha::2024-11-29]
[publicado::2023-05-15]
```

### Enlaces a Otras Notas (Relaciones)
```markdown
[autor::@Cervantes]
[relacionado::@Otra Nota]
```
El prefijo `@` indica que es un enlace a otra nota.

### Listas
```markdown
[tags::rust, gtk, gui]
[generos::novela, aventura, comedia]
```
Los valores separados por comas se convierten automáticamente en listas.

### Tags
```markdown
[etiquetas::#importante #urgente #trabajo]
```

### Múltiples Enlaces
```markdown
[referencias::@Nota1, @Nota2, @Nota3]
```

## Propiedades Agrupadas

Las propiedades agrupadas permiten crear "registros" de datos relacionados:

```markdown
# Mis lecturas

[autor::Cervantes, libro::Don Quijote, año::1605]
[autor::Cervantes, libro::Novelas Ejemplares, año::1613]
[autor::Borges, libro::Ficciones, año::1944]
```

### Características

- Todas las propiedades dentro de un `[...]` comparten un `group_id`
- Puedes buscar "todos los libros de Cervantes" y obtener los registros relacionados
- Los registros pueden estar en diferentes notas
- Ideal para catálogos, bibliografías, inventarios

### Escape de comas

Si necesitas una coma literal en un valor (que no sea separador), usa `\,`:

```markdown
[titulo::Cien años de soledad\, novela]
```

## Ejemplo Completo

```markdown
# Don Quijote de la Mancha

[tipo::libro]
[titulo::Don Quijote de la Mancha]
[autor::@Cervantes]
[año::1605]
[paginas::863]
[leido::true]
[genero::novela, aventura, comedia]
[nota::5]

Este es uno de los libros más importantes de la literatura española...

## Ediciones en mi colección

[edicion::Austral, año::1999, paginas::1200]
[edicion::Cátedra, año::2015, paginas::1400]
```

## Uso con Bases

Las propiedades inline son indexadas automáticamente y pueden usarse en las **Bases** para:

1. **Filtrar notas** - ej: "mostrar solo notas donde [leido::true]"
2. **Ordenar** - ej: "ordenar por [año]"
3. **Agrupar** - ej: "agrupar por [genero]"
4. **Mostrar columnas** - cada propiedad puede ser una columna en la vista de tabla
5. **Vista de grafo** - visualizar relaciones entre propiedades agrupadas

## Autocompletado

Al escribir `[campo::` el editor mostrará sugerencias de valores existentes para ese campo, facilitando la consistencia de datos.

## Ventajas sobre Frontmatter

- **Inline**: Las propiedades están donde las necesitas, no separadas al inicio
- **Múltiples valores**: Puedes tener varias propiedades con el mismo nombre en diferentes partes de la nota
- **Contextuales**: Añade propiedades junto al contenido relevante
- **Visibles**: Las propiedades son parte del contenido, no están ocultas
- **Agrupables**: Puedes crear registros de datos relacionados

## Consideraciones

- Los nombres de campo soportan caracteres Unicode (ej: `[año::2024]`)
- Los nombres de campo deben empezar con letra
- Los valores no pueden contener el carácter `]`
- Para listas, usa comas como separador

## Detección Automática de Tipos

El parser detecta automáticamente el tipo de valor:

| Patrón | Tipo |
|--------|------|
| `@nombre` | Link (relación) |
| `true`/`false` | Checkbox |
| Número válido | Number |
| `YYYY-MM-DD` | Date |
| Contiene `,` | List |
| `#tag1 #tag2` | Tags |
| Otro | Text |

## API para Desarrolladores

```rust
use notnative_app::core::{InlinePropertyParser, PropertyValue};

let content = "[precio::99.99] es el precio de [producto::Widget]";
let props = InlinePropertyParser::parse(content);

for prop in props {
    println!("{}: {} ({})", prop.key, prop.value, prop.value.type_name());
}
```

## Migración desde Frontmatter

Si tienes notas con frontmatter YAML, puedes convertirlas manualmente a propiedades inline. El frontmatter seguirá funcionando para compatibilidad, pero las propiedades inline son el método preferido.
