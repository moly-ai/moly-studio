//! Custom CodeView widget for MCP JSON editing

use makepad_code_editor::code_editor::{CodeEditorAction, KeepCursorInView};
use makepad_code_editor::decoration::DecorationSet;
use makepad_code_editor::{CodeDocument, CodeEditor, CodeSession};
use makepad_widgets::*;

live_design! {
    use link::widgets::*;
    use link::theme::*;
    use link::shaders::*;
    use moly_widgets::theme::*;

    pub MolyCodeView = {{MolyCodeView}} {
        editor: <CodeEditor> {
            pad_left_top: vec2(10.0, 10.0)
            height: Fill
            width: Fill
            empty_page_at_end: false
            read_only: false
            show_gutter: false
            draw_bg: { color: #1d2330 }
            draw_text: {
                text_style: {
                    font_size: 10,
                }
            }

            // Electron Highlighter inspired theme
            token_colors: {
                whitespace: #a8b5d1,
                delimiter: #a8b5d1,
                delimiter_highlight: #c5cee0,
                error_decoration: #f44747,
                warning_decoration: #cd9731,
                unknown: #a8b5d1,
                branch_keyword: #d2a6ef,
                constant: #ffd9af,
                identifier: #a8b5d1,
                loop_keyword: #d2a6ef,
                number: #ffd9af,
                other_keyword: #d2a6ef,
                punctuator: #a8b5d1,
                string: #58ffc7,
                function: #82aaff,
                typename: #fcf9c3,
                comment: #506686,
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct MolyCodeView {
    #[wrap]
    #[live]
    pub editor: CodeEditor,
    #[rust]
    pub session: Option<CodeSession>,
    #[live(false)]
    keep_cursor_at_end: bool,
    #[live]
    text: ArcStringMut,
}

impl MolyCodeView {
    pub fn lazy_init_session(&mut self) {
        if self.session.is_none() {
            let dec = DecorationSet::new();
            let doc = CodeDocument::new(self.text.as_ref().into(), dec);
            self.session = Some(CodeSession::new(doc));
            self.session.as_mut().unwrap().handle_changes();
            if self.keep_cursor_at_end {
                self.session.as_mut().unwrap().set_cursor_at_file_end();
                self.editor.keep_cursor_in_view = KeepCursorInView::Once
            }
        }
    }
}

impl Widget for MolyCodeView {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.lazy_init_session();
        let session = self.session.as_mut().unwrap();
        self.editor.draw_walk_editor(cx, session, walk);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        self.lazy_init_session();
        let session = self.session.as_mut().unwrap();
        for action in self
            .editor
            .handle_event(cx, event, &mut Scope::empty(), session)
        {
            session.handle_changes();

            match action {
                CodeEditorAction::TextDidChange => {
                    let document_text = session.document().as_text().to_string();
                    if self.text.as_ref() != &document_text {
                        self.text.as_mut_empty().clear();
                        self.text.as_mut_empty().push_str(&document_text);
                    }
                }
                _ => {}
            }
        }
    }

    fn text(&self) -> String {
        if let Some(session) = &self.session {
            session.document().as_text().to_string()
        } else {
            self.text.as_ref().to_string()
        }
    }

    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        let current_text = if let Some(session) = &self.session {
            session.document().as_text().to_string()
        } else {
            self.text.as_ref().to_string()
        };

        if current_text != v {
            self.text.as_mut_empty().clear();
            self.text.as_mut_empty().push_str(v);

            if let Some(session) = &mut self.session {
                session.document().replace(v.into());
                session.handle_changes();
            }

            self.redraw(cx);
        }
    }
}

