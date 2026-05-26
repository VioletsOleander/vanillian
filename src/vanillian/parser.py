import argparse
import sys
from pathlib import Path

from .commit.parser import register_subparser as register_commit_parser
from .log.parser import register_subparser as register_log_parser

__all__ = ["parse_args"]


def parse_args() -> argparse.Namespace:
    prog = Path(sys.argv[0]).name
    parser = argparse.ArgumentParser(
        prog=prog,
        description="The toolbox for vault-vanilla.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    register_log_parser(subparsers)
    register_commit_parser(subparsers)

    return parser.parse_args()
