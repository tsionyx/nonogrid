#!/bin/bash -e

# find max available puzzle ID: http://www.nonograms.org/search/p/10000?sort=6
# nohup bash batch-nonograms.org.sh {1..24000} 2>&1 > batch-norg.log &

mkdir -p solutions-norg
mkdir -p puzzles-norg

export RUST_LOG=nonogrid=warn
export RUST_BACKTRACE=1
# 5-10% speedup with this tip https://vfoley.xyz/rust-compilation-tip/
export RUSTFLAGS="-C target-cpu=native"
cargo build --release

echo "Start at $(date)"

function find_puzzle_url() {
    number=$1
    code_org_black=$(wget --spider -S http://www.nonograms.org/nonograms/i/${number} 2>&1 | grep "HTTP/" | awk '{print $2}')
    code_org_color=$(wget --spider -S http://www.nonograms.org/nonograms2/i/${number} 2>&1 | grep "HTTP/" | awk '{print $2}')
    code_ru_black=$(wget --spider -S http://www.nonograms.ru/nonograms/i/${number} 2>&1 | grep "HTTP/" | awk '{print $2}')
    code_ru_color=$(wget --spider -S http://www.nonograms.ru/nonograms2/i/${number} 2>&1 | grep "HTTP/" | awk '{print $2}')

    if [[ ${code_org_black} -eq "200" ]]; then
        echo http://www.nonograms.org/nonograms/i/${number}
    elif [[ ${code_org_color} -eq "200" ]]; then
        echo http://www.nonograms.org/nonograms2/i/${number}
    elif [[ ${code_ru_black} -eq "200" ]]; then
        echo http://www.nonograms.ru/nonograms/i/${number}
    elif [[ ${code_ru_color} -eq "200" ]]; then
        echo http://www.nonograms.ru/nonograms2/i/${number}
    fi
}

for i in $@; do
    path=puzzles-norg/${i}.html
    if [[ ! -f ${path} ]]; then
        echo "File not found locally. Download into $path"
        url=$(find_puzzle_url ${i})
        if [[ ! ${url} ]]; then
            echo "Not found URL for puzzle #$i" >&2
            continue
        fi

        wget --timeout=10 -qO- ${url} > ${path}
        if [[ $? -ne 0 ]]; then
            echo "Failed to download puzzle #$i: timeout" >&2
        fi
    fi

    lines=$(cat ${path} | wc -l)
    if [[ ${lines} < 2 ]]; then
        echo "Removing empty file $path"
        rm -f ${path}
    else
        echo "Solving NORG puzzle #$i ${url}..."
        /usr/bin/time -f 'Total: %U' target/release/nonogrid --nonograms-org ${path} --timeout=3600 --max-solutions=2 2>&1 1>solutions-norg/${i}
    fi
    echo
done

echo "End at $(date)"
