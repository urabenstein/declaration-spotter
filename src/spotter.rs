use llamapun::patterns::Pattern as P;
use llamapun::data::{Document};
use senna::phrase::Phrase;

use libxml::tree::Node;
use libxml::xpath::Context;
use libxml::tree::Document as xmlDocument;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::fs::OpenOptions;

// use std::io::stdout;
// use std::io::Write;

use mathanalyzer::*;


static inv_times : &'static str = "\u{2062}";
static kat_export_file : &'static str = "kat_annotations.xml";

static kat_QE : &'static str = "KAT_1_QuantityExpression";

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


#[derive(Clone)]
pub struct PossQE {
    pub node_id : String,
    pub node_content : String,
}


pub fn get_simple_declaration_pattern() -> P<'static, &'static str, &'static str> {
    let p_indefinite_article= P::Or(vec![P::W("a"), P::W("an"), P::W("any"), P::W("some"), P::W("every"), P::W("each"), P::W("all")]);
    let p_indefinite_article_box = Box::new(p_indefinite_article.clone());
    let p_mathformula = P::W("MathFormula");
    let p_mf_marked = P::Marked("definiendum", vec![], Box::new(p_mathformula));

    // a prime number, any integer, ...
    let p_long_dfs = P::PhrS(Phrase::NP, true, p_indefinite_article_box.clone());
    let p_long_dfs_marked = P::MarkedExcl("definiens", vec!["long"], Box::new(p_long_dfs), 0, 0);

    // let p be a prime number
    let p_let_pattern = P::Seq(vec![P::W("let"), p_mf_marked.clone(), P::W("be"), p_long_dfs_marked.clone()]);

    // p is a prime number
    let p_mf_is = P::Seq(vec![p_mf_marked.clone(), P::W("is"), p_long_dfs_marked.clone()]);

    // a prime number p
    let p_a_np_mf = P::PhrSE(Phrase::NP, false, Box::new(P::Seq(vec![p_indefinite_article.clone(), P::AnyW])),
                            Box::new(p_mf_marked.clone()));
    let p_a_np_mf_marked = P::MarkedExcl("definiens", vec![], Box::new(p_a_np_mf), 0, 1);

    // for $p \in P$ followed by something either an NP or something indicating that the
    // declaration is complete.
    let p_short = P::Seq(vec![P::Or(vec![P::Seq(vec![P::Or(vec![P::W("where"), P::W("with"), P::W("for")]),
                                                        p_indefinite_article.clone()]),
        P::W("for"), P::W("where"), P::W("with"), P::W("let"), P::W("suppose"), P::W("assume"), P::W("assuming"), P::W("given")]),
                         p_mf_marked.clone(),
                         P::Or(vec![p_long_dfs_marked.clone(), P::W(","), P::W("."), P::W("and")])]);

    // of degree d
    let p_short_dfs_no_article = P::Marked("definiens", vec![], Box::new(P::PhrE(Phrase::NP, false,
                                    Box::new(P::Seq(vec![P::AnyW, p_mf_marked.clone()])))));
    let p_of = P::Seq(vec![P::W("of"), p_short_dfs_no_article.clone()]);

    // there is a prime number p
    let p_there = P::W("there");
    let p_existential_q = P::Or(vec![P::W("exists"), P::W("is")]);

    let pattern_existential = P::Marked("declaration", vec!["existential"],
                                        Box::new(P::Seq(vec![p_there, p_existential_q, p_a_np_mf_marked.clone()])));

    let pattern_universal = P::Marked("declaration", vec!["universal"],
                                      Box::new(P::Or(vec![p_let_pattern, p_a_np_mf_marked, p_mf_is, p_short, p_of])));


    P::Or(vec![pattern_existential, pattern_universal])
}


pub fn first_try(document : &mut Document) {
    let xpath_context = Context::new(&document.dom).unwrap();

    println!("path {}",document.path);

    let ref context = document.xpath_context;

    let res = context.evaluate("//math");

    create_kat_annotations_header();

    if res.is_ok(){
        let res_vec : Vec<Node> = match res{
            Ok(object) => object.get_nodes_as_vec(),
            Err (_) => Vec::new(),
        };
        println!("{} elements", res_vec.len());

        for node in res_vec{

            let mut vec1 : Vec<PossQE> = Vec::new();
            let mut vec2 : Vec<PossQE> = Vec::new();
            //mn_mo_mtext_spotter(Some(node),  vec);
            let pattern1 : [&str; 3] = ["mn","mo","mtext"];
          //  pattern_spotter(Some(node.clone()), &pattern1, &mut vec1, check_result_pattern1);

            let pattern2 : [&str; 5] = ["mn", "mo", "mi", "mo", "mtext"];

           // pattern_spotter(Some(node.clone()),&pattern2, &mut vec2, check_result_pattern2);

            let mut leafs = Vec::new();
            leafs = leafs_of_math_tree(Some(node.clone()), leafs);



            for leaf in leafs.clone(){
                println!("{} {}", leaf.get_name(), leaf.get_all_content());
            }

          //  pattern_spotter_leafs(&leafs, &pattern1, &mut Vec::new(), check_result_pattern1);
          //  pattern_spotter_leafs(&leafs, &pattern2, &mut Vec::new(), check_result_pattern2);
          //  pattern_spotter_leafs(&leafs, &pattern3, &mut Vec::new(), check_result_pattern3);

            // pattern 3 is supposed to be a generic pattern, which should also capture
            // longer strings, which are not in mtext. I.e. G H z in math mode,
            // separated by invis. times (mo).
            for i in 1..10 {
                let mut pattern3 : Vec<&str> = Vec::new();
                pattern3.push("mn");

                for j in 1..i{
                    pattern3.push("mo"); pattern3.push("mi");
                }
            //    pattern_spotter_leafs(&leafs, &pattern3, &mut Vec::new(), check_result_pattern3);

            }


            println!("");
        }
    }


    end_kat_document();

    /*
    match document.dom.get_root_element() {
        Ok(t) => {
            let mut str = String::from("") ;
            print_node(t,&str,&document.dom);
        },
        Err(error) => println!("No root element.")
    }
    */

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
            //if both children are of type mn, then return the result of the power operation
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
            }

            if child1.get_name().eq("mn") && child2.get_name().eq("mrow"){
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

            }

            return leafs_of_math_tree(node.get_next_sibling(), res);


        }
        _ => return res,
    }
}

pub fn pattern_spotter_leafs <'a> (leafs : &[Node], mut pattern : &[&str], res : &'a mut Vec<PossQE>,
                              func : fn(vec : &Vec<PossQE>) -> (), total_res : &'a mut Vec<&'a mut Vec<PossQE>>)
                                                                        -> &'a mut Vec<&'a mut Vec<PossQE>>
{

    let mut index : usize = res.len();

    if leafs.len() == 0 {
        return total_res;
    }

    let ref head = leafs[0];

    if head.get_name().eq(pattern[index]) {

        let opt_id = head.get_property("id");

        if opt_id.is_none(){
            println!("error node {} without id", head.get_name());
            return total_res;
        }

        let poss_qe = PossQE {node_id : opt_id.unwrap(), node_content : head.get_all_content()};
        res.push(poss_qe);

        if (index + 1) == pattern.len(){
            func(res);
            let mut new_res : &'a mut Vec<PossQE> = &mut res.clone();
            total_res.push(&mut new_res);
 //           let mut new_res : &'a mut Vec<PossQE>  = &mut Vec::new();
            res.clear();
            return pattern_spotter_leafs(&leafs[1..], pattern, res, func, total_res);
        }else {
            return pattern_spotter_leafs(&leafs[1..], pattern, res, func, total_res);
        }
    }else{
        res.clear();
        return pattern_spotter_leafs(&leafs[1..], pattern, res, func, total_res);
    }

}

pub fn pattern_spotter (opt_node : Option<Node>, mut pattern : &[&str], res : &mut Vec<PossQE>, func : fn(vec : &Vec<PossQE>) -> ()) {

    let mut index : usize =  res.len();

    if index == pattern.len() {
        println!("index out of bounds");
    }

    if opt_node.is_none() {
        return;
    }

    if index > pattern.len(){
        return;
    }

    let node : Node = opt_node.unwrap();

    match &node.get_name() as &str{
        "text" =>
            return,
        "math" | "semantics" =>
            pattern_spotter(node.get_first_child(), pattern, res, func),
        "mrow" => {
            pattern_spotter(node.get_first_child(), pattern, &mut res.clone(), func);
            pattern_spotter(node.get_next_sibling(), pattern, &mut res.clone(), func);
        },
        "mpadded" => {
            pattern_spotter(node.get_first_child(), pattern, res, func);
            // pattern is now finished
            if res.len() == pattern.len(){
                pattern_spotter(node.get_next_sibling(), pattern, &mut Vec::new(), func);
            }
            //if not, try to finish it
            pattern_spotter(node.get_next_sibling(), pattern, res, func);
        },
        _ => {
            let ref element = pattern[index];
            if node.get_name().eq(element){

                let opt_id = node.get_property("id");

                if opt_id.is_none(){
                    println!("error node {} without id", node.get_name());
                    return;
                }

                let poss_qe = PossQE {node_id : opt_id.unwrap(), node_content : node.get_all_content()};
                res.push(poss_qe);

                if res.len() == pattern.len() {
                    func(res);
                    pattern_spotter(node.get_next_sibling(), pattern, &mut Vec::new(), func);
                }else {
                    pattern_spotter(node.get_next_sibling(), pattern, res, func);
                }
                if index > 0 {
                    pattern_spotter(node.get_next_sibling(), pattern, &mut Vec::new(), func);
                }
            }else{
                //restart search
                pattern_spotter(node.get_next_sibling(), pattern, &mut Vec::new(), func);
                pattern_spotter(node.get_first_child(), pattern, &mut Vec::new(), func);

            }

        }
    }

}


pub fn poss_qe_to_string(vec : &[PossQE]) -> String{
    let mut string = String::from("");
    for poss_qe in vec{
        string = string + &poss_qe.node_content;
    }
    return string;

}

pub fn check_result_pattern1(vec : &Vec<PossQE>){
    let string = poss_qe_to_string(&vec);

    assert! (vec.len() == 3);

    if !vec[1].node_content.eq(inv_times) {
        println!("{} is not valid for pattern1", string);
        return;
    }

    if !check_unit_string(&vec[2].node_content.clone()){
        return;
    }

    println!("{}", string);

    add_qe_to_kat(vec);

    return;

}

pub fn check_result_pattern2(vec : &Vec<PossQE>) {
    assert! (vec.len() == 5);

    let string = poss_qe_to_string(&vec);

    if !vec[1].node_content.eq(inv_times) || !vec[3].node_content.eq(inv_times){
        println!("{} is not valid for pattern2", string);
        return;
    }

    if !check_unit_string(&(vec[2].node_content.clone() + &vec[4].node_content.clone())){
        return;
    }

    println!("{}", string);

    add_qe_to_kat(vec);

    return;


}

pub fn check_result_pattern3(vec : &Vec<PossQE>){
    let string = poss_qe_to_string(&vec);

    let mut i = 1;
    let mut s = String::new();
    while i < vec.len(){
        if !vec[i].node_content.eq(inv_times) {
            println!("{} is not valid for pattern1", string);
            return;
        }
        if i+1 < vec.len() {
            s.push_str(&vec[i+1].node_content)
        }
        i = i + 2;
    }

    //println!("{}", string);
    //println!("pattern 3 constructs {}", s);

    if !check_unit_string(&s){
        return;
    }

    println!("{}", string);

    add_qe_to_kat(vec);

    return;

}

pub fn check_unit_string(poss_unit : &str) -> bool{


    //due to different representations µ appears three times...
    let prefixes : Vec<&str> =
        ["da","h","k","M","G","T","P","E","Z","Y","d","c","m","µ", "\u{03BC}", "\u{00B5}","n","p","f","a","z","y"].to_vec();

    //using g instead of kg here...
    let mut si_units : Vec<&str> = ["m","g","s","A","K","mol","cd"].to_vec();

    //TODO add ohm and degree C
    let mut si_coherent_derived_units : Vec<&str> =
        ["rad","sr","Hz","N","Pa","J","W","C","V","F","S","Wb","T","H","lm","lx","Bq","Gy","Sv","kat"].to_vec();

    //Gauß
    let mut others : Vec<&str> = ["G"].to_vec();

    let mut units : Vec<&str> = Vec::new();

    units.append(&mut si_units); units.append(&mut si_coherent_derived_units); units.append(&mut others);


    let mut unit = "";
    let mut prefix = "";


    for u in units{
        if poss_unit.ends_with(u) {
            unit = u;
            break;
        }
    }

    if unit == "" {
        return false;
    }

    let fst_part = poss_unit.split_at(poss_unit.len()-unit.len()).0;

    for p in prefixes{
        if fst_part.eq(p){
            prefix = p;
            break;
        }
    }

    println!("string {} consists of prefix {} and unit {}", poss_unit, prefix, unit);

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
    kat_string.push_str(kat_QE);
    kat_string.push_str("\"></kat:annotation></rdf:Description>\n");

    kat_string.push_str("<rdf:Description rdf:nodeID=\"");
    kat_string.push_str(kat_QE);
    kat_string.push_str("\"><rdf:type rdf:resource=\"kat:kannspec\"></rdf:type><kat:kannspec-name>QuantityExpression</kat:kannspec-name><kat:kannspec-uri>http://localhost:3000/KAnnSpecs/units-annotations.xml</kat:kannspec-uri></rdf:Description>\n");

    let path = Path::new(kat_export_file);
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
        .open(kat_export_file)
        .unwrap();

    let mut res = String::new();

    res.push_str("<rdf:Description rdf:nodeID=\"KAT_");
    res.push_str(&vec[0].node_id);
    res.push_str("\"><kat:run rdf:nodeID=\"kat_run\"></kat:run><kat:kannspec rdf:nodeID=\"");
    res.push_str(kat_QE);
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
        .open(kat_export_file)
        .unwrap();

    let mut res = String::new();

    res.push_str("</rdf:RDF>");


    if let Err(e) = writeln!(file, "{}", res) {
        println!("{}", e);
    }
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
            let mut restr_start = 1;
            let mut restr_end = 1;
            let mut definiens_notes : Option<Vec<&str>> = Some(vec![]);
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
            match xpath_context.evaluate(&format!("(//span[@id='{}']//span[@class='word'])[{}]",
                    sentence_id, var_pos+2)) {
                Err(_) => { },
                Ok(x) => {
                    if x.get_nodes_as_vec()[0].get_all_content() == "-" {
                        continue;  // senna's fault (MathFormula-algebra etc)
                    }
                }
            };
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
                println!("SEQ");
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
            println!("Option 2");
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
            if r.no_noun {
                let x =  first_identifier.as_ref().unwrap().1.get_next_sibling();
                if x.is_none() {
                    continue;
                } else if x.as_ref().unwrap().get_name() == "annotation" || x.as_ref().unwrap().get_name() == "annotation-xml" {
                    continue;
                }
            }
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

