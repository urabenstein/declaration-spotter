.PHONY: all run 

HTML = ../../Semanticextraction/PapersFromCortex/small_eval/cond-mat9807235.html 


OWLAPI = ../libs/owlapi-distribution-4.0.2.jar
JAW = 	 ../libs/jaws-bin.jar

all: 
	cargo test

nodes: all
	./target/debug/examples/simple_kat_export nodes ${HTML}

text: all
	./target/debug/examples/simple_kat_export text ${HTML}

#use anything different from text/nodes here
run: all
	./target/debug/examples/simple_kat_export run ${HTML}


