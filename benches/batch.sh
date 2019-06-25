#!/bin/bash -e

MODE_WEBPBN="webpbn"
MODE_NONOGRAMS="nonograms.org"
MODE_STAT="stat"

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
        echo
        echo "  Run statistic on collected log file"
        echo "    $ bash batch.sh stat batch.log 0.1 --details"
        exit
    fi
done


function run_single_webpbn() {
    local puzzle_id=$1
    local fmt=olsak

    local path=puzzles/${puzzle_id}.${fmt}
    if [[ ! -f ${path} ]]; then
        echo "File not found locally. Download into $path"
        wget --timeout=10 -qO- "https://webpbn.com/export.cgi" --post-data "id=$i&fmt=$fmt&go=1" > ${path}
        if [[ $? -ne 0 ]]; then
            echo "Failed to download puzzle #$i: timeout" >&2
        fi
    fi

    if cat ${path} | grep -q ': rows'; then
        echo "Solving WEBPBN's puzzle #$i (http://webpbn.com/$i) ..."
        /usr/bin/time -f 'Total: %U' target/release/nonogrid ${path} --timeout=3600 --max-solutions=2 2>&1 1>solutions/${puzzle_id}
    else
        echo "No valid file for puzzle #$i" >&2
        local lines=$(cat ${path} | wc -l)
        if [[ ${lines} -lt 2 ]]; then
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

    local path=puzzles-norg/${puzzle_id}.js
    if [[ ! -f ${path} ]]; then
        echo "File not found locally. Download into $path"
        url=$(find_nonogram_url ${puzzle_id})
        if [[ ! ${url} ]]; then
            echo "Not found URL for puzzle #$i" >&2
            return
        fi

        wget --timeout=10 -qO- ${url} > ${path}
        if [[ $? -ne 0 ]]; then
            echo "Failed to download puzzle #$i: timeout" >&2
        fi
    fi

    local lines=$(cat ${path} | wc -l)
    if [[ ${lines} -lt 1 ]]; then
        echo "Removing empty file $path"
        rm -f ${path}
    else
        sed -n '/^var d=.\+;/p' -i ${path}
        echo "Solving NORG puzzle #$i ${url}..."
        /usr/bin/time -f 'Total: %U' target/release/nonogrid ${path} --timeout=3600 --max-solutions=2 2>&1 1>solutions-norg/${puzzle_id}
    fi
}


function prepare() {
    echo "Start at $(date)"
    export RUST_LOG=nonogrid=warn
    export RUST_BACKTRACE=1
    cargo build --release --no-default-features --features="clap std_time env_logger"
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
    local threshold=${2:-10}

    local details=
    for i in "$@"; do
        if [[ ${i} == "--details" ]]; then
           local details=1
        fi
    done

    while read t; do
        if [[ ${details} ]]; then
            exec 5>&1
        else
            exec 5>/dev/null
        fi

        # https://stackoverflow.com/a/12451419
        local id=$(cat ${log_file} | grep -m1 -F "Total: ${t}" -B4 -A3 | tee >(cat - >&5) | grep -oP '#\K(\d+)' | awk '{print $1-1}')
        echo "$id: $t"

        if [[ ${details} ]]; then
            echo -e '-----------------\n'
        fi
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
    ${MODE_STAT})
    shift
    long_solvers $@
    ;;
    *)    # unknown option
    echo "error: Unkknown mode $mode" >&2
    exit 1
    ;;
esac
echo "End at $(date)"
