use hashbrown::HashMap;
use pavenet_core::enums::NodeType;

struct Relation {
    source: NodeType,
    target: Vec<NodeType>,
}

pub struct LinkRelations {
    node_relations: Vec<Relation>,
}

impl LinkRelations {
    pub fn new(links: HashMap<NodeType, Vec<NodeType>>) -> Self {
        let mut node_relations = Vec::with_capacity(links.len());
        for (source, targets) in links {
            let relation = Relation {
                source,
                target: targets,
            };
            node_relations.push(relation);
        }
        Self { node_relations }
    }

    pub fn relations_for(&self, node_type: NodeType) -> Vec<NodeType> {
        self.node_relations
            .iter()
            .find(|relation| relation.source == node_type)
            .map(|relation| relation.target.clone())
            .unwrap_or_default()
    }
}
