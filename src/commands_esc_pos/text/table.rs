use crate::models::print_sections::Table;

/// Valida y procesa sección Table del modelo de impresión
pub fn process_section(table: &Table, chars_per_line: i32) -> Result<Vec<u8>, String> {
    let num_columns = table.columns as usize;

    if let Some(widths) = &table.column_widths {
        let total: i32 = widths.iter().map(|&w| w as i32).sum();
        if total != chars_per_line {
            return Err(format!(
                "column_widths sum ({}) must equal paper chars_per_line ({})",
                total, chars_per_line
            ));
        }
    }

    if let Some(header) = &table.header {
        if !header.is_empty() && header.len() != num_columns {
            return Err(format!(
                "Table header has {} cells but {} columns declared",
                header.len(),
                num_columns
            ));
        }
    }
    for (row_idx, row) in table.body.iter().enumerate() {
        if row.len() != num_columns {
            return Err(format!(
                "Table row {} has {} cells but {} columns declared",
                row_idx,
                row.len(),
                num_columns
            ));
        }
    }

    process_table(table, chars_per_line, table.truncate)
}

pub fn process_table(table: &Table, max_width: i32, truncate: bool) -> Result<Vec<u8>, String> {
    if table.columns == 0 {
        return Ok(Vec::new());
    }

    let num_columns = table.columns as usize;

    // Calcular anchos de columnas
    let column_widths: Vec<i32> = if let Some(widths) = &table.column_widths {
        if widths.len() == num_columns {
            widths.iter().map(|&w| w as i32).collect()
        } else {
            let equal_width = max_width / num_columns as i32;
            vec![equal_width; num_columns]
        }
    } else {
        // Si no se proporcionan anchos, distribuir uniformemente
        let equal_width = max_width / num_columns as i32;
        vec![equal_width; num_columns]
    };

    // Verificar si la suma de anchos excede max_width
    let total_width: i32 = column_widths.iter().sum();

    let mut output = Vec::new();

    if total_width > max_width {
        // Dividir columnas en grupos que quepan en max_width
        let column_groups = split_columns_into_groups(&column_widths, max_width);

        // Procesar header
        if let Some(header) = &table.header {
            if !header.is_empty() {
                for group in &column_groups {
                    let group_cells: Vec<_> = group
                        .iter()
                        .filter_map(|&idx| header.get(idx))
                        .cloned()
                        .collect();
                    let group_widths: Vec<i32> = group
                        .iter()
                        .filter_map(|&idx| column_widths.get(idx).copied())
                        .collect();
                    let row_lines = process_row(&group_cells, &group_widths, truncate);
                    for line in row_lines {
                        output.extend(line.as_bytes());
                        output.extend(b"\n");
                    }
                }
            }
        }

        // Procesar body
        for row in &table.body {
            for group in &column_groups {
                let group_cells: Vec<_> = group
                    .iter()
                    .filter_map(|&idx| row.get(idx))
                    .cloned()
                    .collect();
                let group_widths: Vec<i32> = group
                    .iter()
                    .filter_map(|&idx| column_widths.get(idx).copied())
                    .collect();
                let row_lines = process_row(&group_cells, &group_widths, truncate);
                for line in row_lines {
                    output.extend(line.as_bytes());
                    output.extend(b"\n");
                }
            }
        }
    } else {
        // Procesar normalmente si cabe en max_width
        // Procesar header
        if let Some(header) = &table.header {
            if !header.is_empty() {
                let header_lines = process_row(header, &column_widths, truncate);
                for line in header_lines {
                    output.extend(line.as_bytes());
                    output.extend(b"\n");
                }
            }
        }

        // Procesar body
        for row in &table.body {
            let row_lines = process_row(row, &column_widths, truncate);
            for line in row_lines {
                output.extend(line.as_bytes());
                output.extend(b"\n");
            }
        }
    }

    Ok(output)
}

fn split_columns_into_groups(column_widths: &[i32], max_width: i32) -> Vec<Vec<usize>> {
    let mut groups = Vec::new();
    let mut current_group = Vec::new();
    let mut current_width = 0;

    for (idx, &width) in column_widths.iter().enumerate() {
        if current_width + width <= max_width {
            current_group.push(idx);
            current_width += width;
        } else {
            if !current_group.is_empty() {
                groups.push(current_group);
            }
            current_group = vec![idx];
            current_width = width;
        }
    }

    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}

fn process_row(
    row: &[crate::models::print_sections::Text],
    column_widths: &[i32],
    truncate: bool,
) -> Vec<String> {
    // Para cada celda, obtener las líneas
    let mut cell_lines: Vec<Vec<String>> = Vec::new();
    for (i, cell) in row.iter().enumerate() {
        let width = if i < column_widths.len() {
            column_widths[i]
        } else {
            10
        }; // default
        let text = remove_accents(&cell.text);
        let lines = if truncate {
            vec![truncate_text(&text, width as usize)]
        } else {
            wrap_text(&text, width as usize)
        };
        cell_lines.push(lines);
    }

    // Número máximo de líneas
    let max_lines = cell_lines.iter().map(|l| l.len()).max().unwrap_or(1);

    // Generar líneas de output
    let mut result = Vec::new();
    for line_idx in 0..max_lines {
        let mut line = String::new();
        for (i, cell) in cell_lines.iter().enumerate() {
            let part = if line_idx < cell.len() {
                &cell[line_idx]
            } else {
                ""
            };
            let padded = format!(
                "{:<width$}",
                part,
                width = *column_widths.get(i).unwrap_or(&10) as usize
            );
            line.push_str(&padded);
        }
        result.push(line.trim_end().to_string());
    }

    result
}

fn truncate_text(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        text.to_string()
    } else {
        text.chars().take(width).collect()
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        let current_len = current.chars().count();
        if word_len >= width {
            // Palabra muy larga: vaciar current primero, luego truncar
            if !current.is_empty() {
                lines.push(current);
                current = String::new();
            }
            lines.push(word.chars().take(width).collect());
        } else if current_len + word_len + 1 > width {
            // No cabe junto a lo que ya hay: saltar de línea
            if !current.is_empty() {
                lines.push(current);
            }
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn remove_accents(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'Á' | 'À' | 'Â' | 'Ä' | 'Ã' => 'A',
            'É' | 'È' | 'Ê' | 'Ë' => 'E',
            'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
            'Ó' | 'Ò' | 'Ô' | 'Ö' | 'Õ' => 'O',
            'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
            'ñ' => 'n',
            'Ñ' => 'N',
            _ => c,
        })
        .collect()
}
