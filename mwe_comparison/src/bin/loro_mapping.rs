//! MWE: same spreadsheet-ish table as the yrs example, but with Loro.
//!
//! Layout: root map "sheet" -> { "A1": <value>, ..., "meta": { ... } }
//!
//! NOT COMPILED when written (no crates.io access in the authoring sandbox).
//! Lines marked [unverified] are from memory, not from docs.rs.

use loro::{ExportMode, LoroDoc, LoroMap, ToJson};

fn cell_key(col: u8, row: u32) -> String {
    format!("{}{}", (b'A' + col) as char, row)
}

fn main() {
    // --- 1. Build a table in doc A -------------------------------------
    // No explicit transaction object: methods take &self and return Result.
    let doc_a = LoroDoc::new();
    let sheet_a = doc_a.get_map("sheet");

    for (c, h) in ["item", "qty", "price"].iter().enumerate() {
        sheet_a.insert(&cell_key(c as u8, 1), *h).unwrap();
    }
    let rows = [("widget", 3.0, 2.5), ("gadget", 10.0, 1.25)];
    for (i, (name, qty, price)) in rows.iter().enumerate() {
        let r = i as u32 + 2;
        sheet_a.insert(&cell_key(0, r), *name).unwrap();
        sheet_a.insert(&cell_key(1, r), *qty).unwrap(); // [unverified] f64 -> LoroValue
        sheet_a.insert(&cell_key(2, r), *price).unwrap();
    }
    // Formulas are just strings; no calc engine here either.
    sheet_a.insert("D2", "=B2*C2").unwrap();

    // Nested container (confirmed API: insert_container, as in the crate's readme test).
    let meta = sheet_a.insert_container("meta", LoroMap::new()).unwrap();
    meta.insert("title", "Stock").unwrap();
    meta.insert("version", 1).unwrap();

    doc_a.commit(); // confirmed in README example

    // --- 2. Sync to a second replica ------------------------------------
    let doc_b = LoroDoc::new();
    doc_b
        .import(&doc_a.export(ExportMode::all_updates()).unwrap()) // confirmed in README
        .unwrap();

    // Edit on B, then sync back.
    let sheet_b = doc_b.get_map("sheet");
    sheet_b.insert("B2", 5.0).unwrap();
    doc_b.commit();

    // Incremental sync: only ops A hasn't seen.
    let vv_a = doc_a.oplog_vv(); // [unverified] name
    let diff = doc_b.export(ExportMode::updates(&vv_a)).unwrap(); // [unverified] signature
    doc_a.import(&diff).unwrap();

    // --- 3. Export to JSON ----------------------------------------------
    // get_deep_value().to_json_value() -> serde_json::Value (confirmed in readme test)
    let v = doc_a.get_deep_value().to_json_value();
    println!("{}", serde_json::to_string_pretty(&v).unwrap());

    // --- 4. Point read ---------------------------------------------------
    println!("B2 = {:?}", sheet_a.get("B2")); // [unverified] Option<ValueOrContainer>
}
