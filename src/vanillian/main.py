import argparse
import logging

from .log.canonicalizer import LogCanonicalizer
from .log.constants import LOG_PATH
from .log.utils import read_lines, write_lines_atomic

logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)

logger = logging.getLogger(__name__)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="The dedicated steward for my vanilla vault.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    parser_log = subparsers.add_parser("log", help="Canonicalize log items")
    parser_log.add_argument(
        "-f",
        "--force",
        action="store_true",
        help="Not ask for confirmation before canonicalizing each log item",
    )
    parser_log.add_argument(
        "--overwrite",
        action="store_true",
        help="Overwrite the original log file, otherwise write to another .canonicalized suffixed file",
    )

    parser_commit = subparsers.add_parser("commit", help="Commit changes to the vault")
    parser_commit.add_argument(
        "--message", "-m", type=str, required=True, help="The commit message for the changes"
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.command == "log":
        try:
            lines = read_lines(LOG_PATH)
            LogCanonicalizer.canonicalize(lines, force=args.force)

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
