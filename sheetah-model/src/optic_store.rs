use crate::optic::{Optic, OpticId};

#[derive(Default)]
pub struct OpticStore {
    optic_by_id: Vec<Optic>,
    optic_by_name: BTreeSet<(String, OpticId)>,

    // TTODO(feat): optic_by_params: HashMap<TypeId, VecOpticId>>,
    // TODO(feat): optic_by_partial_params: HashMap<TypeId, VecOpticId>>,
    // TODO(feat): optic_by_result: HashMap<TypeId, VecOpticId>>,
}

impl OpticStore {
    pub fn new() -> Self {
        let mut this = Self::default();
        // Register the optics .
        for kind in inventory::iter::<OpticKind> {
            let id = this.optics_by_id.len();
            this.optics_by_id
                .insert(kind.clone());
            this.optics_by_name
                .insert((kind.name.to_string(), id));
        }
        this
    }
}


#[derive(Clone)]
pub struct OpticKind {
    pub name: &'static str,
    /// Lazily create an on disk store for this backend.
    pub create:
        &'static (dyn Fn() -> Arc<RwLock<dyn Optic>> + Sync + Send + 'static),
}

inventory::collect!(OpticKind);
