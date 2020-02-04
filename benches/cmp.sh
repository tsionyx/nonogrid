#!/usr/bin/env bash

# docker run -it --name=cmp-nono ubuntu:18.04 /bin/bash
# docker start cmp-nono && docker exec -it cmp-nono /bin/bash
# Requirements: curl gcc git libxml2-dev make openjdk-8-jre-headless time unzip

function build_solvers() {
    echo "Collecting all the solvers..."

    echo "1. Naughty"
    curl -s http://kcwu.csie.org/~kcwu/nonogram/naughty/naughty-v88.tgz | tar -xz
    cd naughty-v88
    make
    cd ..

    echo "2. JSolver"
    curl -s -L https://sourceforge.net/projects/jsolver/files/jsolver/jsolver-1.4/jsolver-1.4-src.tar.gz/download | tar -xz
    cd jsolver-1.4-src/
    gcc -O2 -o jsolver jsolver.c
    cd ..

    echo "3. pbnsolve"
    curl -s -L https://storage.googleapis.com/google-code-archive-downloads/v2/code.google.com/pbnsolve/pbnsolve-1.09.tgz | tar -xz
    cd pbnsolve-1.09
    sed 's:^CFLAGS= -O2$:CFLAGS= -O2 -I/usr/include/libxml2:' -i Makefile
    make pbnsolve
    cd ..

    echo "4. grid"
    curl -s http://petr.olsak.net/ftp/olsak/grid/grid.tgz | tar -xz
    cd grid
    gcc -O3 -o grid grid.c
    cd ..

    echo "5. BGU"
    curl -s https://www.cs.bgu.ac.il/~benr/nonograms/bgusolver_cmd_102.jar > bgu.jar

    echo "6. Copris"
    curl -s https://downloads.lightbend.com/scala/2.10.7/scala-2.10.7.tgz | tar -xz
    curl -s http://bach.istc.kobe-u.ac.jp/copris/puzzles/nonogram/copris-nonogram-v1-2.jar > copris-nonogram-v1-2.jar

    echo "7. Color Copris"
    curl -s https://downloads.lightbend.com/scala/2.12.10/scala-2.12.10.tgz | tar -xz
    curl -s http://bach.istc.kobe-u.ac.jp/copris/packages/copris-v2-3-1.zip > copris.zip
    unzip copris.zip && rm copris.zip
    curl -s http://bach.istc.kobe-u.ac.jp/copris/puzzles/nonogram/copris-nonogram-v1-2-src.zip > copris-src.zip
    unzip copris-src.zip && rm copris-src.zip
    curl -s https://webpbn.com/survey/Nonogram-Color.scala > copris-nonogram-v1-2/Nonogram-Color.scala

    echo "8. nonogrid"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh
    bash rustup.sh -y && source $HOME/.cargo/env
    git clone --single-branch --branch=dev https://github.com/tsionyx/nonogrid.git
    cd nonogrid
    cargo build --release --no-default-features --features="args std_time logger sat"
    cp target/release/nonogrid .
    cd ..
}


function benchmarks() {
    echo "Running benchmarks..."
    RES_FILE=$1
    shift

    rm ${RES_FILE}
    for id in $@; do
        echo ${id} | tee -a ${RES_FILE}
        SS_PATH=puzzles/${id}.ss
        SYRO_PATH=puzzles/${id}.syro
        NIN_PATH=puzzles/${id}.nin
        OLSAK_PATH=puzzles/${id}.olsak
        CWD_PATH=puzzles/${id}.cwd
        curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=ss&go=1" > ${SS_PATH}
        curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=syro&go=1" > ${SYRO_PATH}
        curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=nin&go=1" > ${NIN_PATH}
        curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=olsak&go=1" > ${OLSAK_PATH}
        curl -s https://webpbn.com/export.cgi --data "id=$id&fmt=cwd&go=1" > ${CWD_PATH}

        echo -e "\033[7mWu\033[m"
        timeout 2000 /usr/bin/time -f 'naughty: %U' build/naughty-v88/naughty -u < ${SS_PATH} 2>>${RES_FILE}

        echo -e "\033[7mSyromolotov\033[m"
        timeout 2000 /usr/bin/time -f 'JSolver: %U' build/jsolver-1.4-src/jsolver -n 2 ${SYRO_PATH} 2>>${RES_FILE}

        echo -e "\033[7mWolter\033[m"
        timeout 2000 /usr/bin/time -f 'pbnsolve: %U' build/pbnsolve-1.09/pbnsolve -u -x1800 ${NIN_PATH} 2>>${RES_FILE}

        echo -e "\033[7mOlsak\033[m"
        timeout 2000 /usr/bin/time -f 'grid: %U' build/grid/grid -total 2 -log 1 ${OLSAK_PATH} 2>>${RES_FILE}

        echo -e "\033[7mBGU\033[m"
        timeout 2000 /usr/bin/time -f 'BGU: %U' java -jar build/bgu.jar -file ${NIN_PATH} -maxsolutions 2 -timeout 1800 2>>${RES_FILE}

        echo -e "\033[7mTamura/Copris\033[m"
        /usr/bin/time -f 'Copris: %U' build/scala-2.10.7/bin/scala -cp build/copris-nonogram-v1-2.jar nonogram.Solver ${CWD_PATH} 2>>${RES_FILE}

        echo -e "\033[7mtsionyx\033[m"
        timeout 2000 /usr/bin/time -f 'nonogrid: %U' build/nonogrid/nonogrid ${NIN_PATH} --timeout=1800 --max-solutions=2 2>>${RES_FILE}

        echo
    done
}

BASE=${1:-$HOME/bench}
mkdir -p ${BASE}/build ${BASE}/puzzles

cd ${BASE}/build
build_solvers

cd ${BASE}
benchmarks ${BASE}/results 1 6 16 21 23 27 65 436 529 803 1611 1694 2040 2413 2556 2712 3541 4645 6574 6739 7604 8098 9892 10088 10810 12548 18297 22336
