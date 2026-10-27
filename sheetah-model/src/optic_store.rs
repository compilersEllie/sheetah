use std::collections::BTreeSet;

use crate::optic::{ErasedOptic, Optic, OpticId, OpticKind};

#[derive(Default)]
pub struct OpticStore {
    optic_by_id: Vec<ErasedOptic>,
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
            this.add_optics(kind);
        }
        this
    }

    pub fn optic_by_id(&self, id: OpticId) -> &ErasedOptic {
        &self.optic_by_id[id.id as usize]
    }

    pub fn add_optics(&mut self, kind: &OpticKind) {
        let optics = (kind.create)();
        for optic in optics {
            let id = OpticId { id: self.optic_by_id.len() as u128 };
            self.optic_by_id.push(optic);
            self.optic_by_name
                .insert((kind.name.to_string(), id));
        }
    }
}
