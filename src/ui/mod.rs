mod theme;

use gpui::{
    Context, FocusHandle, IntoElement, KeyDownEvent, PathPromptOptions, Render,
    StatefulInteractiveElement, Window, div, prelude::*, px, rgb, uniform_list,
};
use std::ops::Range;
use std::{env, path::PathBuf};

use crate::syntax::highlight::highlight_rust;
use gpui::StyledText;

use crate::editor::EditorDocument;
use crate::workspace::Workspace;
use theme::KryonTheme as Theme;

pub struct KryonApp {
    workspace: Option<Workspace>,
    active_file: Option<PathBuf>,
    document: Option<EditorDocument>,
    message: String,
    editor_focus: FocusHandle,
}

impl KryonApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let path = env::args_os()
            .nth(1)
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok());

        let (workspace, message) = match path {
            Some(path) => match Workspace::open(&path) {
                Ok(workspace) => (Some(workspace), "Ready".to_string()),
                Err(error) => (None, format!("Workspace error: {error}")),
            },
            None => (None, "No workspace".to_string()),
        };

        Self {
            workspace,
            active_file: None,
            document: None,
            message,
            editor_focus: cx.focus_handle(),
        }
    }

    // Open a file or toggle a directory.
    fn activate_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if path.is_dir() {
            if let Some(workspace) = &mut self.workspace {
                workspace.toggle_directory(&path);
            }
        } else {
            match EditorDocument::open(&path) {
                Ok(document) => {
                    self.active_file = Some(path);
                    self.document = Some(document);
                    self.message = "Ready".to_string();
                }

                Err(error) => {
                    self.message = format!("Open failed: {error}");
                }
            }
        }

        cx.notify();
    }

    // Set the active workspace.
    fn set_workspace(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        println!("[KRYON] Opening workspace: {}", path.display());

        match Workspace::open(&path) {
            Ok(workspace) => {
                self.workspace = Some(workspace);
                self.active_file = None;
                self.document = None;

                self.message = format!("Opened {}", path.display());

                println!("[KRYON] Workspace opened successfully");
            }

            Err(error) => {
                self.message = format!("Unable to open folder: {error}");

                eprintln!("[KRYON] Workspace error: {error}");
            }
        }

        cx.notify();
    }

    // Open the native system folder picker.
    fn open_folder(&mut self, cx: &mut Context<Self>) {
        println!("[KRYON] Opening folder picker");

        let picker = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Open Folder".into()),
        });

        cx.spawn(async move |this, cx| match picker.await {
            Ok(Ok(Some(paths))) => {
                println!("[KRYON] Selected paths: {paths:?}");

                if let Some(path) = paths.into_iter().next() {
                    if let Err(error) = this.update(cx, |app, cx| {
                        app.set_workspace(path, cx);
                    }) {
                        eprintln!("[KRYON] Workspace update failed: {error}");
                    }
                }
            }

            Ok(Ok(None)) => {
                println!("[KRYON] Folder selection cancelled");
            }

            Ok(Err(error)) => {
                eprintln!("[KRYON] Folder picker error: {error}");
            }

            Err(error) => {
                eprintln!("[KRYON] Picker task error: {error}");
            }
        })
        .detach();
    }

    // Render the top application bar.
    fn render_title_bar(&self, cx: &Context<Self>) -> impl IntoElement {
        let workspace_name = self
            .workspace
            .as_ref()
            .map(Workspace::name)
            .unwrap_or_else(|| "No Workspace".into());

        div()
            .h(px(48.0))
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .bg(rgb(Theme::SIDEBAR))
            .border_b_1()
            .border_color(rgb(Theme::BORDER))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(Theme::ACCENT))
                            .child("KRYON"),
                    )
                    .child(div().text_sm().text_color(rgb(Theme::MUTED)).child("IDE"))
                    .child(
                        div()
                            .ml_4()
                            .text_sm()
                            .text_color(rgb(Theme::TEXT_SECONDARY))
                            .child(workspace_name),
                    ),
            )
            .child(
                div()
                    .id("open-folder-button")
                    .h(px(30.0))
                    .px_3()
                    .flex()
                    .items_center()
                    .rounded_md()
                    .bg(rgb(Theme::ACTIVE))
                    .text_color(rgb(Theme::TEXT))
                    .text_sm()
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(Theme::SURFACE_HOVER)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open_folder(cx);
                    }))
                    .child("Open Folder"),
            )
    }

    // Render the project explorer.
    fn render_explorer(&self, cx: &Context<Self>) -> impl IntoElement {
        let entries = self
            .workspace
            .as_ref()
            .map(|workspace| workspace.entries())
            .transpose();

        let entries = match entries {
            Ok(Some(entries)) => {
                println!("[KRYON] Explorer entries: {}", entries.len());

                entries
            }

            Ok(None) => Vec::new(),

            Err(error) => {
                eprintln!("[KRYON] Explorer error: {error}");

                Vec::new()
            }
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

                let color = if entry.is_dir {
                    Theme::FOLDER
                } else {
                    Theme::TEXT
                };

                let is_active = self
                    .active_file
                    .as_ref()
                    .is_some_and(|active| active == &path);

                let background = if is_active {
                    Theme::ACTIVE
                } else {
                    Theme::SIDEBAR
                };

                div()
                    .id(format!("explorer-file-{index}"))
                    .h(px(26.0))
                    .w_full()
                    .flex()
                    .items_center()
                    .pl(px((entry.depth * 14 + 12) as f32))
                    .bg(rgb(background))
                    .text_sm()
                    .text_color(rgb(color))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(Theme::SURFACE_HOVER)))
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
            .bg(rgb(Theme::SIDEBAR))
            .border_r_1()
            .border_color(rgb(Theme::BORDER))
            .child(
                div()
                    .h(px(42.0))
                    .flex()
                    .items_center()
                    .px_3()
                    .text_xs()
                    .text_color(rgb(Theme::MUTED))
                    .child("EXPLORER"),
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

    // Render the active file tab.
    fn render_tab_bar(&self) -> impl IntoElement {
        let file_name = self
            .active_file
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Welcome".into());

        div()
            .h(px(38.0))
            .w_full()
            .flex()
            .items_center()
            .bg(rgb(Theme::SIDEBAR))
            .border_b_1()
            .border_color(rgb(Theme::BORDER))
            .child(
                div()
                    .h_full()
                    .flex()
                    .items_center()
                    .px_4()
                    .bg(rgb(Theme::ACTIVE))
                    .border_r_1()
                    .border_color(rgb(Theme::BORDER))
                    .text_sm()
                    .text_color(rgb(Theme::TEXT))
                    .child(file_name),
            )
    }

    // Render a read-only file preview.
    fn render_editor(&self, cx: &Context<Self>) -> impl IntoElement {
        let line_count = self
            .document
            .as_ref()
            .map(EditorDocument::line_count)
            .unwrap_or(0);

        div()
            .id("kryon-editor")
            .flex_1()
            .min_h_0()
            .bg(rgb(Theme::BACKGROUND))
            .track_focus(&self.editor_focus)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.editor_focus.focus(window, cx);
                }),
            )
            .on_key_down(cx.listener(|this, event, window, cx| {
                this.handle_editor_key(event, window, cx);
            }))
            .child(
                uniform_list(
                    "editor-lines",
                    line_count,
                    cx.processor(|this, range: Range<usize>, _window, _cx| {
                        let Some(document) = &this.document else {
                            return Vec::new();
                        };

                        range
                            .filter_map(|index| {
                                let line = document.line(index)?;

                                Some(
                                    div()
                                        .id(index)
                                        .h(px(24.0))
                                        .flex()
                                        .items_center()
                                        .gap_4()
                                        .px_4()
                                        .text_sm()
                                        .child(
                                            div()
                                                .w(px(45.0))
                                                .text_color(rgb(Theme::MUTED))
                                                .child((index + 1).to_string()),
                                        )
                                        .child(
                                            div().text_color(rgb(Theme::TEXT)).child(
                                                StyledText::new(line.clone())
                                                    .with_highlights(highlight_rust(&line)),
                                            ),
                                        ),
                                )
                            })
                            .collect::<Vec<_>>()
                    }),
                )
                .h_full()
                .w_full(),
            )
    }

    // Render the bottom status bar.
    fn render_status_bar(&self) -> impl IntoElement {
        let cursor_position = self
            .document
            .as_ref()
            .map(|document| {
                let cursor = document.cursor();

                format!("Ln {}, Col {}", cursor.line + 1, cursor.column + 1,)
            })
            .unwrap_or_else(|| "Ln 1, Col 1".into());
        div()
            .h(px(28.0))
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .bg(rgb(Theme::SIDEBAR))
            .border_t_1()
            .border_color(rgb(Theme::BORDER))
            .text_xs()
            .text_color(rgb(Theme::MUTED))
            .child(self.message.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(cursor_position)
                    .child("UTF-8")
                    .child("Rust"),
            )
    }

    fn handle_editor_key(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(document) = self.document.as_mut() else {
            return;
        };

        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;

        let command = modifiers.control || modifiers.platform;

        if command {
            match key {
                "s" => match document.save() {
                    Ok(()) => {
                        self.message = "File saved".to_string();
                    }
                    Err(error) => {
                        self.message = format!("Save failed: {error}");
                    }
                },

                "z" if modifiers.shift => {
                    document.redo();
                }

                "z" => {
                    document.undo();
                }

                "y" => {
                    document.redo();
                }

                _ => return,
            }
        } else {
            match key {
                "left" => document.move_left(),
                "right" => document.move_right(),
                "up" => document.move_up(),
                "down" => document.move_down(),

                "backspace" => document.backspace(),
                "delete" => document.delete_forward(),

                "enter" => document.insert_newline(),

                _ => return,
            }
        }

        cx.notify();
    }
}

impl Render for KryonApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(Theme::BACKGROUND))
            .text_color(rgb(Theme::TEXT))
            .child(self.render_title_bar(cx))
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
                            .child(self.render_tab_bar())
                            .child(self.render_editor(cx)),
                    ),
            )
            .child(self.render_status_bar())
    }
}
