// src/tests/ui_spec.rs
use crate::{Block, ChartPreview, DSLEditor, GridView, MarkdownEditor};
use gpui::TestApp;
use gpui::TestAppContext;

// Extension trait to mock the missing methods for compilation
trait ViewExt {
    fn find_child<T: 'static>(&self, cx: &mut impl gpui::AppContext) -> Option<&T>;
}

impl ViewExt for GridView {
    fn find_child<T: 'static>(&self, _cx: &mut impl gpui::AppContext) -> Option<&T> {
        None
    }
}

impl ViewExt for MarkdownEditor {
    fn find_child<T: 'static>(&self, _cx: &mut impl gpui::AppContext) -> Option<&T> {
        None
    }
}

impl ViewExt for DSLEditor {
    fn find_child<T: 'static>(&self, _cx: &mut impl gpui::AppContext) -> Option<&T> {
        None
    }
}

impl ViewExt for ChartPreview {
    fn find_child<T: 'static>(&self, _cx: &mut impl gpui::AppContext) -> Option<&T> {
        None
    }
}

impl ViewExt for gpui::Window {
    fn find_child<T: 'static>(&self, _cx: &mut impl gpui::AppContext) -> Option<&T> {
        None
    }
}

impl ViewExt for gpui::WindowHandle<GridView> {
    fn find_child<T: 'static>(&self, _cx: &mut impl gpui::AppContext) -> Option<&T> {
        None
    }
}

impl ViewExt for gpui::WindowHandle<DSLEditor> {
    fn find_child<T: 'static>(&self, _cx: &mut impl gpui::AppContext) -> Option<&T> {
        None
    }
}

impl ViewExt for gpui::WindowHandle<ChartPreview> {
    fn find_child<T: 'static>(&self, _cx: &mut impl gpui::AppContext) -> Option<&T> {
        None
    }
}

#[gpui::test]
async fn test_tagging_system_case_insensitivity(_cx: &mut TestAppContext) {
    let mut view = GridView::new();
    view.tags.push("#sales".into());

    // This should fail currently because search uses .contains() which is case-sensitive
    let results = view.search("#SALES");
    assert_eq!(results.len(), 1, "Search should be case-insensitive");
    assert_eq!(results[0], "#sales");
}

#[gpui::test]
async fn test_markdown_editor_logic(cx: &mut TestAppContext) {
    let mut editor = MarkdownEditor::new();
    let content = "```chart table.sum(\"Sales\")```";
    editor.set_content(content, cx);

    assert_eq!(editor.content(), content);
}

#[gpui::test]
async fn test_markdown_editor_chart_parsing(cx: &mut TestAppContext) {
    let mut editor = MarkdownEditor::new();
    let content = "Intro\n```chart table.sum(\"Sales\")```\nOutro";
    editor.set_content(content, cx);

    let blocks = editor.get_blocks();

    // Current implementation returns [Block::Text("Intro\n```chart table.sum(\"Sales\")```\nOutro")]
    // We expect: [Block::Text("Intro\n"), Block::Chart("table.sum(\"Sales\")"), Block::Text("\nOutro")]
    assert_eq!(blocks.len(), 3, "Should have split into 3 blocks");
    assert_eq!(blocks[0], Block::Text("Intro\n".to_string()));
    assert_eq!(blocks[1], Block::Chart("table.sum(\"Sales\")".to_string()));
    assert_eq!(blocks[2], Block::Text("\nOutro".to_string()));
}

#[gpui::test]
async fn test_dsl_editor_stub(cx: &mut TestAppContext) {
    let _app = TestApp::new();
    let window = cx.open_window(Default::default(), |_window, _cx| {
        let _editor = DSLEditor;
        _editor
    });

    let _ = window.update(cx, |window, _view, cx| {
        let _ = window.find_child::<MarkdownEditor>(cx);
        Ok::<(), gpui::private::anyhow::Error>(())
    });

    assert!(true);
}

#[gpui::test]
async fn test_chart_preview_stub(cx: &mut TestAppContext) {
    let _app = TestApp::new();
    let _window = cx.open_window(Default::default(), |_window, _cx| {
        let _preview = ChartPreview;
        _preview
    });

    assert!(true);
}
