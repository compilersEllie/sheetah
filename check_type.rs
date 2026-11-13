use loro::LoroDoc;

fn main() {
    let mut doc = LoroDoc::new();
    let changes = doc.export_changes();
    println!("{}", std::any::type_name_of_val(&changes));
}
