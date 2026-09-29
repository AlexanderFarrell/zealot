//! Minimal aligned-column tables. Degrades to plain aligned text when piped.

use super::{bold, dim, visible_width};

pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(headers: &[&str]) -> Self {
        Self {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            rows: Vec::new(),
        }
    }

    pub fn row(&mut self, cells: Vec<String>) {
        self.rows.push(cells);
    }

    pub fn print(&self) {
        let cols = self.headers.len();
        let mut widths: Vec<usize> = self.headers.iter().map(|h| visible_width(h)).collect();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate().take(cols) {
                widths[i] = widths[i].max(visible_width(cell));
            }
        }

        let mut header_line = String::new();
        for (i, h) in self.headers.iter().enumerate() {
            header_line.push_str(&pad(&bold(h), widths[i]));
            if i + 1 < cols {
                header_line.push_str("  ");
            }
        }
        println!("{}", header_line.trim_end());
        println!("{}", dim(&"─".repeat(widths.iter().sum::<usize>() + 2 * (cols - 1))));

        for row in &self.rows {
            let mut line = String::new();
            for (i, cell) in row.iter().enumerate().take(cols) {
                line.push_str(&pad(cell, widths[i]));
                if i + 1 < cols {
                    line.push_str("  ");
                }
            }
            println!("{}", line.trim_end());
        }
    }
}

fn pad(s: &str, width: usize) -> String {
    let current = visible_width(s);
    let mut out = s.to_string();
    for _ in current..width {
        out.push(' ');
    }
    out
}
