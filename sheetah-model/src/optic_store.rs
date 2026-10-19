use std::any::TypeId;
use std::collections::{BTreeSet, HashMap};
use crate::optic::{ErasedOptic, Optic, OpticId, OpticKind};

#[derive(Default)]
pub struct OpticStore {
    optic_by_id: Vec<ErasedOptic>,
    optic_by_name: BTreeSet<(String, OpticId)>,

    optic_by_in_type: HashMap<TypeId, Vec<OpticId>>,
    optic_by_out_type: HashMap<TypeId, Vec<OpticId>>,
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
            self.optic_by_in_type
                .entry(optic.in_type_id)
                .or_default()
                .push(id);
            self.optic_by_out_type
                .entry(optic.out_type_id)
                .or_default()
                .push(id);
        }
    }

    pub fn get_by_name(&self, name: &str) -> Vec<ErasedOptic> {
        self.optic_by_name
            .range((name.to_string(), OpticId::new(0))..)
            .take_while(|(n, _)| n.as_str() == name)
            .filter_map(|(_, id)| self.get_by_id(*id))
            .collect()
    }

    pub fn get_by_id(&self, id: OpticId) -> Option<ErasedOptic> {
        self.optics.get(id.id as usize).cloned()
    }

    pub fn get_by_in_type<T: 'static>(&self) -> Vec<ErasedOptic> {
        self.optic_by_in_type
            .get(&TypeId::of::<T>())
            .map(|ids| ids.iter().filter_map(|id| self.get_by_id(*id)).collect())
            .unwrap_or_default()
    }

    pub fn get_by_out_type<T: 'static>(&self) -> Vec<ErasedOptic> {
        self.optic_by_out_type
            .get(&TypeId::of::<T>())
            .map(|ids| ids.iter().filter_map(|id| self.get_by_id(*id)).collect())
            .unwrap_or_default()
    }

    /// Finds all optics that can be composed AFTER the given optic.
    /// i.e., optics that take the 'out' type of the given optic as their 'in' type.
    pub fn find_composable_after(&self, other: &ErasedOptic) -> Vec<ErasedOptic> {
        let out_type_id = other.out_type_id;
        self.optic_by_in_type
            .get(&out_type_id)
            .map(|ids| ids.iter().filter_map(|id| self.get_by_id(*id)).collect())
            .unwrap_or_default()
    }

    /// Finds all optics that can be composed BEFORE the given optic.
    /// i.e., optics whose 'out' type matches the 'in' type of the given optic.
    pub fn find_composable_before(&self, other: &ErasedOptic) -> Vec<ErasedOptic> {
        let in_type_id = other.in_type_id();
        self.optic_by_out_type
            .get(&in_type_id)
            .map(|ids| ids.iter().filter_map(|id| self.get_by_id(*id)).collect())
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy)]
pub struct OpticKind {
    pub name: &'static str,
    pub create_optics: fn() -> Vec<ErasedOptic>,
}

inventory::collect!(OpticKind);
