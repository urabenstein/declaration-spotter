use llamapun::patterns::Pattern as P;
use llamapun::data::{Document};
use senna::phrase::Phrase;

use libxml::tree::Node;
use libxml::xpath::Context;

// use std::io::stdout;
// use std::io::Write;

use mathanalyzer::*;


pub struct DeclarationSpotter<'t> {
    pattern: P<'t, &'t str, &'t str>,
}

#[derive(Clone, Copy)]
pub enum IdentifierType {
    nosequence,
    elliptic_nonrelated,
    elliptic_related,
}



#[derive(Clone)]
pub struct Declaration {
    pub mathnode: Node,
    pub var_start: Node,
    pub var_end: Node,
    pub restriction_start: Node,
    pub restriction_end: Node,
    pub sentence: Node,

    pub idtype: IdentifierType,
    pub mathnode_is_restriction: bool,
    pub is_universal: bool,
}


pub struct RawDeclaration {
    pub mathnode: Node,
    pub restriction_start: Node,
    pub restriction_end: Node,
    pub sentence: Node,
    pub definiens_notes : Vec<&'static str>,
    pub definiendum_notes : Vec<&'static str>,
    pub no_noun: bool,
    pub is_universal: bool,
}


pub fn get_simple_declaration_pattern() -> P<'static, &'static str, &'static str> {
    let p_indefinite_article= P::Or(vec![P::W("a"), P::W("an"), P::W("any"), P::W("some"), P::W("every"), P::W("each"), P::W("all")]);
    let p_indefinite_article_box = Box::new(p_indefinite_article.clone());
    let p_mathformula = P::W("MathFormula");
    let p_mf_marked = P::Marked("definiendum", vec![], Box::new(p_mathformula));

    // a prime number, any integer, ...
    let p_long_dfs = P::PhrS(Phrase::NP, true, p_indefinite_article_box.clone());
    let p_long_dfs_marked = P::MarkedExcl("definiens", vec!["long"], Box::new(p_long_dfs), 1, 0);

    // let p be a prime number
    let p_let_pattern = P::Seq(vec![P::W("let"), p_mf_marked.clone(), P::W("be"), p_long_dfs_marked.clone()]);

    // p is a prime number
    let p_mf_is = P::Seq(vec![p_mf_marked.clone(), P::W("is"), p_long_dfs_marked.clone()]);

    // a prime number p
    let p_a_np_mf = P::PhrSE(Phrase::NP, false, p_indefinite_article_box, Box::new(p_mf_marked.clone()));
    let p_a_np_mf_marked = P::MarkedExcl("definiens", vec![], Box::new(p_a_np_mf), 1, 1);

    // for $p \in P$ followed by something either an NP or something indicating that the
    // declaration is complete.
    let p_short = P::Seq(vec![P::Or(vec![P::W("for"), P::W("let"), P::Seq(vec![P::W("for"), p_indefinite_article.clone()])]),
                         p_mf_marked.clone(),
                         P::Or(vec![p_long_dfs_marked.clone(), P::W(","), P::W("."), P::W("and")])]);


    // there is a prime number p
    let p_there = P::W("there");
    let p_existential_q = P::Or(vec![P::W("exists"), P::W("is")]);

    let pattern_existential = P::Marked("declaration", vec!["existential"],
                                        Box::new(P::Seq(vec![p_there, p_existential_q, p_a_np_mf_marked.clone()])));

    let pattern_universal = P::Marked("declaration", vec!["universal"],
                                      Box::new(P::Or(vec![p_let_pattern, p_a_np_mf_marked, p_mf_is, p_short])));


    P::Or(vec![pattern_existential, pattern_universal])
}


pub fn get_declarations(document: &mut Document, pattern: &P<&'static str, &'static str>) -> Vec<RawDeclaration> {
    let mut results : Vec<RawDeclaration> = Vec::new();
    let xpath_context = Context::new(&document.dom).unwrap();

    for sentence in document.annotated_sentence_iter() {
        let matches = P::match_sentence(&sentence, &pattern);
        for match_ in &matches {
            let sentence_node = sentence.node.as_ref().unwrap();
            let sentence_id = sentence_node.get_property("id").unwrap();
            let mut var_pos = 0;
            let mut restr_start = 0;
            let mut restr_end = 0;
            let mut definiens_notes : Option<Vec<&str>> = None;
            let mut definiendum_notes : Option<Vec<&str>> = None;
            let mut is_universal = true;
            for mark in &match_.marks {
                if mark.marker == "definiendum" {
                    var_pos = mark.offset_start;
                    definiendum_notes = Some(mark.notes.clone());
                } else if mark.marker == "definiens" {
                    restr_start = mark.offset_start;
                    restr_end = mark.offset_end;
                    definiens_notes = Some(mark.notes.clone());
                } else if mark.marker == "declaration" {
                    if mark.notes.contains(&"existential") {
                        is_universal = false;
                    } else {
                        assert!(mark.notes.contains(&"universal"));
                    }
                }

            }
            let variable_node = xpath_context.evaluate(&format!("(//span[@id='{}']//span[@class='word'])[{}]", sentence_id, var_pos+1)).unwrap().get_nodes_as_vec()[0].clone();
            let r_start_node = xpath_context.evaluate(&format!("(//span[@id='{}']//span[@class='word'])[{}]", sentence_id, restr_start+1)).unwrap().get_nodes_as_vec()[0].clone();
            let r_end_node = xpath_context.evaluate(&format!("(//span[@id='{}']//span[@class='word'])[{}]", sentence_id, restr_end)).unwrap().get_nodes_as_vec()[0].clone();
            results.push(RawDeclaration {
                mathnode: variable_node,
                restriction_start: r_start_node,
                restriction_end: r_end_node,
                sentence: sentence_node.clone(),
                definiens_notes : definiens_notes.unwrap(),
                definiendum_notes : definiendum_notes.unwrap(),
                no_noun : restr_end <= restr_start,
                is_universal: is_universal,
            });
        }
    }
    results
}

pub fn naive_raw_to_quad(raw: &Vec<RawDeclaration>) -> Vec<Declaration> {
    raw.iter().map(|r| Declaration {
        mathnode: r.mathnode.clone(),
        var_start: r.mathnode.clone(),
        var_end: r.mathnode.clone(),
        sentence: r.sentence.clone(),
        restriction_start: r.restriction_start.clone(),
        restriction_end: r.restriction_end.clone(),
        idtype: IdentifierType::nosequence,
        mathnode_is_restriction : false,
        is_universal: r.is_universal,
    }).collect()
}


fn get_math_child(word: Node) -> Option<Node> {
    let child = word.get_first_child();
    match child {
        Some(c) => 
            match &c.get_name() as &str {
                "math" => Some(c),
                _ => None,
            },
        None => None,
    }
}

pub fn first_identifier_purifier(raw: &Vec<RawDeclaration>) -> Vec<Declaration> {
    let mut result : Vec<Declaration> = Vec::new();

    for r in raw {
        let math_node_option = get_math_child(r.mathnode.clone());
        if math_node_option.is_none() {
            println!("Warning: Found mathformula, but not containing <math>...<math>");
            continue;
        }
        let math_node = math_node_option.unwrap();
        let identifiers = find_potential_identifiers(math_node);
        for id in &identifiers {
            if id.tags.contains(&IdentifierTags::First) {
                result.push(Declaration {
                    mathnode: r.mathnode.clone(),
                    var_start: id.start.clone(),
                    var_end: id.end.clone(),
                    restriction_start: r.restriction_start.clone(),
                    restriction_end: r.restriction_end.clone(),
                    sentence: r.sentence.clone(),
                    idtype: IdentifierType::nosequence,
                    mathnode_is_restriction : false,
                    is_universal: r.is_universal,
                });
                break;
            }
        }
        println!("Didn't find first identifier");
    }

    result
}

pub fn sequence_purifier(raw: &Vec<RawDeclaration>) -> Vec<Declaration> {
    let mut result : Vec<Declaration> = Vec::new();

    for r in raw {
        let math_node_option = get_math_child(r.mathnode.clone());
        if math_node_option.is_none() {
            println!("Warning: Found mathformula, but not containing <math>...<math>");
            continue;
        }
        let math_node = math_node_option.unwrap();
        let identifiers = find_potential_identifiers(math_node.clone());

        let mut first_identifier : Option<(Node, Node)> = None;
        let mut is_rel_seq = false;
        let mut is_ell_seq = false;
        let mut seq_start : Option<Node> = None;
        let mut seq_end : Option<Node> = None;
        let mut first_structured_identifier : Option<(Node, Node)> = None;
        for id in &identifiers {
            if id.tags.contains(&IdentifierTags::First) {
                first_identifier = Some((id.start.clone(), id.end.clone()));
            }
            if id.tags.contains(&IdentifierTags::FirstSeq) {
                if seq_start.is_none() {
                    seq_start = Some(id.start.clone());
                    if id.tags.contains(&IdentifierTags::RelSeq) {
                        is_rel_seq = true;
                    }
                    if id.tags.contains(&IdentifierTags::Ellipsis) {
                        is_ell_seq = true;
                    }
                }
                seq_end = Some(id.end.clone());
            }
            if id.tags.contains(&IdentifierTags::Structured) {
                first_structured_identifier = Some((id.start.clone(), id.end.clone()));
            }
        }
        let last_node = get_last_identifier(math_node.clone());


        // OPTION 1: is_ell_seq
        if is_ell_seq {
            result.push(Declaration {
                mathnode: r.mathnode.clone(),
                var_start: seq_start.unwrap(),
                var_end: seq_end.as_ref().unwrap().clone(),
                restriction_start: if !r.no_noun { r.restriction_start.clone() } else { r.mathnode.clone() },
                restriction_end: if !r.no_noun { r.restriction_end.clone() } else { r.mathnode.clone() },
                sentence: r.sentence.clone(),
                idtype: if is_rel_seq { IdentifierType::elliptic_related } else { IdentifierType::elliptic_nonrelated },
                mathnode_is_restriction : !r.no_noun && last_node.is_some() && last_node.as_ref().unwrap() != seq_end.as_ref().unwrap(),
                is_universal: r.is_universal,
            });

        // OPTION 2: !is_rel_seq   (MISSING: is sequence, and plural restriction)
        } else if seq_start.is_some() && !is_rel_seq {
            for id in &identifiers {
                if id.tags.contains(&IdentifierTags::FirstSeq) {
                    result.push(Declaration {
                        mathnode: r.mathnode.clone(),
                        var_start: id.start.clone(),
                        var_end: id.end.clone(),
                        restriction_start: if !r.no_noun { r.restriction_start.clone() } else { r.mathnode.clone() },
                        restriction_end: if !r.no_noun { r.restriction_end.clone() } else { r.mathnode.clone() },
                        sentence: r.sentence.clone(),
                        idtype: IdentifierType::nosequence,
                        mathnode_is_restriction : !r.no_noun && last_node.is_some() && last_node.as_ref().unwrap() != &id.end,
                        is_universal: r.is_universal,
                    });
                }
            }

        // OPTION 3: first_structured_identifier.is_some()
        } else if first_structured_identifier.is_some() {
            result.push(Declaration {
                mathnode: r.mathnode.clone(),
                var_start: first_structured_identifier.as_ref().unwrap().0.clone(),
                var_end: first_structured_identifier.as_ref().unwrap().1.clone(),
                restriction_start: if !r.no_noun { r.restriction_start.clone() } else { r.mathnode.clone() },
                restriction_end: if !r.no_noun { r.restriction_end.clone() } else { r.mathnode.clone() },
                sentence: r.sentence.clone(),
                idtype: IdentifierType::nosequence,
                mathnode_is_restriction : !r.no_noun && last_node.is_some() && last_node.as_ref().unwrap() != &first_structured_identifier.unwrap().1,
                is_universal: r.is_universal,
            });


        // OPTION 4: first_identifier.is_some()
        } else if first_identifier.is_some() {
            result.push(Declaration {
                mathnode: r.mathnode.clone(),
                var_start: first_identifier.as_ref().unwrap().0.clone(),
                var_end: first_identifier.as_ref().unwrap().1.clone(),
                restriction_start: if !r.no_noun { r.restriction_start.clone() } else { r.mathnode.clone() },
                restriction_end: if !r.no_noun { r.restriction_end.clone() } else { r.mathnode.clone() },
                sentence: r.sentence.clone(),
                idtype: IdentifierType::nosequence,
                mathnode_is_restriction : !r.no_noun && last_node.is_some() && last_node.as_ref().unwrap() != &first_identifier.unwrap().1,
                is_universal: r.is_universal,
            });

        }
    }


    result
}

