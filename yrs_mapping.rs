//! MWE: a spreadsheet-ish table in a yrs CRDT document, exported to JSON.
//!
//! Layout: root map "sheet" -> { "A1": <value>, "B1": <value>, ... }
//! Flat A1-style keys keep concurrent edits cell-granular (last-writer-wins
//! per key), which is the natural fit for a spreadsheet.
//!
//! NOT COMPILED when written (no crates.io access in the authoring sandbox).
//! Lines marked [unverified] are from memory, not from docs.rs.

use yrs::updates::decoder::Decode;
use yrs::{Any, Doc, Map, MapPrelim, ReadTxn, StateVector, ToJson, Transact, Update};

fn cell_key(col: u8, row: u32) -> String {
    format!("{}{}", (b'A' + col) as char, row)
}

fn main() {
    // --- 1. Build a table in doc A -------------------------------------
    let doc_a = Doc::new();
    let sheet_a = doc_a.get_or_insert_map("sheet");
    {
        let mut txn = doc_a.transact_mut();
        let header = ["item", "qty", "price"];
        for (c, h) in header.iter().enumerate() {
            sheet_a.insert(&mut txn, cell_key(c as u8, 1), Any::from(*h)); // [unverified] From<&str>
        }
        let rows = [("widget", 3.0, 2.5), ("gadget", 10.0, 1.25)];
        for (i, (name, qty, price)) in rows.iter().enumerate() {
            let r = i as u32 + 2;
            sheet_a.insert(&mut txn, cell_key(0, r), Any::from(*name));
            sheet_a.insert(&mut txn, cell_key(1, r), Any::Number(*qty));
            sheet_a.insert(&mut txn, cell_key(2, r), Any::Number(*price));
        }
        // Formulas are just strings; evaluation is your problem (yrs has no calc engine).
        sheet_a.insert(&mut txn, "D2", Any::from("=B2*C2"));

        // Nested structure also works, e.g. per-cell metadata (confirmed API: MapPrelim::from([..])).
        sheet_a.insert(
            &mut txn,
            "meta",
            MapPrelim::from([("title", Any::from("Stock")), ("version", Any::Number(1.0))]),
        );
    } // txn commits on drop

    // --- 2. Sync to a second replica via state-vector diff --------------
    let doc_b = Doc::new();
    let sheet_b = doc_b.get_or_insert_map("sheet");
    let update = doc_a
        .transact()
        .encode_state_as_update_v1(&StateVector::default()); // [unverified] name
    doc_b
        .transact_mut()
        .apply_update(Update::decode_v1(&update).unwrap()) // [unverified] Result handling
        .unwrap();

    // Concurrent-style edit on B, then sync back.
    {
        let mut txn = doc_b.transact_mut();
        sheet_b.insert(&mut txn, "B2", Any::Number(5.0));
    }
    let sv_a = doc_a.transact().state_vector();
    let diff = doc_b.transact().encode_state_as_update_v1(&sv_a);
    doc_a
        .transact_mut()
        .apply_update(Update::decode_v1(&diff).unwrap())
        .unwrap();

    // --- 3. Export to JSON ----------------------------------------------
    let txn = doc_a.transact();
    let any: Any = sheet_a.to_json(&txn); // ToJson::to_json -> Any (confirmed)
    let mut buf = String::new();
    any.to_json(&mut buf); // Any::to_json(&mut String) (seen in yrs tests)
    // Re-format with serde_json for readable output. Any also implements serde Serialize.
    let v: serde_json::Value = serde_json::from_str(&buf).unwrap();
    println!("{}", serde_json::to_string_pretty(&v).unwrap());

    // --- 4. Point read ---------------------------------------------------
    println!("B2 = {:?}", sheet_a.get(&txn, "B2")); // Option<Out> (confirmed)
}
