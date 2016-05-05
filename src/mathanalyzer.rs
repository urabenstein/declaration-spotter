use libxml::tree::Node;


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IdentifierTags {
    First,
    FirstSeq,
    RelSeq,   // relations between identifiers, e.g. x1 < x2 < ... < xn
    Ellipsis,
    Structured,
}

#[derive(Clone)]
pub struct Identifier {
    pub start : Node,
    pub end : Node,
    pub tags : Vec<IdentifierTags>,
}



pub fn find_potential_identifiers(math_node : Node) -> Vec<Identifier> {
    let mut results : Vec <Identifier> = Vec::new();

    assert_eq!(math_node.get_name(), "math");
    match get_first_identifier(math_node.clone()) {
        Some(x) => {
            results.push(Identifier { start: x.clone(), tags: vec![IdentifierTags::First], end: find_end_of_identifier(x.clone()) } ); },
        None => { }
    };
    println!("Cur: {}", math_node.get_all_content());
    match get_first_identifier_seq(Some(math_node.clone()), None) {
        None => { }
        Some((mut nodes, separator)) => {
            nodes.reverse();
            if nodes.len() > 1 {  // sequence has to have at least two elements ;)
                // check if \ldots was used:
                let mut ellipsis: bool = false;
                let mut pos = 0usize;
                while pos < (&nodes).len() {
                    if nodes[pos].0.get_all_content() == "\u{2026}" {
                        ellipsis = true;
                        nodes.remove(pos);
                        // don#t break, possibly multiple ellipses
                    } else {
                        pos += 1;
                        if pos >= (&nodes).len() { break; }
                    }
                }
                let mut tags = vec![IdentifierTags::FirstSeq];
                if ellipsis {
                    tags.push(IdentifierTags::Ellipsis);
                }
                if separator != "," && separator != "" {
                    // println!("Sep: '{}'", separator);
                    tags.push(IdentifierTags::RelSeq);
                }
                    //for n in &nodes {
                    //     results.push(Identifier { node: n.clone(), tags: tags.clone() });
                    // }
                if ellipsis {
                    results.push(Identifier { start: nodes[0].0.clone(), tags: tags.clone(), end: nodes[nodes.len()-1].1.clone() });
                } else {
                    // println!("\"{}\" S:'{}' - #(nodes) = {}", math_node.get_all_content(), nodes[pos].0.get_all_content(), nodes.len());
                    for n in &nodes {
                        results.push( Identifier { start: n.0.clone(), tags: tags.clone(), end: n.1.clone() });
                    }
                }
            }
        }
    };

    match get_first_structured_identifier(math_node.clone()) {
        None => { },
        Some((start, end, commasep)) => {
            if commasep {
                results.push( Identifier { start: start, end: end, tags: vec![IdentifierTags::Structured] } );
            }
        }
    }
    
    results
}



fn get_first_identifier_seq(root_opt: Option<Node>, sep: Option<&str>) -> Option<(Vec<(Node, Node)>, String)> {
    if root_opt.is_none() {
        return None;
    }
    let root = root_opt.unwrap();
    match &root.get_name() as &str { "mtext" => None,
        "annotation" | "xml-annotation" => None,
        "mfrac" | "mtable" => None,

        "mi" => 
            match get_first_identifier_seq(root.clone().get_next_sibling(), sep) {
                Some((mut v, s)) => { let l = root.clone();
                                      v.push((root, l));
                                      Some((v, s)) },
                None => Some((vec![(root.clone(), root.clone())], if sep.is_none() { "" } else { sep.unwrap() }.to_owned())),
            },
        "msub" | "msup" | "msubsup" => 
            match get_first_identifier_seq_helper(root.clone()) {
                Some(_) =>
                    match get_first_identifier_seq(root.clone().get_next_sibling(), sep) {
                        Some((mut v, s)) => { let l = root.clone();
                                              v.push((root, l));
                                              Some((v, s)) },
                        // None => Some(vec![root]),
                        None => Some((vec![(root.clone(), root.clone())], if sep.is_none() { "" } else { sep.unwrap() }.to_owned())),
                    },
                None => None,
            },
        "mo" => 
            match &root.get_all_content() as &str {
                /* "(" | */ "" | "\u{2062}" /* invisible times */  | /* "\u{2026}" /* \ldots */ | */ "\u{27E8}" /* \langle */ =>
                    get_first_identifier_seq(root.get_next_sibling(), sep),
                "=" => None,
                x => match sep {
                    None => get_first_identifier_seq(root.clone().get_next_sibling(), Some(x)),
                    Some(y) => if x == y {
                        get_first_identifier_seq(root.get_next_sibling(), Some(x))
                    } else { None },
                },
            },
        _ => get_first_identifier_seq(root.get_first_child(), sep),
    }
}


/* fn get_first_identifier_seq(root_opt: Option<Node>, sep: Option<&str>) -> Option<(Vec<(Node, Node)>, String)> {
    if root_opt.is_none() {
        return None;
    }
    let root = root_opt.unwrap();
    match &root.get_name() as &str { "mtext" => None,
        "annotation" | "xml-annotation" => None,
        "mfrac" | "mtable" => None,

        "mi" => 
            match get_first_identifier_seq(find_end_of_identifier(root.clone()).get_next_sibling(), sep) {
                Some((mut v, s)) => { let l = find_end_of_identifier(root.clone());
                                      v.push((root, l));
                                      Some((v, s)) },
                None => Some((vec![(root.clone(), find_end_of_identifier(root.clone()))], if sep.is_none() { "" } else { sep.unwrap() }.to_owned())),
            },
        "msub" | "msup" | "msubsup" => 
            match get_first_identifier_seq_helper(root.clone()) {
                Some(_) =>
                    match get_first_identifier_seq(find_end_of_identifier(root.clone()).get_next_sibling(), sep) {
                        Some((mut v, s)) => { let l = find_end_of_identifier(root.clone());
                                              v.push((root, l));
                                              Some((v, s)) },
                        // None => Some(vec![root]),
                        None => Some((vec![(root.clone(), find_end_of_identifier(root.clone()))], if sep.is_none() { "" } else { sep.unwrap() }.to_owned())),
                    },
                None => None,
            },
        "mo" => 
            match &root.get_all_content() as &str {
                "(" | "" | "\u{2062}" /* invisible times */  | /* "\u{2026}" /* \ldots */ | */ "\u{27E8}" /* \langle */ =>
                    get_first_identifier_seq(root.get_next_sibling(), sep),
                "=" => None,
                x => match sep {
                    None => get_first_identifier_seq(find_end_of_identifier(root.clone()).get_next_sibling(), Some(x)),
                    Some(y) => if x == y {
                        get_first_identifier_seq(find_end_of_identifier(root).get_next_sibling(), Some(x))
                    } else { None },
                },
            },
        _ => get_first_identifier_seq(root.get_first_child(), sep),
    }
}
*/


fn get_first_identifier_seq_helper(root: Node) -> Option<Node> {
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

fn get_first_structured_identifier(root: Node) -> Option<(Node, Node, bool)> {
    match &root.get_name() as &str {
        "annotation" | "xml-annotation" | "mfrac" | "mtable" | "mtext" | "mi" |
            "msub" | "msup" | "msubsup" => None,
        "mo" =>  {
            let d = match &root.get_all_content() as &str {
                "(" => Some(")"),
                "\u{27E8}" => Some("\u{27E9}"),  // \langle => \rangle
                "<" => Some(">"),
                "{" => Some("}"),
                _ => None
            };
            if d.is_some() {
                println!("Started with: {}", d.unwrap());
                let mut commasep : bool = false;
                let mut cur = root.clone().get_next_sibling();
                while cur.is_some() {
                    println!("Matching {}", cur.as_ref().unwrap().get_all_content());
                    match &cur.as_ref().unwrap().get_name() as &str {
                        "msub" | "msup" | "msubsup" | "mi" => cur = root.clone().get_next_sibling(),
                        "mo" => {
                            let c = cur.as_ref().unwrap().get_all_content();
                            if c == d.unwrap() {
                                return Some((root, cur.unwrap(), commasep));
                            } else if c == "," {
                                commasep = true;
                            }
                        },
                        _ => { return None; },
                    }
                    cur = cur.unwrap().get_next_sibling();
                }
                None
            } else {
                None
            }
        },
        _ => match root.get_first_child() {
            None => None,
            Some(x) => get_first_structured_identifier(x),
        }
    }
}



fn get_first_identifier(root: Node) -> Option<Node> {
    match &root.get_name() as &str { "mtext" => None,
        "annotation" => None,
        "xml-annotation" => None,
        "mfrac" => None,
        "mtable" => None,
        "mi" => Some(root),
        "msub" => get_first_identifier_helper(root),
        "msup" => get_first_identifier_helper(root),
        "msubsup" => get_first_identifier_helper(root),
        "mo" => if root.get_all_content() == "(" {
            match root.get_next_sibling() {
                None => None,
                Some(x) => get_first_identifier(x),
            } } else {
                None
            },
        _ => match root.get_first_child() {
            None => None,
            Some(x) => get_first_identifier(x),
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

pub fn get_last_identifier(root: Node) -> Option<Node> {
    match &root.get_name() as &str { "mtext" => None,
        "annotation" | "xml-annotation" | "mfrac" | "mtable" | "mo" => 
            match root.get_next_sibling() {
                None => None,
                Some(x) => get_last_identifier(x),
            },
         "mi" | "msub" | "msup" | "msubsup" =>
         {
             let n = root.get_next_sibling();
             if n.is_some() {
                 let t = get_last_identifier(n.unwrap());
                 if t.is_some() {
                     return t;
                 }
             }
             if &root.get_name() == "mi" { Some(root) }
             else {
                 get_first_identifier_helper(root)
             }
         },
        _ => {
             let n = root.get_next_sibling();
             if n.is_some() {
                 let t = get_last_identifier(n.unwrap());
                 if t.is_some() {
                     return t;
                 }
             }
             match root.get_first_child() {
                 None => None,
                 Some(x) => get_last_identifier(x),
             }
        }
    }
}



fn find_end_of_identifier(from: Node) -> Node {
    let mut cur : Node = from.clone();
    let mut last : Node = from.clone();
    loop {
        match cur.get_next_sibling() {
            None => break,
            Some(x) => cur = x,
        }
        match &cur.get_name() as &str {
            "mi" => { /* println!("Okay with '{}'", cur.get_all_content()); */  last = cur.clone(); },
            "msub" | "msup" | "msubsup" =>
                if get_first_identifier_helper(cur.clone()).is_some() {
                    // println!("Okay with '{}'", cur.get_all_content());
                    last = cur.clone();
                } else {
                    break;
                },
            "mo" => match &cur.get_all_content() as &str {
                "" | "\u{2062}" /* invisible times */ => continue,
                _ => break,
            },
                
                /* if cur.get_all_content() != "" { 
                println!("Breaking with '{}'", cur.get_all_content()); // TODO: FIX - Breaks with seemingly ""
                break; } else {
                    println!("Continuing with '{}'", cur.get_all_content()); continue; }, */
            _ => break,
        }
    }
    return last;
}

