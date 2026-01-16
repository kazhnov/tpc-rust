build: 
	cargo build
	mv target/debug/tpc tpc
build_libs: build
	./tpc o src/pu.tp lib/pu

run: build_libs
	echo "built libs"
