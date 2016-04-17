extern crate llamapun;
extern crate libxml;
extern crate declarationspotter;

use std::io::stdout;
use std::io::Write;


use llamapun::data::Corpus;

use declarationspotter::spotter::*;
use declarationspotter::kat_export::*;

pub fn main() {
    let corpus = Corpus::new("tests/resources/".to_string());

    let mut document = corpus.load_doc("tests/resources/1311.0066.annotated.xhtml".to_owned()).unwrap();
    // let dom = XMLDocument { doc_ptr : document.dom.doc_ptr.clone() };
    // let xpath_context;
    // {
    //     let dom = &document.dom;
    //     xpath_context = Context::new(dom).unwrap();
    // }


    // let mut kat_vec : Vec<DeclarationQuadruple> = Vec::new();

    let pattern = get_simple_declaration_pattern();

    /* for mut sentence in document.annotated_sentence_iter() {
        let matches = Pattern::match_sentence(&sentence, &pattern);
        let ssent = sentence.senna_sentence.as_ref().unwrap();
        for match_ in &matches {
            // let xpath_context = Context::new(&document.dom).unwrap();
            let sentence_node = sentence.node.as_ref().unwrap();
            let sentence_id = sentence_node.get_property("id").unwrap();
            let mut var_pos = 0;
            let mut restr_start = 0;
            let mut restr_end = 0;
            for mark in &match_.marks {
                if mark.marker == "definiendum" {
                    var_pos = mark.offset_start;
                } else if mark.marker == "definiens" {
                    restr_start = mark.offset_start;
                    restr_end = mark.offset_end;
                }
            }
            let variable_node = xpath_context.evaluate(&format!("(//span[@id='{}']//span[@class='word'])[{}]", sentence_id, var_pos+1)).unwrap().get_nodes_as_vec()[0].clone();
            let r_start_node = xpath_context.evaluate(&format!("(//span[@id='{}']//span[@class='word'])[{}]", sentence_id, restr_start+1)).unwrap().get_nodes_as_vec()[0].clone();
            let r_end_node = xpath_context.evaluate(&format!("(//span[@id='{}']//span[@class='word'])[{}]", sentence_id, restr_end)).unwrap().get_nodes_as_vec()[0].clone();
            kat_vec.push(DeclarationQuadruple {
                variable: variable_node,
                restriction_start: r_start_node,
                restriction_end: r_end_node,
                sentence: sentence_node.clone(),
            });
        }
    } */
    let raw_declarations = get_declarations(&mut document, &pattern);
    // let pure_declarations = naive_raw_to_quad(&raw_declarations);
    // let pure_declarations = first_identifier_purifier(&raw_declarations);
    let pure_declarations = sequence_purifier(&raw_declarations);
    println!("{}", kat_export(&pure_declarations));
    stdout().flush().unwrap();
}


