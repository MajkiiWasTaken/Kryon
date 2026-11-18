use gpui::{IntoElement, div, prelude::*, px, rgb};

const BACKGROUND: u32 = 0x14161B;
const PANEL: u32 = 0x191C22;
const BORDER: u32 = 0x30343D;
const TEXT: u32 = 0xD6DAE1;
const MUTED: u32 = 0x858D9B;
const ACCENT: u32 = 0x83A9E0;

pub fn title_bar() -> impl IntoElement {
    div()
        .h(px(44.0))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .bg(rgb(PANEL))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(ACCENT))
                        .child("KRYON"),
                )
                .child(
                    div()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(MUTED))
                        .child("IDE"),
                ),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(MUTED))
                .child("Kryon Workspace"),
        )
}

pub fn explorer() -> impl IntoElement {
    div()
        .w(px(240.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(rgb(PANEL))
        .border_r_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .h(px(42.0))
                .flex()
                .items_center()
                .px_4()
                .text_xs()
                .text_color(rgb(MUTED))
                .child("Explorer"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .px_3()
                .child(file_item("▾  kryon", 0))
                .child(file_item("▾  src", 1))
                .child(file_item("   main.rs", 2))
                .child(file_item("   ui", 2))
                .child(file_item("   Cargo.toml", 1))
                .child(file_item("   README.md", 1)),
        )
}

fn file_item(name: &'static str, indent: u32) -> impl IntoElement {
    div()
        .h(px(25.0))
        .flex()
        .items_center()
        .pl(px((indent * 14) as f32))
        .text_sm()
        .text_color(rgb(TEXT))
        .child(name)
}

pub fn tab_bar() -> impl IntoElement {
    div()
        .h(px(38.0))
        .flex()
        .items_center()
        .bg(rgb(PANEL))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .h_full()
                .flex()
                .items_center()
                .gap_3()
                .px_4()
                .bg(rgb(BACKGROUND))
                .border_r_1()
                .border_color(rgb(BORDER))
                .child(div().text_sm().text_color(rgb(TEXT)).child("main.rs"))
                .child(div().text_color(rgb(MUTED)).child("×")),
        )
}

pub fn editor() -> impl IntoElement {
    let lines = [
        "fn main() {",
        "    println!(\"Hello, Kryon!\");",
        "}",
        "",
        "// Built for Flow.",
    ];

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(rgb(BACKGROUND))
        .p_4()
        .gap_1()
        .children(lines.into_iter().enumerate().map(|(index, line)| {
            div()
                .flex()
                .gap_4()
                .text_sm()
                .child(
                    div()
                        .w(px(32.0))
                        .text_color(rgb(MUTED))
                        .child(format!("{}", index + 1)),
                )
                .child(div().text_color(rgb(TEXT)).child(line.to_string()))
        }))
}

pub fn status_bar() -> impl IntoElement {
    div()
        .h(px(28.0))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .px_3()
        .bg(rgb(PANEL))
        .border_t_1()
        .border_color(rgb(BORDER))
        .text_xs()
        .text_color(rgb(MUTED))
        .child(div().child("Kryon IDE  |  Ready"))
        .child(
            div()
                .flex()
                .gap_4()
                .child("Ln 1, Col 1")
                .child("UTF-8")
                .child("Rust"),
        )
}
