extern crate llamapun;
extern crate senna;
extern crate libxml;
extern crate regex;

use llamapun::data::{Document};

use libxml::tree::Node;
//use libxml::xpath::Context;
//use llamapun::patterns::Pattern as P;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::fs::OpenOptions;
use regex::Regex;
//use regex::RegexSet;



static INV_TIMES : &'static str = "\u{2062}";
static KAT_EXPORT_FILE : &'static str = "kat_annotations.xml";

static KAT_QE : &'static str = "KAT_1_QuantityExpression";

static SI_PREFIX : &'static [&'static str] = &["da","h","k","M","G","T","P","E","Z","Y","d","c","m","µ", "\u{03BC}", "\u{00B5}","n","p","f","a","z","y"];

static UNIT_SYMBOLS : &'static [&'static str] = &["m","g","s","A","K","mol","cd", /* SI-units with g instead of kg */
    "rad","sr","Hz","N","Pa","J","W","C","V","F","S","Wb","T","H","lm","lx","Bq","Gy","Sv","kat", /* SI coherent derived units; Ohm and degree C are still missing */
    "G", "eV", "pc"]; /*others: so far G:Gauß; eV:electron Volt; pc:parsec */


static DEBUG : bool = false;


#[derive(Clone)]
pub struct PossQE {
    pub node_name : String,
    pub node_id : String,
    pub node_content : String,
    pub exp : f64,
}

pub fn first_try(document : &mut Document) {
    // evaluate_text(document);
    // evaluate_math(document);

    for p in document.paragraph_iter(){
        let root = p.dnm.root_node;
        //this is the real root node, not just the one of the paragraph
     //   print_document(Some(root.clone()), String::new());
        walk_through_document(Some(root));
        break;
    }

    /*
    let opt_dnm = document.dnm;
    if opt_dnm.is_none(){
        return;
    }
    let dnm = opt_dnm.unwrap();
    let root = dnm.root_node;

    walk_through_document(Some(root));
    */
}

pub fn walk_through_document(opt_node : Option<Node>){
    if opt_node.is_none(){
        return;
    }
    let node = opt_node.unwrap();

    if node.get_name().eq("math"){
        let res = math_ends_with_mn(Some(node.clone()));
        if res.is_some(){
            if node.get_next_sibling().is_none(){
                return;
            }
            let sib = node.get_next_sibling().unwrap();
            if sib.get_name().eq("text"){
                let content = sib.get_content();
                let mut split = content.split_whitespace();
                let string = format!("{} {}",res.unwrap() ,content);

                //TODO match the string against a regexp

            }
          // check if the next sibling is a text node and starts with a unit expression
        }
    }

    walk_through_document(node.get_first_child());
    walk_through_document(node.get_next_sibling());


}

pub fn math_ends_with_mn(math_node : Option<Node>) -> Option<f64>{
    if math_node.is_none(){
        return None;
    }
    let node = math_node.unwrap();

    match &node.get_name() as &str{
        "math" | "semantics" => math_ends_with_mn(node.get_first_child()),
        "mrow" => {
            let sib = node.get_next_sibling();
            if sib.is_some() && sib.unwrap().get_name().eq("mrow") {
                return math_ends_with_mn(node.get_next_sibling());
            }
            return math_ends_with_mn(node.get_first_child());
        },
        "msub" | "mi" | "mo" | "msup" => math_ends_with_mn(node.get_next_sibling()),
        "mn" => {
            if node.get_next_sibling().is_some(){
                math_ends_with_mn(node.get_next_sibling());
            }
            let content = node.get_all_content().parse::<f64>();
            if content.is_err() {
                return None;
            }
            return Some(content.unwrap());
        },
        _ => None,
    }
}


pub fn print_document(opt_node : Option<Node>, space : String){
    if opt_node.is_none(){
        return;
    }

    let node = opt_node.unwrap();

    println!("{} {} {} {}",space.len(), space, node.get_name(), node.get_content());

    let more_space = format!("{} ",space);

    print_document(node.get_first_child(), more_space);
    print_document(node.get_next_sibling(), space);

}

pub fn get_number_prefix_unit_regexp() -> Regex{
    let number : String = r"\d*\.?\d+".to_string();

    let mut prefix_regex : String = r"".to_string();
    let mut unit_regex : String = r"".to_string();

    for p in SI_PREFIX{
        prefix_regex = format!("{}|{}", prefix_regex, p);
    }
    prefix_regex.remove(0);

    for u in UNIT_SYMBOLS{
        unit_regex = format!("{}|{}", unit_regex, u);
    }
    unit_regex.remove(0);
    let combined : String = format!(r"({})\s(({})?({})\s?)+(\s|\)|\.|,)",number, prefix_regex, unit_regex);
    // searches for number (prefix? unit)+ and a final character at the end (whitespace, closing bracket, dot, comma)
    let combined_regex = Regex::new(&combined).unwrap();
    combined_regex
}


pub fn evaluate_text(document : &mut Document){
    // let simple_pattern : P<'static, &'static str, &'static str> = P::W("GeV");

    for sentence in document.sentence_iter(){
        let ref text = sentence.range.dnm.plaintext;

        let combined_regex = get_number_prefix_unit_regexp();

        for cap in combined_regex.captures_iter(text){
            let mut res = cap.at(0).unwrap_or("").to_string();
            let res_len = res.len();
            res.truncate(res_len - 1);

            println!("{}",res);
        }


        //for res in text.matches("[0-9] GeV"){
        //    println!("{}", res);
       // }

        break;


    }
}

pub fn evaluate_math(document : &mut Document) {
    //let xpath_context = Context::new(&document.dom).unwrap();

    //println!("path {}",document.path);

    let ref context = document.xpath_context;

    let res = context.evaluate("//math");

    create_kat_annotations_header();

    if res.is_err(){
        return;
    }

    let res_vec : Vec<Node> = match res{
        Ok(object) => object.get_nodes_as_vec(),
        Err (_) => Vec::new(),
    };

    for node in res_vec{

        let pattern1 : [&str; 3] = ["mn","mo","mtext"];
        let pattern2 : [&str; 5] = ["mn", "mo", "mi", "mo", "mtext"];
        let pattern3 : [&str; 11] = ["mn", "mo", "mi", "mo", "mi", "mo", "mi", "mo", "mi", "mo", "mi"];


        // pattern_spotter(Some(node.clone()),&pattern2, &mut vec2, check_result_pattern2);

        let mut leafs = Vec::new();
        leafs = leafs_of_math_tree(Some(node.clone()), leafs);


        for leaf in leafs.clone(){
            if DEBUG {
                println!("{} {}", leaf.get_name(), leaf.get_all_content());
            }
        }

        pattern_spotter_leafs(&leafs, &pattern1, &mut Vec::new(), check_result_pattern1);
        pattern_spotter_leafs(&leafs, &pattern2, &mut Vec::new(), check_result_pattern2);
        pattern_spotter_leafs(&leafs, &pattern3, &mut Vec::new(), check_result_pattern3);

        if DEBUG {
            println!("");
        }
    }


    end_kat_document();

}

pub fn leafs_of_math_tree (opt_node : Option<Node>, mut res : Vec<Node>) -> Vec<Node>{
    if opt_node.is_none(){
        return res;
    }
    let node : Node = opt_node.unwrap();

    match &node.get_name() as &str{
        "mrow" | "mpadded" | "math" | "semantics" =>{
            res = leafs_of_math_tree(node.get_first_child(), res);
            return leafs_of_math_tree(node.get_next_sibling(), res);
        },
        "mo" | "mi" | "mn" | "mtext" => {
            res.push(node.clone());
            return leafs_of_math_tree(node.get_next_sibling(), res);
        },
        "msup" => {
            //if both children are of type mn, then return the result of the power operation in the form of an mn node
            //alternatively if child1 : mn and child2 : mrow
            //with child2 having again 2 kids, one of type mo (with content -) and one of mn
            //then compute also compute the result of the operation
            if node.get_first_child().is_none() {
                return res;
            }
            let child1 = node.get_first_child().unwrap();
            if child1.get_next_sibling().is_none() {
                return res;
            }
            let child2 = child1.get_next_sibling().unwrap();

            if child1.get_name().eq("mn") && child2.get_name().eq("mn"){
               //naturally these should both be numbers...
                let res1 = child1.get_all_content().parse::<f64>();
                let res2 = child2.get_all_content().parse::<f64>();

                if !res1.is_ok() || !res2.is_ok(){
                    return res;
                }

                let c1 = res1.unwrap();
                let c2 = res2.unwrap();

                if child1.get_first_child().is_none() {
                    return res;
                }
                child1.get_first_child().unwrap().set_content(&c1.powf(c2).to_string());
          //      println!("replaced {} ^ {} by  {}", c1, c2, c1.powf(c2));
                res.push(child1.clone());
            }else if child1.get_name().eq("mn") && child2.get_name().eq("mrow"){
                let res1 = child1.get_all_content().parse::<f64>();
                //check the kids of child2
                if child2.get_first_child().is_none() {
                    return res;
                }
                let kid1 = child2.get_first_child().unwrap();
                if kid1.get_next_sibling().is_none() {
                    return res;
                }
                let kid2 = kid1.get_next_sibling().unwrap();

                if kid1.get_name().eq("mo") && kid1.get_all_content().eq("-") && kid2.get_name().eq("mn"){
                    let res2 = kid2.get_all_content().parse::<f64>();
                    if !res1.is_ok() || !res2.is_ok(){
                        return res;
                    }
                    let c1 = res1.unwrap();
                    let c2 = res2.unwrap();

                    if child1.get_first_child().is_none() {
                        return res;
                    }
                    child1.get_first_child().unwrap().set_content(&c1.powf((-1.0)*c2).to_string());
                    //     println!("replaced {} ^ {} by  {}", c1, (-1.0 *c2), c1.powf((-1.0)*c2));
                    res.push(child1.clone());
                }
            }else{
                res.push(node.clone());
            }

            return leafs_of_math_tree(node.get_next_sibling(), res);

        },
        "msub" => {
            if node.get_first_child().is_none() {
                return res;
            }
            let child1 = node.get_first_child().unwrap();
            if child1.get_next_sibling().is_some(){
                let child2 = child1.get_next_sibling().unwrap();
                child1.set_content(&(child1.get_all_content() + "_" + &child2.get_all_content()));
            }
            res.push(child1.clone());

            return leafs_of_math_tree(node.get_next_sibling(), res);

        }
        _ => return res,
    }
}


pub fn pattern_spotter_leafs <'a> (leafs : &'a[Node], pattern : &'a[&str], res : &mut Vec<PossQE>,
                              func : fn(vec : &Vec<PossQE>) -> ())
{
    let index: usize = res.len();

    if leafs.len() == 0 {
        if index > 0 {
            func(res);
        }
        return;
    }

    let life_time_prob = leafs[0].get_first_child().unwrap();
    let mut exp : f64 = 1.0;
    let ref head;
    if leafs[0].get_name().eq("msup") {
        if leafs[0].get_first_child().is_none() {
            return;
        }
        if leafs[0].get_first_child().unwrap().get_next_sibling().is_none() {
            return;
        }
        let child1 = &leafs[0].get_first_child().unwrap();
        let child2 = child1.get_next_sibling().unwrap();

        if child2.get_name().eq("mn"){
            let poss_exp = child2.get_all_content().parse::<f64>();
            if poss_exp.is_err(){
                return;
            }
            exp = poss_exp.unwrap();


        }else{
            if DEBUG {
                println!("unknown pattern found msup with no mn as second child");
            }
            return;
        }
        head = &life_time_prob;
    }else{
        head = &leafs[0];
    }

    if head.get_name().eq(pattern[index]) {
        let opt_id = head.get_property("id");

        if opt_id.is_none() {
            println!("error node {} without id", head.get_name());
            return;
        }

        let poss_qe = PossQE { node_name : pattern[index].to_string() , node_id: opt_id.unwrap(), node_content: head.get_all_content(), exp: exp };
        res.push(poss_qe);

        if (index + 1) == pattern.len() {
            func(res);
            res.clear();
            pattern_spotter_leafs(&leafs[1..], pattern, res, func);
        } else {
            pattern_spotter_leafs(&leafs[1..], pattern, res, func);
        }
    }else{
        if index > 0 {
            func(res);
            res.clear();
            pattern_spotter_leafs(&leafs, pattern, res, func);
        }else {
            res.clear();
            pattern_spotter_leafs(&leafs[1..], pattern, res, func);
        }
    }

}



pub fn poss_qe_to_string(vec : &[PossQE]) -> String{
    let mut string = String::from("");
    for poss_qe in vec{
        string = string + &poss_qe.node_content;
        if !(poss_qe.exp == 1.0){
            string = string + "^" + &poss_qe.exp.to_string();
        }
    }
    return string;

}

pub fn check_result_pattern1(vec : &Vec<PossQE>){
    let string = poss_qe_to_string(&vec);

    if !(vec.len() == 3) || !vec[1].node_content.eq(INV_TIMES) {
        if DEBUG {
            println!("{} is not valid for pattern1", string);
        }
        return;
    }

    if !check_unit_string(&vec[2].node_content.clone(), vec){
        return;
    }

    add_qe_to_kat(vec);

    return;

}

pub fn check_result_pattern2(vec : &Vec<PossQE>) {

    let string = poss_qe_to_string(&vec);

    if !(vec.len() == 5) || !vec[1].node_content.eq(INV_TIMES) || !vec[3].node_content.eq(INV_TIMES){
        if DEBUG {
            println!("{} is not valid for pattern2", string);
        }
        return;
    }

    if !check_unit_string(&(vec[2].node_content.clone() + &vec[4].node_content.clone()), vec){
        return;
    }

    add_qe_to_kat(vec);

    return;


}

pub fn check_result_pattern3(vec : &Vec<PossQE>){
    let string = poss_qe_to_string(&vec);

    let mut i = 1;
    let mut s = String::new();
    while i < vec.len(){
        if !vec[i].node_content.eq(INV_TIMES) {
            if DEBUG {
                println!("{} is not valid for pattern1", string);
            }
            return;
        }
        if i+1 < vec.len() {
            s.push_str(&vec[i+1].node_content)
        }
        i = i + 2;
    }

    //println!("{}", string);
    //println!("pattern 3 constructs {}", s);

    if !check_unit_string(&s, vec){
        return;
    }

    add_qe_to_kat(vec);

    return;

}


pub fn check_unit_string(poss_unit : &str, vec : &Vec<PossQE>) -> bool{




    let mut unit = "";
    let mut prefix = "";


    for u in UNIT_SYMBOLS{
        if poss_unit.ends_with(u) {
            unit = u;
            break;
        }
    }

    if unit == "" {
        return false;
    }

    let fst_part = poss_unit.split_at(poss_unit.len()-unit.len()).0;

    for p in SI_PREFIX{
        if fst_part.eq(&p.to_string()){
            prefix = p;
            break;
        }
    }

    let mut exp = 1.0;
    let mut found = false;
    for node in vec{
        if node.exp == 1.0{
            continue;
        }
        if found && DEBUG{
            println!("multiple exps found for unit {} ", poss_unit);
        }
        exp = node.exp;
        found = true;
    }

    if poss_unit.eq(&(prefix.to_string()+unit)) {
        println!("string {} consists of prefix {} and unit {} with exp {}", poss_unit, prefix, unit, exp);
        println!("QE {}", poss_qe_to_string(vec));
    }else{
        println!("Could not disassemble {}", poss_qe_to_string(vec));
    }


    return true;
}


pub fn next_sibling_is (old_node : Node, str : &str) -> bool {
    let sibling = old_node.get_next_sibling();
    match sibling{
        Some(node) => {
            if node.get_name().eq(str) {
                return true;
            }
        },
        None => return false,

    }
    false
}

pub fn create_kat_annotations_header(){
    let mut kat_string = String::new();

    //add the header of the KAT-Format

    kat_string.push_str("<rdf:RDF xmlns:kat=\"https://github.com/KWARC/KAT/\" xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\" xmlns:d=\"http://kwarc.info/semanticextraction/KAnnSpec#\">\n");

    kat_string.push_str("<rdf:Description><kat:annotation rdf:nodeID=\"kat_run\"></kat:annotation></rdf:Description>\n");

    kat_string.push_str("<rdf:Description rdf:nodeID=\"kat_run\"><rdf:type rdf:resource=\"kat:run\"></rdf:type><kat:date rdf:datatype=\"xs:dateTime\">2016-11-17T10:18:33.264Z</kat:date><kat:tool>KAT</kat:tool><kat:runid>0</kat:runid></rdf:Description>\n");

    kat_string.push_str("<rdf:Description><kat:annotation rdf:nodeID=\"");
    kat_string.push_str(KAT_QE);
    kat_string.push_str("\"></kat:annotation></rdf:Description>\n");

    kat_string.push_str("<rdf:Description rdf:nodeID=\"");
    kat_string.push_str(KAT_QE);
    kat_string.push_str("\"><rdf:type rdf:resource=\"kat:kannspec\"></rdf:type><kat:kannspec-name>QuantityExpression</kat:kannspec-name><kat:kannspec-uri>http://localhost:3000/KAnnSpecs/units-annotations.xml</kat:kannspec-uri></rdf:Description>\n");

    let path = Path::new(KAT_EXPORT_FILE);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(file) => file,
    };

    match file.write_all(kat_string.as_bytes()) {
        Err(why) => {
            panic!("couldn't write to {}: {}", display,
                                               why.description())
        },
        Ok(_) => print!(""),
    }


}

pub fn add_qe_to_kat(vec : &[PossQE]){
    let mut file =
    OpenOptions::new()
        .write(true)
        .append(true)
        .open(KAT_EXPORT_FILE)
        .unwrap();

    let mut res = String::new();

    res.push_str("<rdf:Description rdf:nodeID=\"KAT_");
    res.push_str(&vec[0].node_id);
    res.push_str("\"><kat:run rdf:nodeID=\"kat_run\"></kat:run><kat:kannspec rdf:nodeID=\"");
    res.push_str(KAT_QE);
    res.push_str("\"></kat:kannspec><kat:concept>QuantityExpression</kat:concept><kat:type rdf:resource=\"http://kwarc.info/semanticextraction/KAnnSpec#quantityexpression\"></kat:type>");
    res.push_str("<kat:annotates rdf:resource=\"http://localhost:3000/content/sample2.html#cse(");

    for i in 0..(vec.len()){
        let ref poss_qe = vec[i];
        res.push_str("%2F%2F*%5B%40id%3D\'");
        res.push_str(&poss_qe.node_id);
        res.push_str("\'%5D");
        if i < (vec.len() - 1) {
            res.push_str("%2C");
        }
    }

    res.push_str(")\"></kat:annotates><undefinedsymbolname>");

    //print only ascii characters
    for c in poss_qe_to_string(vec).as_bytes(){
        if *c < 128 {
            let s = String::from_utf8(vec! [*c]).unwrap();
            res.push_str(&s);
        }
    }

    //res.push_str(&poss_qe_to_string(vec));
    res.push_str("</undefinedsymbolname></rdf:Description>");

    if let Err(e) = writeln!(file, "{}", res) {
        println!("{}", e);
    }
}

pub fn end_kat_document(){
    let mut file =
    OpenOptions::new()
        .write(true)
        .append(true)
        .open(KAT_EXPORT_FILE)
        .unwrap();

    let mut res = String::new();

    res.push_str("</rdf:RDF>");


    if let Err(e) = writeln!(file, "{}", res) {
        println!("{}", e);
    }
}


