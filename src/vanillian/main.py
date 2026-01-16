# ruff: noqa: PLC0415

import logging
from pathlib import Path

from .parse import parse_args

logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)

logger = logging.getLogger(__name__)


def main() -> int:
    args = parse_args()

    if args.command == "log":
        from .log.canonicalizer import LogCanonicalizer
        from .log.utils import read_lines, write_lines_atomic

        try:
            log_path = Path(args.file)
            lines = read_lines(log_path)

            canonicalizer = LogCanonicalizer()
            canonicalizer.canonicalize(lines, force=args.force)

            dest_path = log_path if args.overwrite else log_path.with_suffix(".canonicalized.md")
            write_lines_atomic(dest_path, lines)
        except FileNotFoundError:
            logger.exception("Log file not found at %s", log_path)
            return 1
        except Exception:
            logger.exception("An error occurred while canonicalizing the log.")
            return 1
    elif args.command == "commit":
        from .commit.regular import Frequency, GitCommitRegularly

        try:
            command = GitCommitRegularly()
            frequency = Frequency[args.frequency.upper()]

            command.execute(
                frequency,
                late=args.late,
                edit=args.edit,
            )
        except Exception:
            logger.exception("An error occurred while executing the commit.")
            return 1
    else:
        raise NotImplementedError(f"Command {args.command} is not implemented yet.")

    return 0
