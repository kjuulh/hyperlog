use ratatui::prelude::*;

pub trait RenderGraph {
    fn render_graph(&self, items: &[usize]) -> Vec<Line>;
    fn render_graph_spans(&self, items: &[usize]) -> Vec<Vec<Span>>;
}

pub mod classic;
