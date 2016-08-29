start:
	npm start
.PHONY: start

install:
	npm install
	if [ ! -f npm-shrinkwrap.json ]; then npm shrinkwrap; fi;
.PHONY: install

distclean: clean
	if [ -f npm-shrinkwrap.json ]; then rm npm-shrinkwrap.json; fi;
.PHONY: distclean

clean:
	rm -fr node_modules typings
.PHONY: clean
