all:
	reflex -r '(.*\.rs|Makefile.*)$$' -- cargo test -- --nocapture
