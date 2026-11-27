#![cfg_attr(target_family = "wasm", no_main)]

mod fonts;

use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, ClickEvent, Context, Decorations, Div,
    FocusHandle, Focusable, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Render, Stateful, TitlebarOptions, Window, WindowBounds, WindowButton,
    WindowButtonLayout, WindowControlArea, WindowControls, WindowDecorations, WindowOptions,
    MAX_BUTTONS_PER_SIDE,
};
use gpui_platform::application;
use sheetah_model::backend::{Backend, BackendOptions};
use sheetah_model::backends::sqlite::SqliteBackend;
use sheetah_model::editor::{Index, Pos};
use sheetah_model::error::{Result, SheetahError};
use sheetah_model::table::{Table, TableId, TableName};
use sheetah_model::value::{Value, Values, Values2D};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

struct Spreadsheet {
    backend: Arc<RwLock<SqliteBackend>>,
    table_id: Option<TableId>,
    cells: Vec<Vec<String>>,
    selected: (u32, u32),
    draft: String,
    editing: bool,
    status: String,
    next_field_id: u32,
    titlebar_drag_armed: bool,
    focus_handle: FocusHandle,
}

impl Spreadsheet {
    fn new(cx: &mut Context<Self>) -> Self {
        let backend = Arc::new(RwLock::new(SqliteBackend::new(BackendOptions {
            name: "local".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("sheetah.sqlite"),
        })));
        let startup_backend = backend.clone();

        cx.spawn(async move |this, cx| {
            let result = async {
                let mut guard = startup_backend.write().await;
                guard.setup().await?;
                let id = match guard.table_ids().first().copied() {
                    Some(id) => id,
                    None => {
                        let table = seed_table()?;
                        let id = table.id();
                        guard.insert(id, table).await?;
                        guard.flush().await?;
                        id
                    }
                };
                let cells = guard.snapshot(&id).await?;
                let table = guard.get(&id).await?;
                let next_field_id = table
                    .shape
                    .columns
                    .order()
                    .iter()
                    .chain(table.shape.rows.order())
                    .copied()
                    .max()
                    .unwrap_or(0)
                    .saturating_add(1);
                Ok::<_, SheetahError>((id, cells, next_field_id))
            }
            .await;

            this.update(cx, |this, cx| {
                match result {
                    Ok((id, cells, next_field_id)) => {
                        this.table_id = Some(id);
                        this.cells = cells;
                        this.next_field_id = next_field_id;
                        this.status = "ready".to_string();
                    }
                    Err(error) => this.status = format!("error: {error}"),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();

        Self {
            backend,
            table_id: None,
            cells: vec![vec![String::new(); 4]; 4],
            selected: (0, 0),
            draft: String::new(),
            editing: false,
            status: "loading".to_string(),
            next_field_id: 100,
            titlebar_drag_armed: false,
            focus_handle: cx.focus_handle(),
        }
    }

    fn current_cell(&self) -> &str {
        self.cells
            .get(self.selected.1 as usize)
            .and_then(|row| row.get(self.selected.0 as usize))
            .map(String::as_str)
            .unwrap_or_default()
    }

    fn handle_key(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let modifiers = event.keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
            return;
        }
        if self.table_id.is_none() {
            return;
        }
        match event.keystroke.key.as_str() {
            "enter" => {
                if self.editing {
                    self.commit_edit(cx);
                    return;
                }
                self.editing = true;
                self.draft = self.current_cell().to_string();
            }
            "escape" => {
                self.editing = false;
                self.draft.clear();
            }
            "backspace" if self.editing => {
                self.draft.pop();
            }
            "up" => self.move_selection(0, -1),
            "down" => self.move_selection(0, 1),
            "left" => self.move_selection(-1, 0),
            "right" => self.move_selection(1, 0),
            key if self.editing => {
                let character = event.keystroke.key_char.as_deref().unwrap_or(key);
                if character.chars().count() == 1 {
                    self.draft.push_str(character);
                }
            }
            _ => {}
        }
        cx.notify();
    }

    fn move_selection(&mut self, column_delta: i32, row_delta: i32) {
        let columns = self.cells.first().map_or(0, Vec::len) as i32;
        let rows = self.cells.len() as i32;
        let column = (self.selected.0 as i32 + column_delta).clamp(0, columns.saturating_sub(1));
        let row = (self.selected.1 as i32 + row_delta).clamp(0, rows.saturating_sub(1));
        self.selected = (column as u32, row as u32);
        self.editing = false;
        self.draft.clear();
    }

    fn commit_edit(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.table_id else {
            return;
        };
        let (column, row) = self.selected;
        let text = self.draft.clone();
        let backend = self.backend.clone();

        cx.spawn(async move |this, cx| {
            let result = async {
                let value = parse_value(&text);
                let display = value.to_text();
                let mut guard = backend.write().await;
                guard
                    .set_data(
                        id,
                        &Pos {
                            column: Index::new(column),
                            row: Index::new(row),
                        },
                        Values2D::new(Index::new(1), Index::new(1), vec![value])?,
                    )
                    .await?;
                guard.flush().await?;
                Ok::<_, SheetahError>(display)
            }
            .await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(display) => {
                        if let Some(cell) = this
                            .cells
                            .get_mut(row as usize)
                            .and_then(|cells| cells.get_mut(column as usize))
                        {
                            *cell = display;
                        }
                        this.status = "saved".to_string();
                    }
                    Err(error) => this.status = format!("error: {error}"),
                }
                this.editing = false;
                this.draft.clear();
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn add_row(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.table_id else {
            return;
        };
        let columns = self.cells.first().map_or(0, Vec::len) as u32;
        let row_index = self.cells.len() as u32;
        let field_id = self.take_field_id();
        let row: Values = (0..columns)
            .map(|_| Box::new(String::new()) as Box<dyn Value>)
            .collect();
        let backend = self.backend.clone();

        cx.spawn(async move |this, cx| {
            let result = async {
                let mut guard = backend.write().await;
                guard
                    .add_row(id, field_id, Index::new(row_index), row)
                    .await?;
                guard.flush().await?;
                Ok::<_, SheetahError>(())
            }
            .await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(()) => {
                        this.cells.push(vec![String::new(); columns as usize]);
                        this.status = "saved".to_string();
                    }
                    Err(error) => this.status = format!("error: {error}"),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn add_column(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.table_id else {
            return;
        };
        let rows = self.cells.len() as u32;
        let column_index = self.cells.first().map_or(0, Vec::len) as u32;
        let field_id = self.take_field_id();
        let column: Values = (0..rows)
            .map(|_| Box::new(String::new()) as Box<dyn Value>)
            .collect();
        let backend = self.backend.clone();

        cx.spawn(async move |this, cx| {
            let result = async {
                let mut guard = backend.write().await;
                guard
                    .add_column(id, field_id, Index::new(column_index), column)
                    .await?;
                guard.flush().await?;
                Ok::<_, SheetahError>(())
            }
            .await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(()) => {
                        for row in &mut this.cells {
                            row.push(String::new());
                        }
                        this.status = "saved".to_string();
                    }
                    Err(error) => this.status = format!("error: {error}"),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn save(&mut self, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let backend = self.backend.clone();
        cx.spawn(async move |this, cx| {
            let result = async {
                backend.write().await.flush().await?;
                Ok::<_, SheetahError>(())
            }
            .await;

            this.update(cx, |this, cx| {
                this.status = match result {
                    Ok(()) => "saved".to_string(),
                    Err(error) => format!("error: {error}"),
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn take_field_id(&mut self) -> u32 {
        let id = self.next_field_id;
        self.next_field_id += 1;
        id
    }
}

impl Focusable for Spreadsheet {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Spreadsheet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let columns = self.cells.first().map_or(0, Vec::len);
        let mut cells: Vec<Stateful<Div>> = Vec::new();
        for (row, values) in self.cells.iter().enumerate() {
            for (column, value) in values.iter().enumerate() {
                let selected = self.selected == (column as u32, row as u32);
                let text = if selected && self.editing {
                    self.draft.clone()
                } else {
                    value.clone()
                };
                let listener = cx.listener(move |this: &mut Self, _, _, cx| {
                    this.selected = (column as u32, row as u32);
                    this.editing = false;
                    this.draft.clear();
                    cx.notify();
                });

                let mut cell = div()
                    .id(format!("cell-{column}-{row}"))
                    .on_click(listener)
                    .min_w(px(96.))
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(0x313244))
                    .child(text);
                if selected {
                    cell = cell.bg(rgb(0x585b70));
                }
                cells.push(cell);
            }
        }

        let mut root = div()
            .id("spreadsheet")
            .key_context("Spreadsheet")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key))
            .flex()
            .flex_col()
            .size_full()
            .text_color(rgb(0xcdd6f4));
        if uses_custom_titlebar(window) {
            root = root.child(titlebar(window, cx));
        }
        root.bg(rgb(0x1e1e2e))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .p_4()
                    .pb_0()
                    .child(button("Add row", cx.listener(Self::add_row)))
                    .child(button("Add column", cx.listener(Self::add_column)))
                    .child(button("Save", cx.listener(Self::save))),
            )
            .child(div().px_4().py_1().text_sm().child(self.status.clone()))
            .child(
                div()
                    .flex_1()
                    .grid()
                    .grid_cols(columns.max(1) as u16)
                    .gap_1()
                    .px_4()
                    .pb_4()
                    .children(cells),
            )
    }
}

fn button(
    label: &'static str,
    listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(label)
        .px_3()
        .py_1()
        .rounded_md()
        .bg(rgb(0x313244))
        .hover(|style| style.bg(rgb(0x45475a)))
        .cursor_pointer()
        .on_click(listener)
        .child(label)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TitlebarDoubleClick {
    Native,
    PlatformTitlebar,
    Zoom,
}

fn titlebar_double_click_action() -> TitlebarDoubleClick {
    if cfg!(target_os = "windows") {
        TitlebarDoubleClick::Native
    } else if cfg!(target_os = "macos") {
        TitlebarDoubleClick::PlatformTitlebar
    } else {
        TitlebarDoubleClick::Zoom
    }
}

fn uses_explicit_titlebar_actions() -> bool {
    titlebar_double_click_action() != TitlebarDoubleClick::Native
}

fn uses_custom_titlebar(window: &Window) -> bool {
    cfg!(any(target_os = "macos", target_os = "windows"))
        || matches!(window.window_decorations(), Decorations::Client { .. })
}

fn default_button_layout() -> WindowButtonLayout {
    if cfg!(target_os = "macos") {
        WindowButtonLayout {
            left: [
                Some(WindowButton::Close),
                Some(WindowButton::Minimize),
                Some(WindowButton::Maximize),
            ],
            right: [None; MAX_BUTTONS_PER_SIDE],
        }
    } else {
        WindowButtonLayout {
            left: [None; MAX_BUTTONS_PER_SIDE],
            right: [
                Some(WindowButton::Minimize),
                Some(WindowButton::Maximize),
                Some(WindowButton::Close),
            ],
        }
    }
}

fn window_is_expanded(window: &Window) -> bool {
    window.is_maximized()
        || matches!(
            window.window_decorations(),
            Decorations::Client { tiling } if tiling.is_tiled()
        )
}

fn titlebar(window: &mut Window, cx: &mut Context<Spreadsheet>) -> impl IntoElement {
    let controls = window.window_controls();
    let layout = cx.button_layout().unwrap_or_else(default_button_layout);
    let resizable = window.is_resizable();
    let minimizable = window.is_minimizable();
    let expanded = window_is_expanded(window);

    let mut drag_area = div()
        .id("titlebar-drag")
        .window_control_area(WindowControlArea::Drag)
        .flex_1()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .child("Sheetah");

    if uses_explicit_titlebar_actions() {
        drag_area = drag_area
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this: &mut Spreadsheet, _event: &MouseDownEvent, _, _| {
                    this.titlebar_drag_armed = true;
                }),
            )
            .on_mouse_move(cx.listener(
                |this: &mut Spreadsheet, _event: &MouseMoveEvent, window: &mut Window, _| {
                    if this.titlebar_drag_armed {
                        this.titlebar_drag_armed = false;
                        window.start_window_move();
                    }
                },
            ))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this: &mut Spreadsheet, _event: &MouseUpEvent, _, _| {
                    this.titlebar_drag_armed = false;
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this: &mut Spreadsheet, _event: &MouseUpEvent, _, _| {
                    this.titlebar_drag_armed = false;
                }),
            )
            .on_click(cx.listener(
                |_: &mut Spreadsheet, event: &ClickEvent, window: &mut Window, _| {
                    if event.click_count() != 2 {
                        return;
                    }
                    match titlebar_double_click_action() {
                        TitlebarDoubleClick::PlatformTitlebar => window.titlebar_double_click(),
                        TitlebarDoubleClick::Zoom => window.zoom_window(),
                        TitlebarDoubleClick::Native => {}
                    }
                },
            ));
    }

    div()
        .id("titlebar")
        .h(px(36.))
        .flex()
        .flex_shrink_0()
        .items_center()
        .bg(rgb(0x181825))
        .child(div().flex().h_full().children(titlebar_buttons(
            &layout.left,
            "left",
            controls,
            resizable,
            minimizable,
            expanded,
            cx,
        )))
        .child(drag_area)
        .child(div().flex().h_full().children(titlebar_buttons(
            &layout.right,
            "right",
            controls,
            resizable,
            minimizable,
            expanded,
            cx,
        )))
}

fn titlebar_buttons(
    buttons: &[Option<WindowButton>; MAX_BUTTONS_PER_SIDE],
    side: &str,
    controls: WindowControls,
    resizable: bool,
    minimizable: bool,
    expanded: bool,
    cx: &Context<Spreadsheet>,
) -> Vec<Stateful<Div>> {
    buttons
        .iter()
        .flatten()
        .copied()
        .filter(|button| button_supported(*button, controls, resizable, minimizable))
        .map(|button| {
            let hover_color = if button == WindowButton::Close {
                rgb(0xe81123)
            } else {
                rgb(0x313244)
            };
            let mut element = div()
                .id(format!("titlebar-{side}-{}", button.id()))
                .window_control_area(button_area(button))
                .w(px(46.))
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .hover(move |style| style.bg(hover_color));
            if uses_explicit_titlebar_actions() {
                element =
                    element.on_click(cx.listener(
                        move |_, _, window: &mut Window, _| match button {
                            WindowButton::Minimize => window.minimize_window(),
                            WindowButton::Maximize => window.zoom_window(),
                            WindowButton::Close => window.remove_window(),
                        },
                    ));
            }
            element.child(button_glyph(button, expanded))
        })
        .collect()
}

fn button_supported(
    button: WindowButton,
    controls: WindowControls,
    resizable: bool,
    minimizable: bool,
) -> bool {
    match button {
        WindowButton::Minimize => controls.minimize && minimizable,
        WindowButton::Maximize => controls.maximize && resizable,
        WindowButton::Close => true,
    }
}

fn button_area(button: WindowButton) -> WindowControlArea {
    match button {
        WindowButton::Minimize => WindowControlArea::Min,
        WindowButton::Maximize => WindowControlArea::Max,
        WindowButton::Close => WindowControlArea::Close,
    }
}

fn button_glyph(button: WindowButton, expanded: bool) -> &'static str {
    match button {
        WindowButton::Minimize => "−",
        WindowButton::Maximize if expanded => "❐",
        WindowButton::Maximize => "□",
        WindowButton::Close => "×",
    }
}

fn seed_table() -> Result<Table> {
    let mut table = Table::named(TableName {
        name: Some("Sheet1".to_string()),
        clash: 0,
    });
    for id in 1..=4 {
        table.shape.columns.add(None, id)?;
        table.shape.rows.add(None, id)?;
    }
    Ok(table)
}

fn parse_value(text: &str) -> Box<dyn Value> {
    match text.parse::<i32>() {
        Ok(number) => Box::new(number),
        Err(_) => Box::new(text.to_string()),
    }
}

fn run_app() {
    application().run(|cx| {
        if !fonts::load_fonts(&*cx) {
            return;
        }

        let bounds = Bounds::centered(None, size(px(720.), px(480.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Sheetah".into()),
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                app_owns_titlebar_drag: true,
                window_decorations: Some(WindowDecorations::Client),
                ..Default::default()
            },
            |window, cx| {
                let spreadsheet = cx.new(Spreadsheet::new);
                let entity_id = spreadsheet.entity_id();
                window
                    .observe_button_layout_changed(move |_, cx| cx.notify(entity_id))
                    .detach();
                spreadsheet.focus_handle(cx).focus(window, cx);
                spreadsheet
            },
        )
        .unwrap();
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_app();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_app();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_value_prefers_numbers() {
        assert_eq!(parse_value("42").to_text(), "42");
        assert_eq!(parse_value("hello").to_text(), "hello");
    }

    #[test]
    fn seed_table_has_a_four_by_four_grid() {
        let table = seed_table().unwrap();
        assert_eq!(table.dims().column.raw(), 4);
        assert_eq!(table.dims().row.raw(), 4);
    }

    #[test]
    fn titlebar_buttons_follow_platform_and_window_controls() {
        let all = WindowControls {
            fullscreen: true,
            maximize: true,
            minimize: true,
            window_menu: true,
        };
        assert!(button_supported(WindowButton::Minimize, all, true, true));
        assert!(button_supported(WindowButton::Maximize, all, true, true));
        assert!(button_supported(WindowButton::Close, all, true, true));
        assert!(!button_supported(WindowButton::Minimize, all, true, false));
        assert!(!button_supported(WindowButton::Maximize, all, false, true));

        let none = WindowControls {
            fullscreen: false,
            maximize: false,
            minimize: false,
            window_menu: false,
        };
        assert!(!button_supported(WindowButton::Minimize, none, true, true));
        assert!(!button_supported(WindowButton::Maximize, none, true, true));
        assert!(button_supported(WindowButton::Close, none, true, true));

        assert_eq!(button_area(WindowButton::Minimize), WindowControlArea::Min);
        assert_eq!(button_area(WindowButton::Maximize), WindowControlArea::Max);
        assert_eq!(button_area(WindowButton::Close), WindowControlArea::Close);
        assert_eq!(button_glyph(WindowButton::Minimize, false), "−");
        assert_eq!(button_glyph(WindowButton::Maximize, false), "□");
        assert_eq!(button_glyph(WindowButton::Maximize, true), "❐");
        assert_eq!(button_glyph(WindowButton::Close, true), "×");

        let fallback = default_button_layout();
        let expected = if cfg!(target_os = "macos") {
            WindowButtonLayout {
                left: [
                    Some(WindowButton::Close),
                    Some(WindowButton::Minimize),
                    Some(WindowButton::Maximize),
                ],
                right: [None; MAX_BUTTONS_PER_SIDE],
            }
        } else {
            WindowButtonLayout {
                left: [None; MAX_BUTTONS_PER_SIDE],
                right: [
                    Some(WindowButton::Minimize),
                    Some(WindowButton::Maximize),
                    Some(WindowButton::Close),
                ],
            }
        };
        assert_eq!(fallback, expected);

        let action = titlebar_double_click_action();
        if cfg!(target_os = "windows") {
            assert_eq!(action, TitlebarDoubleClick::Native);
            assert!(!uses_explicit_titlebar_actions());
        } else if cfg!(target_os = "macos") {
            assert_eq!(action, TitlebarDoubleClick::PlatformTitlebar);
            assert!(uses_explicit_titlebar_actions());
        } else {
            assert_eq!(action, TitlebarDoubleClick::Zoom);
            assert!(uses_explicit_titlebar_actions());
        }
    }
}
