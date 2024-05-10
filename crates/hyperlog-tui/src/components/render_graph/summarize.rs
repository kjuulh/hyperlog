use crate::components::movement_graph::{GraphItemType, MovementGraph, MovementGraphItem};
use itertools::Itertools;
use ratatui::prelude::*;

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
                    Span::from(" ~ "),
                    Span::from(format!("(items: {})", items)),
                ]
            }
            GraphItemType::Item { done } => {
                vec![Span::from(if done { "- [x] " } else { "- [ ] " }), name]
            }
        }
    }

    fn brief(&self) -> Vec<Vec<Span>> {
        let heading = self.heading();

        let items = &self.values.items;

        let items = if items.len() > 2 {
            vec![items.first().unwrap(), items.last().unwrap()]
        } else {
            items.iter().collect()
        };

        let mut output = vec![heading];

        for item in items {
            let mut heading = item.heading();
            heading.insert(0, Span::from(" ".repeat(4)));
            output.push(heading);
        }

        output
    }

    fn full(&self, selected: bool) -> Vec<Vec<Span>> {
        let heading = self
            .heading()
            .into_iter()
            .map(|h| if selected { h.bold() } else { h })
            .collect_vec();

        let items = &self.values.items;

        let mut output = vec![heading];

        for item in items {
            for mut brief in item.brief() {
                brief.insert(0, Span::from(" ".repeat(4)));
                output.push(brief);
            }
        }

        output
    }
}
pub trait SummarizeRenderGraph {
    fn render_graph(&self, items: &[usize]) -> Vec<ratatui::prelude::Line>;
    fn render_graph_spans(&self, items: &[usize], depth: usize)
        -> Vec<Vec<ratatui::prelude::Span>>;
}

impl SummarizeRenderGraph for MovementGraph {
    fn render_graph(&self, items: &[usize]) -> Vec<ratatui::prelude::Line> {
        self.render_graph_spans(items, 0).to_lines()
    }

    fn render_graph_spans(
        &self,
        items: &[usize],
        depth: usize,
    ) -> Vec<Vec<ratatui::prelude::Span>> {
        match items.split_first() {
            Some((first, rest)) => match rest.is_empty() {
                true => match self.items.get(*first) {
                    Some(item) => {
                        let mut output = Vec::new();

                        if *first > 0 {
                            if let Some(sibling) = self.items.get(*first - 1) {
                                output.append(&mut sibling.brief())
                            }
                        }

                        output.append(&mut item.full(true));

                        if *first < self.items.len() {
                            if let Some(sibling) = self.items.get(*first + 1) {
                                output.append(&mut sibling.brief())
                            }
                        }

                        output
                    }
                    None => vec![],
                },
                false => {
                    if rest.len() > 1 {
                        let mut output = Vec::new();
                        // TODO: add heading for the current item, and shift lines by one
                        if let Some(item) = self.items.get(*first) {
                            output.push(item.heading())
                        }
                        for mut line in self.render_graph_spans(rest, 0) {
                            line.insert(0, Span::from("~".repeat(4)));

                            output.push(line);
                        }

                        output
                    } else {
                        match self.items.get(*first) {
                            Some(item) => match item.values.items.get(*rest.first().unwrap()) {
                                Some(actual_item) => {
                                    let mut output = Vec::new();

                                    if *first > 0 {
                                        if let Some(sibling) = self.items.get(*first - 1) {
                                            output.append(&mut sibling.brief())
                                        }
                                    }

                                    output.append(&mut actual_item.full(true));

                                    if *first < self.items.len() {
                                        if let Some(sibling) = self.items.get(*first + 1) {
                                            output.append(&mut sibling.brief())
                                        }
                                    }

                                    output
                                }
                                None => vec![],
                            },
                            None => vec![],
                        }
                    }
                }
            },
            None => self.items.iter().flat_map(|i| i.brief()).collect_vec(),
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
