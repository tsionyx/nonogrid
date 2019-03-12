#!/bin/bash -e

# nohup bash batch-webpbn.sh {1..35000} 2>&1 > batch.log &

mkdir -p solutions
mkdir -p puzzles

export RUST_LOG=nonogrid=warn
export RUST_BACKTRACE=1
cargo build --release

echo "Start at $(date)"
for i in $@; do
    path=puzzles/${i}.xml
    if [[ ! -f ${path} ]]; then
        echo "File not found locally. Donwload into $path"
        wget --timeout=10 -qO- "http://webpbn.com/XMLpuz.cgi?id=$i" > ${path}
        if [[ $? -ne 0 ]]; then
            echo "Failed to download puzzle #$i: timeout" >&2
        fi
    fi

    if cat ${path} | head -1 | grep -q '<?xml'; then
        echo "Solving PBN's puzzle #$i (http://webpbn.com/$i) ..."
        /usr/bin/time -f 'Total: %U' target/release/nonogrid --webpbn ${path} --timeout=3600 --max-solutions=2 2>&1 1>solutions/${i}
    else
        echo "No valid file for puzzle #$i" >&2
        lines=$(cat ${path} | wc -l)
        if [[ ${lines} < 2 ]]; then
            echo "Removing empty file $path"
            rm -f ${path}
        fi
    fi
    echo
done

echo "End at $(date)"


function long_solvers() {
    # use this function to get the longest solved puzzles
    # you have to specify log file with LOG=warn and the threshold in seconds
    #
    # long_solvers batch.log 3  # to show every puzzle that solved more than 3 seconds

    log_file=$1
    threshold=$2
    while read t; do
        id=$(grep -P ${t} ${log_file} -A3 | grep -oP '#\K(\d+)' | awk '{print $1-1}')
        echo "$id: $t"
    done < <(grep -oP 'Total: \K(.+)' ${log_file} | awk '$1 > t' t=${threshold})
}
