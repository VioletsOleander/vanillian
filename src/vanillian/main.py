# ruff: noqa: PLC0415

import logging
import subprocess
from pathlib import Path
from typing import TYPE_CHECKING

from .parser import parse_args

if TYPE_CHECKING:
    from collections.abc import Callable

logging.basicConfig(level=logging.INFO, format="%(message)s")

logger = logging.getLogger(__name__)


def handle_exception(func: Callable[[], int]) -> Callable[[], int]:
    def wrapper() -> int:
        try:
            return func()
        except FileNotFoundError:
            logger.exception("A required file was not found.")
            return 1
        except subprocess.CalledProcessError:
            logger.exception("A subprocess command failed.")
            return 1
        except ValueError:
            logger.exception("A value error occurred during processing.")
            return 1
        except Exception:
            logger.exception("An unexpected error occurred.")
            return 1

    return wrapper


@handle_exception
def main() -> int:
    args = parse_args()

    if args.command == "log":
        from .log.canonicalizer import LogCanonicalizer
        from .log.utils import read_lines, write_lines_atomic

        log_path = Path(args.file)
        lines = read_lines(log_path)

        canonicalizer = LogCanonicalizer()
        lines = canonicalizer.canonicalize(lines, force=args.force)

        dest_path = log_path if args.overwrite else log_path.with_suffix(".canonicalized.md")
        write_lines_atomic(dest_path, lines)
    elif args.command == "commit":
        if args.commit_subcommand == "regularly":
            from .commit.regular import GitCommitRegularly

            command = GitCommitRegularly()
            command.execute(
                frequency=args.frequency,
                late=args.late,
                no_edit=args.no_edit,
            )
        elif args.commit_subcommand == "item":
            from .commit.item import GitCommitItem

            command = GitCommitItem()
            command.execute()
        else:
            raise ValueError(f"Unknown commit subcommand: {args.commit_subcommand}")
    else:
        raise ValueError(f"Unknown command: {args.command}")

    return 0
