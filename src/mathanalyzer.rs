use libxml::tree::Node;


#[derive(Clone, Copy)]
pub enum IdentifierTags {
    First,
}

#[derive(Clone)]
pub struct Identifier {
    node : Node,
    tags : Vec<IdentifierTags>,
}



pub fn find_potential_identifiers(math_node : Node) -> Vec<Identifier> {
    let mut results : Vec <Identifier> = Vec::new();

    assert_eq!(math_node.get_name(), "math");
    match get_first_identifier(math_node) {
        Some(x) => results.push(Identifier { node: x, tags: vec![IdentifierTags::First] } ),
        None => { }
    }
    
    results
}

fn get_first_identifier(root: Node) -> Option<Node> {
    match &root.get_name() as &str {
        "mtext" => None,
        "annotation" => None,
        "xml-annotation" => None,
        "mfrac" => None,
        "mtable" => None,
        "mi" => Some(root),
        "msub" => get_first_identifier_helper(root),
        "msup" => get_first_identifier_helper(root),
        "msubsup" => get_first_identifier_helper(root),
        _ => match root.get_first_child() {
            None => None,
            Some(x) => x.get_first_child(),
        }
    }
}


fn get_first_identifier_helper(root: Node) -> Option<Node> {
    match root.get_first_child() {
        None => None,
        Some(x) =>
            if get_first_identifier(x).is_some() {
                Some(root)
            } else {
                None
            },
    }
}

