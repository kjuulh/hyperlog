use crate::components::movement_graph::{GraphItemType, MovementGraph, MovementGraphItem};
use itertools::Itertools;
use ratatui::prelude::*;

const GREEN: Color = Color::Rgb(127, 255, 0);
const ORANGE: Color = Color::Rgb(255, 165, 0);
const PURPLE: Color = Color::Rgb(138, 43, 226);

pub trait Summarize {
    fn heading(&self) -> Vec<Span>;
    fn brief(&self) -> Vec<Vec<Span>>;
    fn full(&self, selected: bool) -> Vec<Vec<Span>>;
}

impl Summarize for MovementGraphItem {
    fn heading(&self) -> Vec<Span> {
        let name = Span::from(self.name.clone());

        match self.item_type {
            GraphItemType::Section => {
                let items = self.values.items.len();

                vec![
                    name,
                    Span::from(" ~ ").fg(PURPLE),
                    Span::from(format!("(items: {})", items)).fg(Color::DarkGray),
                ]
            }
            GraphItemType::Item { done } => {
                if done {
                    vec![
                        Span::from("[").fg(Color::DarkGray),
                        Span::from("x").fg(GREEN),
                        Span::from("] ").fg(Color::DarkGray),
                        name,
                    ]
                } else {
                    vec![Span::from("[ ] ").fg(Color::DarkGray), name]
                }
            }
        }
    }

    fn brief(&self) -> Vec<Vec<Span>> {
        let heading = self.heading();
        let mut output = vec![heading];

        let items = &self.values.items;

        let items = if items.len() > 2 {
            vec![
                items.first().unwrap().heading(),
                vec![Span::from("...").fg(Color::DarkGray)],
                items.last().unwrap().heading(),
                vec![Span::raw("")],
            ]
        } else {
            items.iter().map(|i| i.heading()).collect()
        };

        for mut item in items {
            item.insert(0, Span::from(" ".repeat(4)));
            output.push(item);
        }

        output
    }

    fn full(&self, selected: bool) -> Vec<Vec<Span>> {
        let heading = self
            .heading()
            .into_iter()
            .map(|h| {
                if selected {
                    h.patch_style(Style::new().fg(ORANGE))
                } else {
                    h
                }
            })
            .collect_vec();

        let items = &self.values.items;

        let mut output = vec![heading];

        for item in items {
            for mut brief in item.brief() {
                brief.insert(0, Span::from(" ".repeat(4)));
                output.push(brief);
            }
        }

        if !items.is_empty() {
            output.push(vec![Span::raw("")]);
        }

        output
    }
}
pub trait SummarizeRenderGraph {
    fn render_graph(&self, items: &[usize]) -> Vec<ratatui::prelude::Line>;
    fn render_graph_spans(&self, items: &[usize]) -> Vec<Vec<ratatui::prelude::Span>>;
}

impl SummarizeRenderGraph for MovementGraph {
    fn render_graph(&self, items: &[usize]) -> Vec<ratatui::prelude::Line> {
        self.render_graph_spans(items).to_lines()
    }

    fn render_graph_spans(&self, items: &[usize]) -> Vec<Vec<ratatui::prelude::Span>> {
        match items.split_first() {
            Some((first, rest)) => match self.items.get(*first) {
                Some(item) => {
                    let mut output = Vec::new();

                    if rest.is_empty() {
                        for item in 0..*first {
                            if let Some(sibling) = self.items.get(item) {
                                output.append(&mut sibling.brief());
                            }
                        }

                        output.append(&mut item.full(true));

                        for item in *first + 1..self.items.len() {
                            if let Some(sibling) = self.items.get(item) {
                                output.append(&mut sibling.brief());
                            }
                        }
                    } else {
                        let heading = item.heading();
                        output.push(heading);

                        let mut next_level = item.values.render_graph_spans(rest);
                        for item in next_level.iter_mut() {
                            item.insert(0, Span::raw(" ".repeat(4)));
                        }
                        output.append(&mut next_level);
                    }

                    output
                }
                None => {
                    let mut output = Vec::new();
                    for item in &self.items {
                        output.append(&mut item.brief());
                    }
                    output
                }
            },
            None => {
                let mut output = Vec::new();
                for item in &self.items {
                    output.append(&mut item.brief());
                }
                output
            }
        }
    }
}

pub trait ToLine<'a> {
    fn to_lines(&self) -> Vec<Line<'a>>;
}

impl<'a> ToLine<'a> for Vec<Vec<Span<'a>>> {
    fn to_lines(&self) -> Vec<Line<'a>> {
        self.iter()
            .map(|i| Line::from(i.to_vec()))
            .collect::<Vec<_>>()
    }
}
