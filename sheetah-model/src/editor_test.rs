#[cfg(test)]
mod tests {
    use crate::editor::{Index, Pos, Rect, Tab, View};
    use crate::shape::Shape;
    use crate::table::TableId;

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
}
