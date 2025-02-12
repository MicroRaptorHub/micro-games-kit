use crate::context::GameContext;
use nodio::{
    graph::Graph,
    query::{Node, Traverse},
    AnyIndex,
};

pub struct GameObjectParent;
pub struct GameObjectChild;

#[derive(Default)]
#[allow(clippy::type_complexity)]
pub struct GameObjectNode {
    on_activate: Option<Box<dyn Fn(AnyIndex, &mut GameContext)>>,
    on_deactivate: Option<Box<dyn Fn(AnyIndex, &mut GameContext)>>,
    on_process: Option<Box<dyn Fn(AnyIndex, &mut GameContext, f32)>>,
    on_draw: Option<Box<dyn Fn(AnyIndex, &mut GameContext)>>,
}

impl GameObjectNode {
    pub fn activate(graph: &Graph, root: AnyIndex, context: &mut GameContext) {
        for index in graph.query::<Traverse<GameObjectChild, Node<GameObjectNode>>>(root) {
            if let Ok(node) = graph.read::<GameObjectNode>(index) {
                if let Some(callback) = node.on_activate.as_deref() {
                    callback(index, context);
                }
            }
        }
    }

    pub fn deactivate(graph: &Graph, root: AnyIndex, context: &mut GameContext) {
        for index in graph.query::<Traverse<GameObjectChild, Node<GameObjectNode>>>(root) {
            if let Ok(node) = graph.read::<GameObjectNode>(index) {
                if let Some(callback) = node.on_deactivate.as_deref() {
                    callback(index, context);
                }
            }
        }
    }

    pub fn process(graph: &Graph, root: AnyIndex, context: &mut GameContext, delta_time: f32) {
        for index in graph.query::<Traverse<GameObjectChild, Node<GameObjectNode>>>(root) {
            if let Ok(node) = graph.read::<GameObjectNode>(index) {
                if let Some(callback) = node.on_process.as_deref() {
                    callback(index, context, delta_time);
                }
            }
        }
    }

    pub fn draw(graph: &Graph, root: AnyIndex, context: &mut GameContext) {
        for index in graph.query::<Traverse<GameObjectChild, Node<GameObjectNode>>>(root) {
            if let Ok(node) = graph.read::<GameObjectNode>(index) {
                if let Some(callback) = node.on_draw.as_deref() {
                    callback(index, context);
                }
            }
        }
    }
}
