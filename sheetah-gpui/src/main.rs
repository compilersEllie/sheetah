#![cfg_attr(target_family = "wasm", no_main)]
//! Example demonstrating GPUI's testing infrastructure.
//!
//! When run normally, this displays an interactive counter window.
//! The tests below demonstrate various GPUI testing patterns.
//!
//! Run the app: cargo run -p gpui --example testing
//! Run tests:   cargo test -p gpui --example testing --features test-support

#[cfg(test)]
mod tests;

mod fonts;

use gpui::{
    actions, div, prelude::*, px, rgb, size, App, Bounds, Context, FocusHandle, Focusable, Render,
    Task, Window, WindowBounds, WindowOptions,
};
use gpui_platform::application;

actions!(counter, [Increment, Decrement]);

struct Counter {
    count: i32,
    focus_handle: FocusHandle,
    _subscription: gpui::Subscription,
}

/// Event emitted by Counter
struct CounterEvent;

impl gpui::EventEmitter<CounterEvent> for Counter {}

impl Counter {
    fn new(cx: &mut Context<Self>) -> Self {
        let subscription = cx.subscribe_self(|this: &mut Self, _event: &CounterEvent, _cx| {
            this.count = 999;
        });

        Self {
            count: 0,
            focus_handle: cx.focus_handle(),
            _subscription: subscription,
        }
    }

    fn increment(&mut self, _: &Increment, _window: &mut Window, cx: &mut Context<Self>) {
        self.count += 1;
        cx.notify();
    }

    fn decrement(&mut self, _: &Decrement, _window: &mut Window, cx: &mut Context<Self>) {
        self.count -= 1;
        cx.notify();
    }

    fn load(&self, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            // Simulate loading data (e.g., from disk or network)
            this.update(cx, |counter, cx| {
                counter.count = 100;
                cx.notify();
            })
            .ok();
        })
    }

    fn reload(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            // Simulate reloading data in the background
            this.update(cx, |counter, cx| {
                counter.count += 50;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

impl Focusable for Counter {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("counter")
            .key_context("Counter")
            .on_action(cx.listener(Self::increment))
            .on_action(cx.listener(Self::decrement))
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .gap_4()
            .bg(rgb(0x1e1e2e))
            .size_full()
            .justify_center()
            .items_center()
            .child(
                div()
                    .text_3xl()
                    .text_color(rgb(0xcdd6f4))
                    .child(format!("{}", self.count)),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .id("decrement")
                            .px_4()
                            .py_2()
                            .bg(rgb(0x313244))
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .rounded_md()
                            .cursor_pointer()
                            .text_color(rgb(0xcdd6f4))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.decrement(&Decrement, window, cx)
                            }))
                            .child("−"),
                    )
                    .child(
                        div()
                            .id("increment")
                            .px_4()
                            .py_2()
                            .bg(rgb(0x313244))
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .rounded_md()
                            .cursor_pointer()
                            .text_color(rgb(0xcdd6f4))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.increment(&Increment, window, cx)
                            }))
                            .child("+"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .id("load")
                            .px_4()
                            .py_2()
                            .bg(rgb(0x313244))
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .rounded_md()
                            .cursor_pointer()
                            .text_color(rgb(0xcdd6f4))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.load(cx).detach();
                            }))
                            .child("Load"),
                    )
                    .child(
                        div()
                            .id("reload")
                            .px_4()
                            .py_2()
                            .bg(rgb(0x313244))
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .rounded_md()
                            .cursor_pointer()
                            .text_color(rgb(0xcdd6f4))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.reload(cx);
                            }))
                            .child("Reload"),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x6c7086))
                    .child("Press ↑/↓ or click buttons"),
            )
    }
}

fn run_example() {
    application().run(|cx| {
        if !fonts::load_fonts(&*cx) {
            return;
        }
        cx.bind_keys([
            gpui::KeyBinding::new("up", Increment, Some("Counter")),
            gpui::KeyBinding::new("down", Decrement, Some("Counter")),
        ]);

        let bounds = Bounds::centered(None, size(px(300.), px(200.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let counter = cx.new(Counter::new);
                counter.focus_handle(cx).focus(window, cx);
                counter
            },
        )
        .unwrap();
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}

#[derive(Clone)]
pub struct GridView {
    pub tags: Vec<String>,
}
impl Render for GridView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

impl GridView {
    pub fn new() -> Self {
        Self { tags: vec![] }
    }

    pub fn search(&self, query: &str) -> Vec<String> {
        let lower_query = query.to_lowercase();
        self.tags
            .iter()
            .filter(|tag| tag.to_lowercase().contains(&lower_query))
            .cloned()
            .collect()
    }
}

impl Default for GridView {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Block {
    Text(String),
    Chart(String),
}

#[derive(Clone)]
pub struct MarkdownEditor {
    pub content: String,
}
impl MarkdownEditor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }
    pub fn set_content(&mut self, content: &str, _cx: &mut impl gpui::AppContext) {
        self.content = content.to_string();
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn get_blocks(&self) -> Vec<Block> {
        let mut blocks = Vec::new();
        let mut current_pos = 0;
        let pattern_start = "```chart ";
        let pattern_end = "```";

        while let Some(start_idx) = self.content[current_pos..].find(pattern_start) {
            let absolute_start = current_pos + start_idx;

            // Add text before the chart block
            if absolute_start > current_pos {
                blocks.push(Block::Text(
                    self.content[current_pos..absolute_start].to_string(),
                ));
            }

            let chart_start = absolute_start + pattern_start.len();
            if let Some(end_idx) = self.content[chart_start..].find(pattern_end) {
                let absolute_end = chart_start + end_idx;
                blocks.push(Block::Chart(
                    self.content[chart_start..absolute_end].to_string(),
                ));
                current_pos = absolute_end + pattern_end.len();
            } else {
                // Malformed chart block, treat as text and break
                break;
            }
        }

        // Add remaining text
        if current_pos < self.content.len() {
            blocks.push(Block::Text(self.content[current_pos..].to_string()));
        }

        // If no blocks were found, it's just one text block
        if blocks.is_empty() && !self.content.is_empty() {
            blocks.push(Block::Text(self.content.clone()));
        }

        blocks
    }
}

impl Default for MarkdownEditor {
    fn default() -> Self {
        Self::new()
    }
}
impl Render for MarkdownEditor {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Clone)]
pub struct DSLEditor;
impl DSLEditor {
    pub fn set_content(&mut self, _content: &str, _cx: &mut impl gpui::AppContext) {}
}
impl Render for DSLEditor {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Clone)]
pub struct ChartPreview;
impl Render for ChartPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Clone)]
pub struct ViewConfig;
impl ViewConfig {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ViewConfig {
    fn default() -> Self {
        Self::new()
    }
}
impl Render for ViewConfig {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
