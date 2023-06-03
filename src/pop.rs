// pop: Physical operators

use crate::{
    flow::Flow,
    graph::{ExprKey, Graph, POPKey},
    includes::*,
    pcode::PCode,
    pop_aggregation::Aggregation,
    pop_csv::CSV,
    pop_hashjoin::HashJoin,
    pop_repartition::{RepartitionRead, RepartitionWrite},
};
use std::collections::HashMap;

pub type POPGraph = Graph<POPKey, POP, POPProps>;

/***************************************************************************************************/
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Projection {
    QunCol(QunCol),
    VirtCol(ExprKey),
}

/***************************************************************************************************/
#[derive(Debug)]
pub struct ProjectionMap {
    pub hashmap: HashMap<Projection, ColId>,
}

impl ProjectionMap {
    pub fn new() -> ProjectionMap {
        ProjectionMap { hashmap: HashMap::new() }
    }

    pub fn set(&mut self, prj: Projection, colid: ColId) {
        self.hashmap.insert(prj, colid);
    }

    pub fn get(&self, prj: Projection) -> Option<ColId> {
        self.hashmap.get(&prj).cloned()
    }
}

/***************************************************************************************************/
#[derive(Debug, Serialize, Deserialize)]
pub struct POPProps {
    pub predicates: Option<Vec<PCode>>,
    pub cols: Option<Vec<ColId>>,
    pub virtcols: Option<Vec<PCode>>,
    pub npartitions: usize,
    pub index_in_stage: usize,
}

impl POPProps {
    pub fn new(predicates: Option<Vec<PCode>>, cols: Option<Vec<ColId>>, virtcols: Option<Vec<PCode>>, npartitions: usize) -> POPProps {
        POPProps {
            predicates,
            cols,
            virtcols,
            npartitions,
            index_in_stage: 0,
        }
    }
}

/***************************************************************************************************/
#[derive(Debug, Serialize, Deserialize)]
pub enum POP {
    CSV(CSV),
    HashJoin(HashJoin),
    RepartitionWrite(RepartitionWrite),
    RepartitionRead(RepartitionRead),
    Aggregation(Aggregation),
}

/***************************************************************************************************/
pub trait POPContext {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn next(&mut self, flow: &Flow) -> Result<Chunk<Box<dyn Array>>, String>;
}
