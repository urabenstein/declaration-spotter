use std::collections::HashMap;

use libxml::tree::Node;

use spotter::DeclarationQuadruple;

// cse(//*[@id='S5.p2.1.m1.1.1'],//*[@id='    S5.p2.1.m1.1.1'],//*[@id='S5.p2.1.m1.1.1'])
// <rdf:RDF xmlns:d="http://jfschaefer.de/declarations/KAnnSpec#"
// xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
// xmlns:kat="https://github.com/KWARC/KAT/"><rdf:Description><kat:annotation
// rdf:nodeID="kat_run"></kat:annotation></rdf:Description><rdf:Description
// rdf:nodeID="kat_run"><rdf:type rdf:resource="kat:run"></rdf:type><kat:date
// rdf:datatype="xs:dateTime">2016-03-15T08:13:38.061Z</kat:date><kat:tool>KAT</kat:tool><kat:runid>0</kat:runid></rdf:Description><rdf:Description><kat:annotation
// rdf:nodeID="KAT_1458029586470_Declarations"></kat:annotation></rdf:Description><rdf:Description
// rdf:nodeID="KAT_1458029586470_Declarations"><rdf:type
// rdf:resource="kat:kannspec"></rdf:type><kat:kannspec-name>Declarations</kat:kannspec-name><kat:kannspec-uri>http://localhost:3000/KAnnSpecs/declaration-annotations.xml</kat:kannspec-uri></rdf:Description><rdf:Description
// rdf:nodeID="KAT_1458029614119_1327"><kat:run rdf:nodeID="kat_run"></kat:run><kat:kannspec
// rdf:nodeID="KAT_1458029586470_Declarations"></kat:kannspec><kat:concept>Variable</kat:concept><kat:type
// rdf:resource="http://jfschaefer.de/declarations/KAnnSpec#Variable"></kat:type><kat:annotates
// rdf:resource="http://localhost:3000/content/test2.html#cse(%2F%2F*%5B%40id%3D'S1'%5D%2C%2F%2F*%5B%40id%3D'word.30.2'%5D%2C%2F%2F*%5B%40id%3D'word.34.22'%5D)"></kat:annotates><d:loremipsum>abc</d:loremipsum></rdf:Description></rdf:RDF>



pub fn kat_export(declarations: &Vec<DeclarationQuadruple>) -> String {
    let mut s = String::new();
    s.push_str("<rdf:RDF xmlns:d=\"http://jfschaefer.de/declarations/KAnnSpec#\" xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\" xmlns:kat=\"https://github.com/KWARC/KAT/\"><rdf:Description><kat:annotation rdf:nodeID=\"kat_run\"></kat:annotation></rdf:Description><rdf:Description rdf:nodeID=\"kat_run\"><rdf:type rdf:resource=\"kat:run\"></rdf:type><kat:date rdf:datatype=\"xs:dateTime\">2016-03-13T14:37:30.858Z</kat:date><kat:tool>KAT</kat:tool><kat:runid>0</kat:runid></rdf:Description><rdf:Description><kat:annotation rdf:nodeID=\"KAT_Declarations_KAnnSpec\"></kat:annotation></rdf:Description><rdf:Description rdf:nodeID=\"KAT_Declarations_KAnnSpec\"><rdf:type rdf:resource=\"kat:kannspec\"></rdf:type><kat:kannspec-name>Declarations</kat:kannspec-name><kat:kannspec-uri>http://localhost:3000/KAnnSpecs/declaration-annotations.xml</kat:kannspec-uri></rdf:Description>");
    let mut hs : HashMap<Node, String> = HashMap::new();
    let mut var_id_count = 0usize;
    let mut decl_id_count = 0usize;
    for q in declarations {
        // let variablekatid : String;
        // let x = hs.get(&q.variable);
        // match x {
        if !hs.contains_key(&q.variable) {
                let variablekatid = format!("var_{}", var_id_count);
                var_id_count += 1;
                let varid = q.variable.get_property("id").unwrap();
                // s.push_str(&format!("</rdf:Description><rdf:Description rdf:nodeID=\"{}\"><kat:run rdf:nodeID=\"kat_run\"></kat:run><kat:kannspec rdf:nodeID=\"KAT_Declarations_KAnnSpec\"></kat:kannspec><kat:concept>Variable</kat:concept><kat:type rdf:resource=\"http://jfschaefer.de/declarations/KAnnSpec#Variable\"></kat:type><kat:annotates rdf:resource=\"http://localhost:3000/content/test.html#cse(//*[@id='{}'],//*[@id='{}'],//*[@id='{}'])\"></kat:annotates><d:loremipsum></d:loremipsum></rdf:Description>", &variablekatid, varid, varid, varid));
                s.push_str(&format!("<rdf:Description rdf:nodeID=\"{}\"><kat:run rdf:nodeID=\"kat_run\"></kat:run><kat:kannspec rdf:nodeID=\"KAT_Declarations_KAnnSpec\"></kat:kannspec><kat:concept>Variable</kat:concept><kat:type rdf:resource=\"http://jfschaefer.de/declarations/KAnnSpec#Variable\"></kat:type><kat:annotates rdf:resource=\"http://localhost:3000/content/test.html#cse(%2F%2F*%5B%40id%3D'{}'%5D%2C%2F%2F*%5B%40id%3D'{}'%5D%2C%2F%2F*%5B%40id%3D'{}'%5D)\"></kat:annotates><d:loremipsum>siamet</d:loremipsum></rdf:Description>", &variablekatid, varid, varid, varid));
                //%2F%2F*%5B%40id%3D
                hs.insert(q.variable.clone(), variablekatid);
        }
        //     Some(ref x) => {
        //         variablekatid = x.to_string();
        //     }
        let variablekatid = hs.get(&q.variable).unwrap();
        let declid = format!("declaration_{}", decl_id_count);
        decl_id_count += 1;
        s.push_str(&format!("<rdf:Description rdf:nodeID=\"{}\"><kat:run rdf:nodeID=\"kat_run\"></kat:run><kat:kannspec rdf:nodeID=\"KAT_Declarations_KAnnSpec\"></kat:kannspec><kat:concept>Declaration</kat:concept><kat:type rdf:resource=\"http://jfschaefer.de/declarations/KAnnSpec#Declaration\"></kat:type><kat:annotates rdf:resource=\"http://localhost:3000/content/test.html#cse(%2F%2F*%5B%40id%3D'{}'%5D%2C%2F%2F*%5B%40id%3D'{}'%5D%2C%2F%2F*%5B%40id%3D'{}'%5D)\"></kat:annotates><d:declares rdf:nodeID=\"{}\"></d:declares><d:polarity rdf:resource=\"http://jfschaefer.de/declarations/KAnnSpec#universal\"></d:polarity></rdf:Description>", declid, q.sentence.get_property("id").unwrap(), q.restriction_start.get_property("id").unwrap(), q.restriction_end.get_property("id").unwrap(), variablekatid));
    }
    s.push_str("</rdf:RDF>");
    return s;
}


