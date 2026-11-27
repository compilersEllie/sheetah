#[cfg(test)]
mod tests {
    use crate::editor::{Editor, Index, Pos, Rect, Tab, View};
    use crate::shape::Shape;
    use crate::table::TableId;
    use proptest::prelude::*;
    use std::path::PathBuf;

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
    async fn editor_new_sets_workspace_dir() {
        let workspace_dir = PathBuf::from("/tmp/sheetah-workspace");
        let editor = Editor::new(workspace_dir.clone()).await.unwrap();

        assert_eq!(editor.workspace_dir, Some(workspace_dir));
        assert_eq!(editor.active_tab, Index::new(0));
        assert!(editor.tabs.is_empty());
    }

    #[test]
    fn pos_display_and_rect_dims() {
        let pos = Pos {
            column: Index::new(3),
            row: Index::new(2),
        };
        assert_eq!(pos.to_string(), "(2, 3)");

        let rect = Rect {
            start: pos,
            columns: Index::new(4),
            rows: Index::new(5),
        };
        assert_eq!(
            rect.dims(),
            Pos {
                column: Index::new(4),
                row: Index::new(5),
            }
        );
    }

    proptest! {
        #[test]
        fn index_u32_roundtrip(value in any::<u32>()) {
            let index = Index::from(value);
            prop_assert_eq!(index.raw(), value);
            prop_assert_eq!(u32::from(index), value);
            prop_assert_eq!(index.to_string(), value.to_string());
        }
    }
}
