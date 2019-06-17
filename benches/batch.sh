#!/bin/bash -e

MODE_WEBPBN="webpbn"
MODE_NONOGRAMS="nonograms.org"

for i in "$@"; do
    if [[ ${i} == "--help" ]] || [[ ${i} == "-h" ]]; then
        echo "Batch mode for nonogram solver"
        echo "Examples: "
        echo "  Run all http://webpbn.com puzzles till id=35000"
        echo "    $ nohup bash batch.sh $MODE_WEBPBN {1..35000} 2>&1 > batch.log &"
        echo "  (you can find maximum available puzzle ID with the command"
        echo "    $ curl -s https://webpbn.com/find.cgi --data 'order=1&perpage=5&search=1' | grep -oP 'play.cgi\?id=\d+'"
        echo
        echo "  Run all http://nonograms.org puzzles till id=24000"
        echo "  (you can find maximum available puzzle ID with http://www.nonograms.org/search/p/10000?sort=6)"
        echo "    $ nohup bash batch.sh $MODE_NONOGRAMS {1..24000} 2>&1 > batch.log &"
        exit
    fi
done


function run_single_webpbn() {
    local puzzle_id=$1

    local path=puzzles/${puzzle_id}.xml
    if [[ ! -f ${path} ]]; then
        echo "File not found locally. Download into $path"
        wget --timeout=10 -qO- "http://webpbn.com/XMLpuz.cgi?id=$i" > ${path}
        if [[ $? -ne 0 ]]; then
            echo "Failed to download puzzle #$i: timeout" >&2
        fi
    fi

    if cat ${path} | head -1 | grep -q '<?xml'; then
        echo "Solving WEBPBN's puzzle #$i (http://webpbn.com/$i) ..."
        /usr/bin/time -f 'Total: %U' target/release/nonogrid --webpbn ${path} --timeout=3600 --max-solutions=2 2>&1 1>solutions/${puzzle_id}
    else
        echo "No valid file for puzzle #$i" >&2
        local lines=$(cat ${path} | wc -l)
        if [[ ${lines} < 2 ]]; then
            echo "Removing empty file $path"
            rm -f ${path}
        fi
    fi
}


function find_nonogram_url() {
    local number=$1
    local code_org_black=$(wget --spider -S http://www.nonograms.org/nonograms/i/${number} 2>&1 | grep "HTTP/" | awk '{print $2}')
    local code_org_color=$(wget --spider -S http://www.nonograms.org/nonograms2/i/${number} 2>&1 | grep "HTTP/" | awk '{print $2}')
    local code_ru_black=$(wget --spider -S http://www.nonograms.ru/nonograms/i/${number} 2>&1 | grep "HTTP/" | awk '{print $2}')
    local code_ru_color=$(wget --spider -S http://www.nonograms.ru/nonograms2/i/${number} 2>&1 | grep "HTTP/" | awk '{print $2}')

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


function run_single_nonogram() {
    local puzzle_id=$1

    local path=puzzles-norg/${puzzle_id}.html
    if [[ ! -f ${path} ]]; then
        echo "File not found locally. Download into $path"
        url=$(find_nonogram_url ${puzzle_id})
        if [[ ! ${url} ]]; then
            echo "Not found URL for puzzle #$i" >&2
            continue
        fi

        wget --timeout=10 -qO- ${url} > ${path}
        if [[ $? -ne 0 ]]; then
            echo "Failed to download puzzle #$i: timeout" >&2
        fi
    fi

    local lines=$(cat ${path} | wc -l)
    if [[ ${lines} < 2 ]]; then
        echo "Removing empty file $path"
        rm -f ${path}
    else
        echo "Solving NORG puzzle #$i ${url}..."
        /usr/bin/time -f 'Total: %U' target/release/nonogrid --nonograms-org ${path} --timeout=3600 --max-solutions=2 2>&1 1>solutions-norg/${puzzle_id}
    fi
}


function prepare() {
    echo "Start at $(date)"
    export RUST_LOG=nonogrid=warn
    export RUST_BACKTRACE=1
    cargo build --release
}


function run_webpbn() {
    mkdir -p solutions
    mkdir -p puzzles
    prepare

    for i in "$@"; do
        run_single_webpbn ${i}
        echo
    done
}


function run_nonograms() {
    mkdir -p solutions-norg
    mkdir -p puzzles-norg
    prepare

    for i in "$@"; do
        run_single_nonogram ${i}
        echo
    done
}


function long_solvers() {
    # use this function to get the longest solved puzzles
    # you have to specify log file with LOG=warn and the threshold in seconds
    #
    # $ long_solvers batch.log 3  # to show every puzzle that solved more than 3 seconds
    #
    # You can use the total time results from this function
    # to quickly find information about the solution (depth, rate, etc):
    # just issue the following command by providing grep with the total time for given puzzle:
    # $ cat batch.log | grep -m1 -F '3599.93' -B4 -A3
    #
    # Also, you can use ripgrep instead: `rg -o 'Total: (.+)' -r'$1'`.

    local log_file=$1
    local threshold=$2
    while read t; do
        local id=$(cat ${log_file} | grep -m1 -F ${t} -A3 | grep -oP '#\K(\d+)' | awk '{print $1-1}')
        echo "$id: $t"
        #cat ${log_file} | grep -m1 -F ${t} -B4 -A3
        #echo -e '-----------------\n'
    done < <(cat ${log_file} | grep -oP 'Total: \K(.+)' | awk '$1 > t' t=${threshold})
}



mode=$1

case ${mode} in
    ${MODE_WEBPBN})
    shift
    run_webpbn $@
    ;;
    ${MODE_NONOGRAMS})
    shift
    run_nonograms $@
    ;;
    *)    # unknown option
    echo "error: Unkknown mode $mode" >&2
    exit 1
    ;;
esac
echo "End at $(date)"
