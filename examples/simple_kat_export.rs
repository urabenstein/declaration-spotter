extern crate libxml;
extern crate spotter;
extern crate llamapun;


//use std::io::stdout;
//use std::io::Write;
use std::env;

use llamapun::data::Corpus;

use spotter::*;

pub fn main() {
    let corpus = Corpus::new("/home/ulrich/Documents/uni/11/Masterarbeit/Semanticextraction/PapersFromCortex/small_eval/".to_string());

    /*
    for mut document in corpus.iter(){
        println!("{}", document.path);
        first_try(&mut document);
    }
    */

    let args : Vec<_> = env::args().collect();
    let mut document = corpus.load_doc(args[1].to_owned()).unwrap();
    first_try(&mut document);

    //

    return;

}


