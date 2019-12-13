CC=gcc

all: mksunxiboot

mksunxiboot: mksunxiboot.o
	gcc mksunxiboot.o -o mksunxiboot

mksunxiboot.o: mksunxiboot.c
	gcc -c mksunxiboot.c

.PHONY: clean
clean:
	rm -f mksunxiboot
	rm -f *.o

