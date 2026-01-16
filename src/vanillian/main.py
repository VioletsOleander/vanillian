import argparse
import logging
import sys
from pathlib import Path

from .log.canonicalizer import LogCanonicalizer
from .log.constants import LOG_PATH
from .log.utils import read_lines, write_lines_atomic

logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)

logger = logging.getLogger(__name__)


def parse_args() -> argparse.Namespace:
    prog = Path(sys.argv[0]).name
    parser = argparse.ArgumentParser(
        prog=prog,
        description="The dedicated steward for vault-vanilla.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command", required=True)
    log_parser = subparsers.add_parser(
        "log",
        description="The canonicalizer for the vault log file.",
        help="Canonicalize log lines in the vault log file",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    log_parser.add_argument(
        "file",
        nargs="?",
        default="logs/Personal Log.md",
        help="Path to the log file to be canonicalized",
    )
    log_parser.add_argument(
        "-f",
        "--force",
        action="store_true",
        help="Do not ask for confirm before canonicalizing each log line",
    )
    log_parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Overwrite the original log file, otherwise, write to another .canonicalized suffixed file",
    )

    commit_parser = subparsers.add_parser(
        "commit",
        help="Commit changes to the vault",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    commit_parser.add_argument(
        "--message", "-m", type=str, required=True, help="The commit message for the changes"
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.command == "log":
        try:
            lines = read_lines(LOG_PATH)

            canonicalizer = LogCanonicalizer()
            canonicalizer.canonicalize(lines, force=args.force)

            dest_path = LOG_PATH if args.overwrite else LOG_PATH.with_suffix(".canonicalized.md")
            write_lines_atomic(dest_path, lines)
        except FileNotFoundError:
            logger.exception("Log file not found at %s", LOG_PATH)
            return 1
        except Exception:
            logger.exception("An error occurred while canonicalizing the log.")
            return 1
    else:
        raise NotImplementedError(f"Command {args.command} is not implemented yet.")
    return 0
