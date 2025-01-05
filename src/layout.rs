use std::collections::HashMap;

use fjadra::{Collide, Link, ManyBody, Node, PositionY, SimulationBuilder};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::{EdgeRef, IntoEdgesDirected},
};

pub trait Radius {
    fn radius(&self) -> f32;
}

pub struct GraphLayout<N: Radius, E> {
    graph: DiGraph<N, E>,
    entry: NodeIndex,
    node_positions: HashMap<NodeIndex, (f32, f32)>,
}

impl<N, E> GraphLayout<N, E>
where
    N: Radius,
{
    pub fn new(graph: DiGraph<N, E>, entry: NodeIndex) -> Self {
        let radii: Vec<_> = graph
            .raw_nodes()
            .iter()
            .map(|n| n.weight.radius())
            .collect();
        let mut sim = SimulationBuilder::new()
            .with_velocity_decay(0.1)
            .build(graph.node_indices().map(|n| {
                if n == entry {
                    Node::default().fixed_position(0., 0.)
                } else {
                    Node::default()
                }
            }))
            .add_force("gravity", PositionY::new().strength(-1.))
            .add_force("charge", ManyBody::new().strength(2.))
            .add_force("collide", Collide::new().radius(move |i| radii[i] as f64))
            .add_force(
                "link",
                Link::new(graph.node_indices().flat_map(|i| {
                    graph
                        .edges_directed(i, petgraph::Direction::Outgoing)
                        .map(|e| (e.source().index(), e.target().index()))
                }))
                .strength(1.1)
                .distance(60.),
            );

        let node_positions = sim
            .iter()
            .last()
            .expect("Simulation should return")
            .iter()
            .enumerate()
            .map(|(i, p)| (NodeIndex::new(i), (p[0] as f32, p[1] as f32)))
            .collect();

        Self {
            graph,
            entry,
            node_positions,
        }
    }
}
