#[cfg(test)]
mod tests {
    use crate::editor::{Editor, Index, Pos, Rect, Tab, View};
    use crate::error::Result;
    use crate::shape::Shape;
    use crate::table::TableId;
    use std::sync::Arc;

    #[test]
    fn test_tab() {
        let mut tab = Tab::default();
        tab.active_pane = Index::new(0);
        assert_eq!(tab.active_pane, Index::new(0));
    }

    #[test]
    fn test_view() {
        let table_id = TableId::new(123);
        let view = View {
            table: table_id,
            cursor: Pos {
                column: Index::new(0),
                row: Index::new(0),
            },
            window: Rect {
                start: Pos {
                    column: Index::new(0),
                    row: Index::new(0),
                },
                columns: Index::new(10),
                rows: Index::new(5),
            },
            text_start: 0,
            text_length: 0,
            selected: vec![],
            shape: Shape::default(),
        };
        assert_eq!(view.table, table_id);
        assert_eq!(view.cursor.column, Index::new(0));
    }

    #[tokio::test]
    async fn test_editor_docs() -> Result<()> {
        let mut editor = Editor::default();
        let table_id1 = TableId::new(1);
        let table_id2 = TableId::new(2);

        let doc1 = editor.get_or_create_doc(table_id1);
        let doc2 = editor.get_or_create_doc(table_id2);

        assert!(!Arc::ptr_eq(&doc1, &doc2));

        let doc1_again = editor.get_or_create_doc(table_id1);
        assert!(Arc::ptr_eq(&doc1, &doc1_again));

        Ok(())
    }
}
