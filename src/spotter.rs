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
    "G", "eV", "pc", "mag", "M_⊙"]; /*others: so far G:Gauß; eV:electron Volt; pc:parsec; mag: :(; M_⊙: solar mass*/


static DEBUG : bool = false;


#[derive(Clone)]
pub struct PossQE {
    pub node_name : String,
    pub node_id : String,
    pub node_content : String,
    pub exp : f64,
}

pub fn is_times_symbol(s : String) -> bool {
    return s.eq("×") || s.eq(INV_TIMES) || s.eq("⋅");
}

pub fn first_try(status : String, document : &mut Document) {

    create_kat_annotations_header();

    for p in document.paragraph_iter(){
        let root = p.dnm.root_node;
        if status.eq("nodes"){
            print_document(Some(root.clone()), String::new());
            continue;
        }
        pre_process(Some(root), true, status.clone());
    }

    if status.eq("nodes"){
        return;
    }

    evaluate_text(document, status.clone());
    //evaluate_math(document);

    end_kat_document();

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

/* Performs a number simplifying steps on the math nodes, by looking at their "leafs" (cmp. leafs_of_math_tree):
    - times expressions (i.e. 5 x 1000) are resolved to a single <mn> node
    - for expressions ending with i.e. 5 ± 1 or  ± 1 , this is recognized as a range and put into the following text node
        as "4 to 6" or "-1 to 1"
    - for expressions ending with number ("$x = 4$"), the number is copied to the following text
    - for math expressions consisting only of empty msup nodes ("{}^{10}"), the exponent is also copied to the following
      text as ^(10).
   The copying is done for cases where the number / formula / exponent is in math-mode, but the corresponding unit is in the
   following text, i.e. $ x = 4 $ cm $^2$ ~> $x = 4$ 4 cm^2 $^2$. Like this, the unit expression can be detected by a regex
   (cmp. evaluate_text). */
pub fn pre_process(opt_node : Option<Node>, start : bool, status : String){
    if opt_node.is_none(){
        return;
    }
    let node = opt_node.unwrap();

    // if the math node ends with a number i.e. $R = 5$, then copy that number to the following text, s.t.
    // one can detect $R = 5$ kg as $R = 5$ 5 kg.
    if node.get_name().eq("math"){

        let mut leafs = Vec::new();
        leafs = leafs_of_math_tree(Some(node.clone()), leafs);

        //resolve times expressions to a single <mn> node
        loop{
            let new_leafs = Vec::from(resolve_times(&mut leafs.clone()));
            if new_leafs.len() == leafs.len(){
                leafs = new_leafs;
                break;
            }else{
                leafs = new_leafs;
            }
        }

        /*
        println!("LEAF ");
        for leaf in leafs.clone(){
            if leaf.get_name().eq("msup"){
                println!("msup");
                let mut child = leaf.get_first_child();
                while child.is_some(){
                    let unwr = child.unwrap();
                    println!("\t {} {}", unwr.get_name(), unwr.get_all_content());
                    child = unwr.get_next_sibling();
                }

            }else {
                println!("{} {}", leaf.get_name(), leaf.get_all_content());
            }
        }

        print!("MATH ");
        for leaf in leafs.clone(){
            print!("{}",leaf.get_all_content())
        }
        println!(" ");
        println!(" ");
        */

        let len = leafs.len();

        if len > 0 {
            //check if leafs ends with mn mo mn
            // where mo = +-, hence e.g. 5 +- 1
            // in this case, detect a range (4 to 6)
            if len >= 3 && leafs[len-3].get_name().eq("mn") && leafs[len-2].get_name().eq("mo") &&
                leafs[len-1].get_name().eq("mn") && leafs[len-2].get_all_content().eq("±") {

                let number1_res = leafs[len-3].get_all_content().parse::<f64>();
                let number2_res = leafs[len-1].get_all_content().parse::<f64>();

                if number1_res.is_err() || number2_res.is_err(){
                    println!("mn with no number {} {}", leafs[len-3].get_all_content(), leafs[len-1].get_all_content());
                    return;
                }

                let number1 = number1_res.unwrap();
                let number2 = number2_res.unwrap();
                let range_min = number1 - number2;
                let range_max = number1 + number2;
                let new_content = format!("{} to {}", range_min, range_max);
                prefix_node_content(node.get_next_sibling(), &new_content);

                //see if the formula ends with +- mn, if so detect a range. e.g. +-1 -> -1 to +1
            }else if len >= 2 && leafs[len-1].get_name().eq("mn") && leafs[len-2].get_name().eq("mo") &&
                    leafs[len-2].get_all_content().eq("±"){
                let number_res = leafs[len-1].get_all_content().parse::<f64>();

                if number_res.is_err(){
                    println!("mn with no number {} ", leafs[len-1].get_all_content());
                    return;
                }

                let number = number_res.unwrap();
                let new_content = format!("{} to {}", -number, number);
                prefix_node_content(node.get_next_sibling(), &new_content);

                //if the last node is a number (mn), copy it to the following text, s.t.
                // a unit can be detected, if the unit symbol is in text only.
                // also detects e.g. 10^10, since this is automatically resolved to one number (mn)
            }else if leafs[len-1].get_name().eq("mn") {
                let number = leafs[len - 1].get_all_content();
                prefix_node_content(node.get_next_sibling(), &number);
            }else if len == 1 && leafs[0].get_name().eq("msup"){
                //replace cm$^2$ by a pure textual representation, i.e. cm^2
                let base_exp = msup_base_exp(leafs[0].clone());
                if base_exp.is_some(){
                    let be = base_exp.unwrap();
                    if be.0.eq(""){
                        let prefix = format!("^({})", be.1);
                        prefix_node_content(node.get_next_sibling(), &prefix);
                    }
                }
            }
        }
        if !status.eq("text") {
            evaluate_math(leafs.clone());
        }
    }

    pre_process(node.get_first_child(), false, status.clone());
    if !start {
        pre_process(node.get_next_sibling(), false, status.clone());
    }


}

//resolves mn mo mn, where mo is a times symbol to only one mn node with the result of the operation
pub fn resolve_times(leafs : &mut Vec<Node>) -> &[Node]{
    let len = leafs.len();
    for i in 0..len{
        if leafs[i].get_name().eq("mn") && i+2 < len && leafs[i+1].get_name().eq("mo") &&
            is_times_symbol(leafs[i+1].get_all_content()) && leafs[i+2].get_name().eq("mn"){
            let number1 = leafs[i].get_all_content().parse::<f64>().unwrap();
            let number2 = leafs[i+2].get_all_content().parse::<f64>().unwrap();
            let res_as_str = (number1 * number2).to_string();
            leafs[i].set_content(&res_as_str);
            leafs.remove(i+1);
            leafs.remove(i+1);
            //println!("{} times {} replaced by {}", number1, number2, res_as_str);
            return leafs.as_slice();
        }

    }
    return leafs.as_slice();

}

/* Attaches the prefix to the textual content of the node */
pub fn prefix_node_content(opt_node : Option<Node>, prefix : &str){
    if opt_node.is_none(){
        return;
    }
    let node = opt_node.unwrap();
    if node.get_name().eq("text"){
        let new_content = &format!(" {}{}", prefix, node.get_content());
        node.set_content(new_content);
    }

}

/* Returns the base and the exponent of a msup node */
pub fn msup_base_exp(msup_node : Node) -> Option<(String, String)> {
    if msup_node.get_first_child().is_none() {
        return None;
    }
    let child1 = msup_node.get_first_child().unwrap();
    if child1.get_next_sibling().is_none() {
        return None;
    }
    let child2 = child1.get_next_sibling().unwrap();

    let base = child1.get_all_content();
    let mut exp = String::new();

    if child2.get_name().eq("mrow") {
        //check the kids of child2
        if child2.get_first_child().is_none() {
            return None;
        }
        let kid1 = child2.get_first_child().unwrap();
        if kid1.get_next_sibling().is_none() {
            return None;
        }
        let kid2 = kid1.get_next_sibling().unwrap();

        if kid1.get_name().eq("mo") && kid1.get_all_content().eq("-") && kid2.get_name().eq("mn") {
            exp.push_str("-"); exp.push_str(&kid2.get_all_content());
        }
    }else{
        exp.push_str(&child2.get_all_content());
    }
    return Some((base,exp));
}


/* Calculates the value of an msup node, if both children are numbers, i.e. 10^(-1) ~> 0.1 */
pub fn calculate_msup(msup_node : Node) -> Option<String> {
    let base_exp = msup_base_exp(msup_node);
    if base_exp.is_none(){
        return None;
    }

    let b_e = base_exp.unwrap();

    let base = b_e.0.parse::<f64>();
    let exp  = b_e.1.parse::<f64>();

    if base.is_err() || exp.is_err(){
        return None;
    }
    return Some(base.unwrap().powf(exp.unwrap()).to_string());
}

/* Prints the nodes of the document in an user-friendly format with
    indents for the depth of the node */
pub fn print_document(opt_node : Option<Node>, space : String){
    if opt_node.is_none(){
        return;
    }

    let node = opt_node.unwrap();

    println!("{} {} {} {}",space.len(), space, node.get_name(), node.get_content());

    let more_space = format!("{} ",space);

    print_document(node.get_first_child(), more_space);
    if space.len() > 0 {
        print_document(node.get_next_sibling(), space);
    }

}

/* Returns a regex matching numbers (with "."), i.e. 0.123 */
pub fn get_number_regex_string() -> String{
    return r"\d*\.?\d+".to_string();
}


/* Returns a regex matching i.e. " 5 MHz.", where the match consists of
    - a whitespace
    - a number
    - whitespaces
    - an optional prefix
    - the unit symbol
    - a symbol for the end (whitespace . ) ,)
  */
pub fn get_number_prefix_unit_regexp() -> Regex{
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
    let combined : String = format!(r"\s({})\s*({})?({})(\s|\)|\.|,)",get_number_regex_string() , prefix_regex, unit_regex);
    // searches for number prefix? unit and a final character at the end (whitespace, closing bracket, dot, comma)
    let combined_regex = Regex::new(&combined).unwrap();
    combined_regex
}

/* Returns a regex matching i.e. " 5 MHz - 10 MHz", where the match consists of
    - a number
    - a range symbol (- or 'to')
    - another number
    - an optional prefix
    - the unit symbol
    - a symbol for the end (whitespace . ) ,)
  */
pub fn get_range_regexp() -> Regex{
    let number : String = r"-?\d*\.?\d+".to_string();

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
    let combined : String = format!(r"\s({}\s?(-|(to))\s?{})\s*({})?({})\s?(\s|\)|\.|,)",number, number, prefix_regex, unit_regex);
    //searches for ranges in text, i.e. 70 - 100 MeV
    let combined_regex = Regex::new(&combined).unwrap();
    combined_regex
}


/*
    Looks at the plaintext of the document (modified by pre_process), where "MathFormula" is deleted from the text
    and power expressions (10^10) are replaced by their result.
    After that, it is looking for quantity expressions and range expressions using regular expressions and prints them.
*/
pub fn evaluate_text(document : &mut Document, status : String){
    for sentence in document.sentence_iter(){
        let ref old_text = sentence.range.dnm.plaintext;

        //remove all the MathFormula words from the text
      //  let mathformula_regex = Regex::new(r"MathFormula").unwrap();
        //let mut text = &mathformula_regex.replace_all(old_text," ");
        let mut text = old_text.replace("MathFormula"," ");

        //search for number^(number) expressions and replace them by their result
        let number_power_string = format!(r"({})\s* \^\(({})\)",get_number_regex_string(), get_number_regex_string());
        println!("STRING {}", number_power_string);
        let number_power_regex = Regex::new(&number_power_string).unwrap();

        for cap in number_power_regex.captures_iter(&text.clone()){
            if cap.at(1).is_none() && cap.at(2).is_none(){
                continue;
            }
            let base = cap.at(1).unwrap();
            let exp  = cap.at(2).unwrap();
            let basef = base.parse::<f64>().unwrap();
            let expf  = exp.parse::<f64>().unwrap();
            let res = basef.powf(expf).to_string();
            text = text.replace(cap.at(0).unwrap(),&res);

            //println!("Total: {} |Base: {} |Exp: {} | Res: {}",cap.at(0).unwrap(), base, exp, res);
        }

        let combined_regex = get_number_prefix_unit_regexp();

        if status.eq("text"){
            println!("{}", text);
            return;
        }

        for cap in combined_regex.captures_iter(&text){
            let mut res = cap.at(0).unwrap_or("").to_string();
            let res_len = res.len();
            res.truncate(res_len - 1);

            println!("Unit in text: {}",res);

            unsafe {
                let pos = cap.pos(0).unwrap();
                let offset = 15;
                if pos.0 - offset > 0 && pos.1 + offset < text.len() {
                    println!("Context: {} \n", text.as_str().slice_unchecked(pos.0 - offset, pos.1 + offset));
                }
            }


        }

        let range_regex = get_range_regexp();

        for cap in range_regex.captures_iter(&text){

            let mut res = cap.at(0).unwrap_or("").to_string();
            let res_len = res.len();
            res.truncate(res_len - 1);

            println!("Range in text: {}",res);
        }
        break;
    }
}

/*
    Searches for the patterns listed below in the leafs of a math node. If a pattern is found, then the
    corresponding evaluation-function of that pattern is called on the text found, to verify that the
    text is also as expected.
    Also searches for degree-expressions in math-nodes.
*/
pub fn evaluate_math(leafs : Vec<Node>) {
        for leaf in leafs.clone(){
            if DEBUG {
                println!("{} {}", leaf.get_name(), leaf.get_all_content());
            }
        }

        let pattern1 : [&str; 3] = ["mn","mo","mtext"];
        let pattern2 : [&str; 5] = ["mn", "mo", "mi", "mo", "mtext"];
        let pattern3 : [&str; 11] = ["mn", "mo", "mi", "mo", "mi", "mo", "mi", "mo", "mi", "mo", "mi"];
        let pattern4 : [&str; 12] = ["mn", "mo", "mn", "mo", "mi", "mo", "mi", "mo", "mi", "mo", "mi", "mo"];

        pattern_spotter_leafs(&leafs, &pattern1, &mut Vec::new(), check_result_pattern1);
        pattern_spotter_leafs(&leafs, &pattern2, &mut Vec::new(), check_result_pattern2);
        pattern_spotter_leafs(&leafs, &pattern3, &mut Vec::new(), check_result_pattern3);
        pattern_spotter_leafs(&leafs, &pattern4, &mut Vec::new(), check_result_pattern4);

        find_degree(&leafs);


        if DEBUG {
            println!("");
        }
}

/*
    Returns the "leafs" of a math tree (well, not quite, but almost), i.e.
    mo, mi, mn and mtext nodes.
    For msup-nodes it returns either the result of the power operations if possible, else the whole node.
    For msub-nodes it returns the first child with the content of the second one attached i.e. M_s.

    mrow, mpadded, semantics and mstyle nodes are not returned.
*/
pub fn leafs_of_math_tree (opt_node : Option<Node>, mut res : Vec<Node>) -> Vec<Node>{
    if opt_node.is_none(){
        return res;
    }
    let node : Node = opt_node.unwrap();

    match &node.get_name() as &str{
        "math" => {
          return leafs_of_math_tree(node.get_first_child(), res);
        },
        "mrow" | "mpadded" | "semantics" | "mstyle" =>{
            res = leafs_of_math_tree(node.get_first_child(), res);
            return leafs_of_math_tree(node.get_next_sibling(), res);
        },
        "mo" | "mi" | "mn" | "mtext" => {
            res.push(node.clone());
            return leafs_of_math_tree(node.get_next_sibling(), res);
        },
        "msup" => {
            let msup_res = calculate_msup(node.clone());
            if msup_res.is_none() {
                res.push(node.clone());
            }else {
                let child = node.get_first_child().unwrap();
                child.set_content(&msup_res.unwrap());
                res.push(child);
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
        },
        "msubsup" => {
            //create a new msup node and do the same hack as for msub
            // need a link to the  document here for that
           /* let new_doc = libxml::tree::Document::new().unwrap();
            let new_node = Node::new("msup", None, &new_doc).unwrap();
            node.add
            new_node.add
            */
            return leafs_of_math_tree(node.get_next_sibling(), res);
        },
        _ => return res,
    }
}


/*
    Called by evaluate_math.
    Matches the pattern given as an argument with the leafs also given as an argument.
    Calls the verify-function if either the pattern is complete or there are no more leafs.
    It stores matches as PossQEs with possible exponents stored in the PossQEs as well.
*/
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

    let life_time_prob = if leafs[0].get_first_child().is_some() {leafs[0].get_first_child().unwrap()} else {leafs[0].clone()};

    let mut exp : f64 = 1.0;
    let ref head;

    //search for an exponent
    if leafs[0].get_first_child().is_some() && leafs[0].get_name().eq("msup") {
        let base_exp = msup_base_exp(leafs[0].clone());
        if base_exp.is_none(){
            println!("no proper sub-nodes in an msup node");
        }else{
            let poss_exp = base_exp.unwrap().parse::<f64>();
            if poss_exp.is_err(){
                return;
            }
            exp = poss_exp.unwrap();
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

/*
    Builds a string from the content and the exponent of the PossQE-vec.
*/
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

/*
    Checks that the content matching pattern1 is correct
*/
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

    println!("pattern1 found {}\n", string);

    add_qe_to_kat(vec);

    return;

}

/*
    Checks that the content matching pattern2 is correct
*/
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

    println!("pattern2 found {}\n", string);

    add_qe_to_kat(vec);

    return;


}

/*
    Checks that the content matching pattern3 is correct
*/
pub fn check_result_pattern3(vec : &Vec<PossQE>){
    let string = poss_qe_to_string(&vec);

    let mut i = 1;
    let mut s = String::new();
    while i < vec.len(){
        if !vec[i].node_content.eq(INV_TIMES) {
            if DEBUG {
                println!("{} is not valid for pattern3", string);
            }
            return;
        }
        if i+1 < vec.len() {
            s.push_str(&vec[i+1].node_content)
        }
        i = i + 2;
    }

    if !check_unit_string(&s, vec){
        return;
    }

    println!("pattern3 found {}\n", string);

    add_qe_to_kat(vec);

    return;

}

/*
    Checks that the content matching pattern4 is correct
*/
pub fn check_result_pattern4(vec : &Vec<PossQE>){
    let string = poss_qe_to_string(&vec);

    if vec.len() < 4{
        return;
    }

    if !vec[1].node_content.eq("-"){
        if DEBUG {
            println!("pattern4, not a minus on second position; {} instead", vec[1].node_content);
        }
        return;
    }

    let mut i = 3;
    let mut s = String::new();
    while i < vec.len(){
        if !vec[i].node_content.eq(INV_TIMES) {
            if DEBUG {
                println!("{} is not valid for pattern4", string);
            }
            return;
        }
        if i+1 < vec.len() {
            s.push_str(&vec[i+1].node_content)
        }
        i = i + 2;
    }

    if !check_unit_string(&s, vec){
        return;
    }

    println!("pattern4 found range {}\n",string);


    add_qe_to_kat(vec);

    return;

}

/*
    Searches for degree expressions in math-nodes, especially in msup nodes with
    a number and a degree-symbol as children. Additionally searches for a following "C",
    so for degree Celsius.
*/
pub fn find_degree (leafs : &[Node]){
    if leafs.len() == 0{
        return;
    }

    if leafs[0].get_name().eq("msup"){
        if leafs[0].get_first_child().is_none(){
            return;
        }
        let child1 = leafs[0].get_first_child().unwrap();
        if child1.get_next_sibling().is_none(){
            return;
        }
        let child2 = child1.get_next_sibling().unwrap();

        if child1.get_name().eq("mn") && child2.get_name().eq("mo") && child2.get_all_content().eq("∘"){ //"\u{00B0}"){

            let text = format!("{} \u{00B0}", child1.get_all_content());

            // degree found, but continue to check, whether it is followed by C, resulting in degree Celsius
            let opt_sib1 = leafs[0].get_next_sibling();
            let mut found_c = false;
            if opt_sib1.is_some(){
                let sib1 = opt_sib1.unwrap();
                let opt_sib2 = sib1.get_next_sibling();
                if opt_sib2.is_some(){
                    let sib2 = opt_sib2.unwrap();

                    if sib1.get_name().eq("mo") && sib1.get_all_content().eq(INV_TIMES) && sib2.get_name().eq("mi") && sib2.get_all_content().eq("C"){
                        found_c = true;
                        let new_text = format!("{}C",text);
                        let new_poss_qe = PossQE { node_name : "msup".to_string() , node_id: leafs[0].get_property("id").unwrap(),
                            node_content: new_text.clone(), exp: 1.0 };
                        println!("find_degree found {}", new_text);
                        add_qe_to_kat(&[new_poss_qe]);
                    }
                }
            }

            if !found_c{
                let poss_qe = PossQE { node_name : "msup".to_string() , node_id: leafs[0].get_property("id").unwrap(),
                    node_content: text.clone(), exp: 1.0 };
                println!("find_degree found {}", text);
                add_qe_to_kat(&[poss_qe]);

            }

        }

    }

    find_degree(&leafs[1..]);

}

//TODO work in progress
pub fn check_unit_string(poss_unit : &str, vec : &Vec<PossQE>) -> bool {

    let mut parts : Vec<&[PossQE]> = Vec::new();
    let mut last_ind = 0;
    for i in 0..vec.len(){
        let ref possqe = vec[i];
        if possqe.node_content.eq("/") || possqe.node_content.eq("×"){
            parts.push(&vec[last_ind..i]);
            last_ind = i+1;
        }
    }

    for part in parts{
        println!("TEST {}", poss_qe_to_string(part));
    }

    let temp = check_single_unit_string(poss_unit, vec);
    if temp.is_some() {
        return true;
    }
    return false;
}

/*
    Decomposes a unit string to a prefix, an unit symbol and an exponent, if possible.
*/
pub fn check_single_unit_string(poss_unit : &str, vec : &Vec<PossQE>) -> Option<(String, String, f64)>{

    let mut unit = "";
    let mut prefix = "";


    for u in UNIT_SYMBOLS{
        if poss_unit.ends_with(u) {
            unit = u;
            break;
        }
    }

    if unit == "" {
        return None;
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
    }else{
        println!("Could not disassemble {}\n", poss_qe_to_string(vec));
        return None;
    }

    return Some((prefix.to_string(), unit.to_string(), exp));
}


/*
    Creates the header of the KAT-document. This is still very much work in progress, still waiting for Tom's response
*/
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


