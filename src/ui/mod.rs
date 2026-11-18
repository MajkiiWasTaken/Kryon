mod components;

use std::{env, path::PathBuf};

use gpui::{
    Context, IntoElement, Render, StatefulInteractiveElement, Window, div, prelude::*, px, rgb,
};

use crate::workspace::{Workspace, read_preview};

const BACKGROUND: u32 = 0x14161B;
const PANEL: u32 = 0x191C22;
const BORDER: u32 = 0x30343D;
const TEXT: u32 = 0xD6DAE1;
const MUTED: u32 = 0x858D9B;
const ACCENT: u32 = 0x83A9E0;

pub struct KryonApp {
    workspace: Option<Workspace>,
    active_file: Option<PathBuf>,
    preview: String,
    message: String,
}

impl KryonApp {
    pub fn new() -> Self {
        let path = env::args_os()
            .nth(1)
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok());

        let (workspace, message) = match path {
            Some(path) => match Workspace::open(&path) {
                Ok(workspace) => (Some(workspace), "Ready".to_string()),
                Err(error) => (None, format!("Workspace error: {error}")),
            },
            None => (None, "No workspace directory".to_string()),
        };

        Self {
            workspace,
            active_file: None,
            preview: String::new(),
            message,
        }
    }

    fn activate_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if path.is_dir() {
            if let Some(workspace) = &mut self.workspace {
                workspace.toggle_directory(&path);
            }
        } else {
            match read_preview(&path) {
                Ok(content) => {
                    self.preview = content;
                    self.active_file = Some(path);
                    self.message = "Ready".to_string();
                }
                Err(error) => {
                    self.message = format!("Open failed: {error}");
                }
            }
        }

        cx.notify();
    }

    fn render_explorer(&self, cx: &Context<Self>) -> impl IntoElement {
        let entries = self
            .workspace
            .as_ref()
            .map(|workspace| workspace.entries())
            .transpose();

        let entries = match entries {
            Ok(Some(entries)) => entries,
            Ok(None) => Vec::new(),
            Err(_) => Vec::new(),
        };

        let rows = entries
            .into_iter()
            .enumerate()
            .map(|(index, entry)| {
                let path = entry.path.clone();

                let marker = if entry.is_dir {
                    if entry.expanded { "▾" } else { "▸" }
                } else {
                    " "
                };

                let label = format!("{marker}  {}", entry.name);

                let color = if entry.is_dir { ACCENT } else { TEXT };

                div()
                    .id(format!("explorer-file-{index}"))
                    .h(px(26.0))
                    .flex()
                    .items_center()
                    .pl(px((entry.depth * 14 + 12) as f32))
                    .text_sm()
                    .text_color(rgb(color))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x272C35)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.activate_path(path.clone(), cx);
                    }))
                    .child(label)
            })
            .collect::<Vec<_>>();

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
                    .px_3()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child("Explorer"),
            )
            .child(
                div()
                    .id("explorer-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(rows),
            )
    }

    fn render_editor(&self) -> impl IntoElement {
        let content = if self.active_file.is_some() {
            self.preview.clone()
        } else {
            "Open a file from Explorer.".to_string()
        };

        let rows = content
            .lines()
            .enumerate()
            .map(|(index, line)| {
                div()
                    .flex()
                    .gap_4()
                    .text_sm()
                    .child(
                        div()
                            .w(px(40.0))
                            .text_color(rgb(MUTED))
                            .child((index + 1).to_string()),
                    )
                    .child(div().text_color(rgb(TEXT)).child(line.to_string()))
            })
            .collect::<Vec<_>>();

        div()
            .id("editor-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .p_4()
            .bg(rgb(BACKGROUND))
            .children(rows)
    }
}

impl Render for KryonApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace_name = self
            .workspace
            .as_ref()
            .map(Workspace::name)
            .unwrap_or_else(|| "No Workspace".into());

        let file_name = self
            .active_file
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Welcome".into());

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BACKGROUND))
            .text_color(rgb(TEXT))
            .child(
                div()
                    .h(px(44.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .bg(rgb(PANEL))
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(ACCENT))
                            .child("KRYON IDE"),
                    )
                    .child(workspace_name),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_explorer(cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .h(px(38.0))
                                    .flex()
                                    .items_center()
                                    .px_4()
                                    .bg(rgb(PANEL))
                                    .border_b_1()
                                    .border_color(rgb(BORDER))
                                    .child(file_name),
                            )
                            .child(self.render_editor()),
                    ),
            )
            .child(
                div()
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .bg(rgb(PANEL))
                    .border_t_1()
                    .border_color(rgb(BORDER))
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child(self.message.clone())
                    .child("UTF-8  |  Read-only preview"),
            )
    }
}
