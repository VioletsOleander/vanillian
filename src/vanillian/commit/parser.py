from typing import TYPE_CHECKING

from .regular import Frequency

if TYPE_CHECKING:
    import argparse

__all__ = ["register_subparser"]


def register_subparser(subparsers: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = subparsers.add_parser(
        "commit",
        description="Generate daily or weekly commit message and commit changes.",
        help="Generate daily or weekly commit message and commit changes.",
    )
    parser.add_argument(
        "frequency",
        choices=Frequency,
        help="Specify 'daily' for a daily commit, 'weekly' for a weekly commit.",
    )
    parser.add_argument(
        "-l",
        "--late",
        action="store_true",
        help="Generate headline as if now is one day earlier",
    )
    parser.add_argument(
        "-e",
        "--edit",
        action="store_true",
        help="Open the editor for editing the commit message before committing",
    )
