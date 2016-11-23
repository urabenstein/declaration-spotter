extern crate libxml;
extern crate spotter;
extern crate llamapun;


//use std::io::stdout;
//use std::io::Write;
use std::env;

use llamapun::data::Corpus;

use spotter::*;

pub fn main() {
    let corpus = Corpus::new("tests/resources/".to_string());

    // let mut document = corpus.load_doc("tests/resources/1311.0066.annotated.xhtml".to_owned()).unwrap();
    let args : Vec<_> = env::args().collect();
    let mut document = corpus.load_doc(args[1].to_owned()).unwrap();
    // let dom = XMLDocument { doc_ptr : document.dom.doc_ptr.clone() };
    // let xpath_context;
    // {
    //     let dom = &document.dom;
    //     xpath_context = Context::new(dom).unwrap();
    // }


    // let mut kat_vec : Vec<DeclarationQuadruple> = Vec::new();

    first_try(&mut document);

    return;

}


