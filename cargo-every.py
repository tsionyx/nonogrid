#!/usr/bin/env python3

import argparse
import configparser
import os
from itertools import chain, combinations

CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
DEFAULT_MANIFEST = os.path.join(CURRENT_DIR, "Cargo.toml")


def all_features(manifest_file):
    config = configparser.ConfigParser()
    config.read(manifest_file)
    return [f for f in sorted(config['features']) if f != "default"]


def all_subsets(it):
    """
    Generate all subsets of any iterable.
    The total number of such a set is 2^len(it)
    https://stackoverflow.com/a/5898031
    """
    return chain(*map(lambda x: combinations(it, x), range(0, len(it) + 1)))


def main():
    parser = argparse.ArgumentParser(description="Run cargo command on all features combinations (e.g. `check`)")
    parser.add_argument('command', help="Cargo subcommand to run")
    parser.add_argument('trailing_args', nargs='*', help="Additional arguments")
    parser.add_argument('--manifest-file', default=DEFAULT_MANIFEST,
                        help="Alternative path to Cargo.toml (default is {}).".format(
                            DEFAULT_MANIFEST))
    args = parser.parse_args()

    cargo_cmd = "cargo {}".format(args.command)
    features = all_features(args.manifest_file)
    total_features_number = 1 << len(features)

    for i, feature_set in enumerate(all_subsets(features)):
        cmd = "{} --no-default-features --features={} {}".format(cargo_cmd, ','.join(feature_set),
                                                                 ' '.join(args.trailing_args))
        print("======== ({}/{}) Running with features '{}' ========".format(i + 1,
                                                                            total_features_number,
                                                                            feature_set))
        print(cmd)

        return_code = os.system(cmd)
        if return_code != 0:
            raise ValueError("Bad return code: {}".format(return_code))


if __name__ == '__main__':
    main()
