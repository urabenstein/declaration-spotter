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


#[derive(Clone)]
pub struct Declaration {
    pub mathnode: Node,
    pub var_start: Node,
    pub var_end: Node,
    pub restriction_start: Node,
    pub restriction_end: Node,
    pub sentence: Node,
}


pub struct RawDeclaration {
    pub mathnode: Node,
    pub restriction_start: Node,
    pub restriction_end: Node,
    pub sentence: Node,
    pub definiens_notes : Vec<&'static str>,
    pub definiendum_notes : Vec<&'static str>,
}


pub fn get_simple_declaration_pattern() -> P<'static, &'static str, &'static str> {
    let p_indefinite_article= P::Or(vec![P::W("a"), P::W("an"), P::W("any"), P::W("some"), P::W("every")]);
    let p_indefinite_article_box = Box::new(p_indefinite_article);
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
    let p_a_np_mf = P::PhrSE(Phrase::NP, false, p_indefinite_article_box, Box::new(p_mf_marked));
    let p_a_np_mf_marked = P::MarkedExcl("definiens", vec![], Box::new(p_a_np_mf), 1, 1);


    let pattern = P::Or(vec![p_let_pattern, p_a_np_mf_marked, p_mf_is]);

    pattern
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
            for mark in &match_.marks {
                if mark.marker == "definiendum" {
                    var_pos = mark.offset_start;
                    definiendum_notes = Some(mark.notes.clone());
                } else if mark.marker == "definiens" {
                    restr_start = mark.offset_start;
                    restr_end = mark.offset_end;
                    definiens_notes = Some(mark.notes.clone());
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
                });
                break;
            }
        }
        println!("Didn't find first identifier");
    }

    result
}

