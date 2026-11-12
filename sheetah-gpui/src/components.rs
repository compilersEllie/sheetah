use crate::AppState;
use gpui::{div, prelude::*, rgb, flex_col, items_center, justify_center};

pub fn sidebar_left(_app: &AppState) -> impl IntoElement {
    div()
        .w_1_4()
        .h_full()
        .flex()
        .flex_col()
        .bg(rgb(0x181825))
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .child(div().px_4().py_2().text_sm().text_color(rgb(0x6c7086)).child("BACKENDS"))
                .child(div().px_4().py_1().text_sm().child("No backends loaded"))
        )
        .child(
            div()
                .h_1_3()
                .bg(rgb(0x11111b))
                .flex()
                .flex_col()
                .child(div().px_4().py_2().text_xs().text_color(rgb(0x6c7086)).child("TASKS"))
                .child(div().px_4().py_1().text_sm().child("No tasks"))
        )
}

pub fn center_panes(_app: &AppState) -> impl IntoElement {
    div()
        .w_2_4()
        .h_full()
        .bg(rgb(0x24273a))
        .child(div().size_full().justify_center().items_center().child("Main Panes"))
}

pub fn sidebar_right(_app: &AppState) -> impl IntoElement {
    div()
        .w_1_4()
        .h_full()
        .bg(rgb(0x181825))
        .child(div().size_full().px_4().py_2().child("Today's Note"))
}
