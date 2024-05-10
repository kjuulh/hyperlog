use ratatui::prelude::*;

use crate::components::movement_graph::{GraphItemType, MovementGraph};

use super::RenderGraph;

impl RenderGraph for MovementGraph {
    /// render_graph takes each level of items, renders them, and finally renders a strongly set selector for the current item the user is on
    /// This is done from buttom up, and composed via. string padding
    fn render_graph(&self, items: &[usize]) -> Vec<Line> {
        // Gets the inner content of the strings

        let mut lines = Vec::new();

        for item in &self.items {
            let prefix = match item.item_type {
                GraphItemType::Section => "- ",
                GraphItemType::Item { done } => {
                    if done {
                        "- [x]"
                    } else {
                        "- [ ]"
                    }
                }
            };

            match items.split_first().map(|(first, rest)| {
                if item.index == *first {
                    (true, rest)
                } else {
                    (false, rest)
                }
            }) {
                Some((true, rest)) => {
                    if rest.is_empty() {
                        lines.push(
                            Line::raw(format!("{} {}", prefix, item.name))
                                .style(Style::new().bold().white()),
                        );
                    } else {
                        lines.push(
                            Line::raw(format!("{} {}", prefix, item.name))
                                .patch_style(Style::new().dark_gray()),
                        );
                    }

                    lines.push("".into());

                    let embedded_sections = item.values.render_graph_spans(rest);
                    for section in &embedded_sections {
                        let mut line = vec![Span::raw(" ".repeat(4))];
                        line.extend_from_slice(section);
                        lines.push(Line::from(line));
                    }
                }
                _ => {
                    lines.push(
                        Line::raw(format!("{} {}", prefix, item.name))
                            .patch_style(Style::new().dark_gray()),
                    );

                    lines.push("".into());

                    let embedded_sections = item.values.render_graph_spans(&[]);
                    for section in &embedded_sections {
                        let mut line = vec![Span::raw(" ".repeat(4))];
                        line.extend_from_slice(section);
                        lines.push(Line::from(line));
                    }
                }
            }
        }

        lines
    }

    fn render_graph_spans(&self, items: &[usize]) -> Vec<Vec<Span>> {
        let mut lines = Vec::new();

        for item in &self.items {
            let prefix = match item.item_type {
                GraphItemType::Section => "-",
                GraphItemType::Item { done } => {
                    if done {
                        "- [x]"
                    } else {
                        "- [ ]"
                    }
                }
            };
            match items.split_first().map(|(first, rest)| {
                if item.index == *first {
                    (true, rest)
                } else {
                    (false, rest)
                }
            }) {
                Some((true, rest)) => {
                    let mut line = Vec::new();
                    if rest.is_empty() {
                        line.push(
                            Span::raw(format!("{} {}", prefix, item.name))
                                .style(Style::new().bold().white()),
                        );
                    } else {
                        line.push(
                            Span::raw(format!("{} {}", prefix, item.name))
                                .patch_style(Style::new().dark_gray()),
                        );
                    }

                    lines.push(line);
                    lines.push(vec!["".into()]);

                    let embedded_sections = item.values.render_graph_spans(rest);
                    for section in &embedded_sections {
                        let mut line = vec![Span::raw(" ".repeat(4))];
                        line.extend_from_slice(section);
                        lines.push(line);
                    }
                }
                _ => {
                    lines.push(vec![Span::raw(format!("{prefix} {}", item.name))
                        .patch_style(Style::new().dark_gray())]);

                    lines.push(vec!["".into()]);

                    let embedded_sections = item.values.render_graph_spans(&[]);
                    for section in &embedded_sections {
                        let mut line = vec![Span::raw(" ".repeat(4))];
                        line.extend_from_slice(section);
                        lines.push(line);
                    }
                }
            }
        }

        lines
    }
}
