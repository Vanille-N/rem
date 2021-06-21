MENUS = $(shell grep -E '<[a-z]+>' 'help.fmt' | grep -v end | tr -d '<>')
GEN = $(wildcard *.sh)
ANSI = $(MENUS:%=%.ansi)
CHAPS = $(MENUS:%=%.chap)

.SUFFIXES: 

all: $(ANSI) $(GEN)

%.ansi: %.chap $(GEN)
	./shade.sh $< > $@

%.chap: help.fmt $(GEN)
	./cut.sh $(shell basename $@ | cut -d'.' -f1) > $@

clean:
	rm *.ansi
	rm *.chap

.PHONY: clean
