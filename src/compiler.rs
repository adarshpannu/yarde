use crate::ast::*;
use crate::ast::{Expr::*, *};
use crate::flow::*;
use crate::graph::*;
use crate::includes::*;
use crate::row::*;
use crate::task::*;

pub struct Compiler;

impl Compiler {
    pub fn compile(env: &Env, qgm: &mut QGM) -> Result<Flow, String> {
        let arena: NodeArena = Arena::new();
        let graph = replace(&mut qgm.graph, Graph::new());
        let topqblock = &qgm.qblock;

        // Currently only single-table queries supported.
        assert!(topqblock.quns.len() == 1);

        let qun = &topqblock.quns[0];
        assert!(qun.tablename.is_some() && qun.qblock.is_none());

        // Turn QUN -> CSV, pred_list -> Filter
        let mut topnode;

        let name = qun.tablename.as_ref().unwrap();
        let colmap = qun.column_read_map.borrow().clone();
        topnode = CSVNode::new(env, &arena, name.clone(), 4, colmap);
        if let Some(pred_list) = topqblock.pred_list.as_ref() {
            for &pred_id in pred_list {
                topnode = topnode.filter(&arena, pred_id);
            }
        }

        // Compile selectlist, temporarily stripping agg functions
        let select_list: Vec<NodeId> = topqblock
            .select_list
            .iter()
            .map(|ne| {
                let expr_id = ne.expr_id;
                let (expr, children) = graph.get_node_with_children(expr_id);
                if let AggFunction(aggtype, is_distinct) = expr {
                    children.unwrap()[0]
                } else {
                    expr_id
                }
            })
            .collect();
        let topnode = topnode.emit(&arena, select_list);

        let flow = Flow {
            id: 99,
            nodes: arena.into_vec(),
            graph,
        };
        Ok(flow)
    }
}

/***************************************************************************************************/
impl Expr {
    pub fn eval<'a>(graph: &Graph<Expr>, expr_id: NodeId, row: &'a Row) -> Datum {
        let (expr, children) = &graph.get_node_with_children(expr_id);
        match expr {
            CID(ix) => row.get_column(*ix).clone(),
            Column { prefix, colname, qun_id, offset} => row.get_column(*offset).clone(),
            Literal(lit) => lit.clone(),
            RelExpr(op) => {
                let children = children.unwrap();
                let c0 = Expr::eval(graph, children[0], row);
                let c1 = Expr::eval(graph, children[1], row);
                let res = match (c0, op, c1) {
                    (Datum::INT(i1), RelOp::Eq, Datum::INT(i2)) => i1 == i2,
                    (Datum::INT(i1), RelOp::Ne, Datum::INT(i2)) => i1 != i2,
                    (Datum::INT(i1), RelOp::Le, Datum::INT(i2)) => i1 <= i2,
                    (Datum::INT(i1), RelOp::Lt, Datum::INT(i2)) => i1 < i2,
                    (Datum::INT(i1), RelOp::Ge, Datum::INT(i2)) => i1 >= i2,
                    (Datum::INT(i1), RelOp::Gt, Datum::INT(i2)) => i1 > i2,
                    (Datum::STR(s1), RelOp::Eq, Datum::STR(s2)) => *s1 == *s2,
                    (Datum::STR(s1), RelOp::Ne, Datum::STR(s2)) => *s1 != *s2,
                    _ => panic!("Internal error: Operands of RelOp not resolved yet."),
                };
                Datum::BOOL(res)
            }
            BinaryExpr(op) => {
                let children = children.unwrap();
                let c0 = Expr::eval(graph, children[0], row);
                let c1 = Expr::eval(graph, children[1], row);
                let res = match (c0, op, c1) {
                    (Datum::INT(i1), ArithOp::Add, Datum::INT(i2)) => i1 + i2,
                    (Datum::INT(i1), ArithOp::Sub, Datum::INT(i2)) => i1 - i2,
                    (Datum::INT(i1), ArithOp::Mul, Datum::INT(i2)) => i1 * i2,
                    (Datum::INT(i1), ArithOp::Div, Datum::INT(i2)) => i1 / i2,
                    _ => panic!("Internal error: Operands of ArithOp not resolved yet."),
                };
                Datum::INT(res)
            }
            LogExpr(op) => {
                let children = children.unwrap();
                let c0 = Expr::eval(graph, children[0], row);
                let c1 = if children.len() == 2 {
                    Expr::eval(graph, children[1], row)
                } else {
                    Datum::NULL
                };
                let res = match (c0, op, c1) {
                    (Datum::BOOL(b0), LogOp::And, Datum::BOOL(b1)) => b0 && b1,
                    (Datum::BOOL(b0), LogOp::Or, Datum::BOOL(b1)) => b0 || b1,
                    (Datum::BOOL(b0), LogOp::Not, _) => !b0,
                    _ => panic!("Internal error: Operands of LogExpr not resolved yet."),
                };
                Datum::BOOL(res)
            }
            _ => {
                debug!("Expr::eval: {:?} not implemented.", &expr);
                unimplemented!()
            }
        }
    }
}