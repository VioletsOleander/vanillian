import argparse

__all__ = ["register_subparser"]


def register_subparser(subparsers: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = subparsers.add_parser(
        "log",
        description="Parse and canonicalize log lines in the vault log file.",
        help="Parse and canonicalize log lines in the vault log file.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "file",
        nargs="?",
        default="logs/2026.md",
        help="Path to the log file to be canonicalized",
    )
    parser.add_argument(
        "-f",
        "--force",
        action="store_true",
        help="Do not ask for confirm before canonicalizing each log line",
    )
    parser.add_argument(
        "-o",
        "--overwrite",
        action="store_true",
        help="Overwrite the original log file, otherwise, write to another .canonicalized suffixed file",
    )
