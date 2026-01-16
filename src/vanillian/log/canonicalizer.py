import logging
import re
from enum import StrEnum

from .utils import ask_for_confirm

__all__ = ["LogCanonicalizer"]

logger = logging.getLogger(__name__)


class _LogSection(StrEnum):
    DOC = "Doc"
    PAPER = "Paper"


def _canonicalize_item(log_section: _LogSection, log_item: str, sep: str = "|") -> str:
    """Canonicalize a log item to the standard format.

    For items in the "Doc" section, the canonical format is: `doc-notes/<path>|<path>`

    For items in the "Paper" section, the canonical format is:
    `paper-notes/<path>|<year>-<publisher>-<title>`, where `<year>`, `<publisher>`,
    and `<title>` are extracted from the filename in the last part of `<path>`.
    The format of the filename is assumed to be `<title>-<year>-<publisher>.md`, where
    `<publisher>` is optional.

    Args:
        log_section (LogSection): The section of the log (Doc or Paper).
        log_item (str): The log item string to be canonicalized.
        sep (str, optional): The separator between path and title in the log item. Defaults to "|".

    Returns:
        str: The canonicalized log item.
    """
    path = log_item.partition(sep)[0]

    if log_section not in _LogSection:
        raise ValueError(f"Unknown log section: {log_section}")

    match log_section:
        case _LogSection.DOC:
            new_title = path.removeprefix("doc-notes/")
        case _LogSection.PAPER:
            file_name = path.rpartition("/")[-1]
            paper_name, _, paper_info = file_name.partition("-")
            year, _, publisher = paper_info.rpartition("-")
            new_title = f"{year}-{publisher}-{paper_name}" if publisher else f"{year}-{paper_name}"
        case _:
            raise ValueError(f"Unhandled log section: {log_section}")

    return f"{path}{sep}{new_title}"


def _canonicalize_line(
    log_section: _LogSection,
    line: str,
    prefix_sep: str = "[[",
    suffix_sep: str = "]]",
    *,
    force: bool = False,
) -> str:
    """Canonicalize a log item line if not already in canonical form.

    Args:
        log_section (LogSection): The section of the log (Doc or Paper).
        line (str): The log item line to be canonicalized.
        prefix_sep (str, optional): The separator before the log item. Defaults to "[[".
        suffix_sep (str, optional): The separator after the log item. Defaults to "]]".
        force (bool, optional): If True, canonicalize without asking for confirmation. Defaults to False.

    Returns:
        str: The canonicalized log item line.
    """
    line_prefix, _, line_suffix = line.partition(prefix_sep)
    log_item, _, line_suffix = line_suffix.rpartition(suffix_sep)

    if _canonicalize_item(log_section, log_item) == log_item:
        logger.info("Log item %s is already canonicalized.", log_item)
        return line

    logger.info("Log item %s is not canonicalized.", log_item)
    if force or ask_for_confirm(
        "Do you want to canonicalize this item? (press enter to confirm, press any other key to refuse): "
    ):
        logger.info("Force flag is set. Canonicalizing without confirmation.")
        log_item = _canonicalize_item(log_section, log_item)
        line = f"{line_prefix}{prefix_sep}{log_item}{suffix_sep}{line_suffix}"
        logger.info("Canonicalized log item: %s", line.strip())

    logger.info("User refused to canonicalize the log item, skipping.")
    return line


class LogCanonicalizer:
    @staticmethod
    def canonicalize(lines: list[str], *, force: bool = False) -> None:
        """Canonicalize log items in the provided lines.

        Modifies the input lines in place.

        Args:
            lines (list[str]): The lines of the log file.
            force (bool, optional): If True, canonicalize without asking for confirmation. Defaults to False.
        """
        doc_pattern = re.compile(r"\\\[Doc\\\]\s")
        paper_pattern = re.compile(r"\\\[Paper\\\]\s")
        log_item_pattern = re.compile(r"^- \[\[.+\]\]:?")

        logger.info("Starting log canonicalization.")
        current_section = None
        for idx, line in enumerate(lines):
            if doc_pattern.match(line):
                logger.info("Line: %d, entering Doc section", idx)
                current_section = _LogSection.DOC
            elif paper_pattern.match(line):
                logger.info("Line: %d, entering Paper section", idx)
                current_section = _LogSection.PAPER
            elif log_item_pattern.match(line) and current_section is not None:
                logger.info("Line: %d, processing log item", idx)
                lines[idx] = _canonicalize_line(current_section, line, force=force)
                logger.info("Updated line: %s", lines[idx].strip())

        logger.info("Log canonicalization completed.")
