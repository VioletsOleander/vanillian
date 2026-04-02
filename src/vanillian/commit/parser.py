from typing import TYPE_CHECKING

from .regular import Frequency

if TYPE_CHECKING:
    import argparse

__all__ = ["register_subparser"]


def register_subparser(subparsers: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = subparsers.add_parser(
        "commit",
        description="Commit related commands",
        help="Commit related commands",
    )
    sub_subparsers = parser.add_subparsers(dest="commit_subcommand", required=True)
    _register_regular_parser(sub_subparsers)
    _register_item_parser(sub_subparsers)


def _register_regular_parser(
    sub_subparsers: argparse._SubParsersAction[argparse.ArgumentParser],
) -> None:
    parser = sub_subparsers.add_parser(
        "regularly",
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
        "--no-edit",
        action="store_true",
        help="Do not open the editor for editing the commit message before committing",
    )


def _register_item_parser(
    sub_subparsers: argparse._SubParsersAction[argparse.ArgumentParser],
) -> None:
    _ = sub_subparsers.add_parser(
        "item",
        description="Construct commit messages based on staged note items and launch git commit.",
        help="Construct commit messages based on staged note items and launch git commit.",
    )
