#![cfg_attr(target_family = "wasm", no_main)]
//! Example demonstrating GPUI's testing infrastructure.

#[cfg(test)]
mod tests;

mod fonts;
mod model;
mod ui_model;
mod components;

use gpui::{
    actions, div, prelude::*, px, rgb, size, App, Bounds, Context, FocusHandle, Focusable, Render,
    Task as GpuiTask, Window, WindowBounds, WindowOptions,
};
use gpui_platform::application;
use sheetah_model::workspace::Workspace;
use ui_model::{View, UiTask};

actions!(load_backends);

pub struct AppState {
    pub workspace: Workspace,
    pub views: Vec<View>,
    pub tasks: Vec<UiTask>,
    pub backend_names: Vec<String>,
    pub focus_handle: FocusHandle,
}

impl Focusable for AppState {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl AppState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            workspace: Workspace::new(),
            views: vec![],
            tasks: vec![],
            backend_names: vec![],
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn load_backends(&mut self, cx: &mut Context<Self>) {
        let workspace_handle = self.workspace.clone(); // Wait, Workspace is not Clone?
        // Let's check Workspace.
//...

fn run_example() {
    application().run(|cx| {
        if !fonts::load_fonts(&*cx) {
            return;
        }

        let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let app_state = cx.new(AppState::new);
                app_state.focus_handle(cx).focus(window, cx);
                app_state
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
