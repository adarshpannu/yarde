// QGM Iterators

use bitmaps::Iter;

use crate::expr::{Expr::*, *};
use crate::graph::*;
use crate::includes::*;
use crate::qgm::*;

pub struct QueryBlockIter<'a> {
    qblock_graph: &'a QueryBlockGraph,
    queue: Vec<QueryBlockKey>,
}

impl<'a> Iterator for QueryBlockIter<'a> {
    type Item = QueryBlockKey;
    fn next(&mut self) -> Option<Self::Item> {
        while self.queue.len() > 0 {
            let qbkey = self.queue.pop().unwrap();
            let (qblocknode, _, children) = self.qblock_graph.get3(qbkey);
            /*
            if let Some(children) = children {
                // UIE set operators have legs; make sure we traverse them
                self.queue.append(&mut children.clone());
            }
            */
            let children: Vec<QueryBlockKey> = qblocknode.quns.iter().filter_map(|qun| qun.qblock).collect();
            self.queue.append(&mut children.clone());
            return Some(qbkey);
        }
        None
    }
}

impl QGM {
    pub fn iter_qblocks(&self) -> QueryBlockIter {
        let mut queue = vec![self.main_qblock_key];
        queue.append(&mut self.cte_list.clone());
        QueryBlockIter {
            qblock_graph: &self.qblock_graph,
            queue,
        }
    }

    pub fn iter_quncols(&self) -> Box<dyn Iterator<Item = QunCol> + '_> {
        let qblock_iter = self.iter_qblocks();
        let iter =
            qblock_iter.flat_map(move |qblock_key| qblock_key.iter_quncols(&self.qblock_graph, &self.expr_graph));
        Box::new(iter)
    }

    pub fn iter_preds(&self) -> Box<dyn Iterator<Item = ExprKey> + '_> {
        let qblock_iter = self.iter_qblocks();
        let iter =
            qblock_iter.flat_map(move |qblock_key| qblock_key.iter_preds(&self.qblock_graph));
        Box::new(iter)
    }
}

impl ExprKey {
    pub fn iter_quncols<'g>(&self, expr_graph: &'g ExprGraph) -> Box<dyn Iterator<Item = QunCol> + 'g> {
        let it = expr_graph
            .iter(*self)
            .filter_map(move |nodeid| match &expr_graph.get(nodeid).value {
                Column { qunid, colid, .. } => Some(QunCol(*qunid, *colid)),
                CID(qunid, cid) => Some(QunCol(*qunid, *cid)),
                _ => None,
            });
        Box::new(it)
    }

    pub fn iter_quns<'g>(&self, expr_graph: &'g ExprGraph) -> Box<dyn Iterator<Item = QunId> + 'g> {
        Box::new(self.iter_quncols(expr_graph).map(|quncol| quncol.0))
    }
}

impl QueryBlockKey {
    pub fn iter_preds<'g>(&self, qblock_graph: &'g QueryBlockGraph) -> Box<dyn Iterator<Item = ExprKey> + 'g> {
        let qblock = &qblock_graph.get(*self).value;

        // Append select_list expressions
        let mut iter: Box<dyn Iterator<Item = ExprKey>> = Box::new(qblock.select_list.iter().map(|ne| ne.expr_key));

        // Append pred_list, group_by and having_clause expressions
        // todo: order_by
        for expr_list in vec![&qblock.pred_list, &qblock.group_by, &qblock.having_clause] {
            if let Some(expr_list) = expr_list {
                iter = Box::new(iter.chain(expr_list.iter().copied()));
            }
        }
        iter
    }

    pub fn iter_quncols<'g>(
        &self, qblock_graph: &'g QueryBlockGraph, expr_graph: &'g ExprGraph,
    ) -> Box<dyn Iterator<Item = QunCol> + 'g> {
        let iter = self.iter_preds(qblock_graph);
        let iter = iter.flat_map(move |expr_key| expr_key.iter_quncols(expr_graph));
        Box::new(iter)
    }
}

impl QGM {
    pub fn scratch(&self) {
        let qblock_iter = self.iter_qblocks();
        for qbkey in qblock_iter {
            debug!("qgm iter_qblock: {:?}", qbkey);
        }
        for quncol in self.iter_quncols() {
            debug!("iter_quncols {:?}", quncol);
        }
        for expr_key in self.iter_preds() {
            debug!("qblock iter_pred: {:?}", expr_key);
        }
    }
}
