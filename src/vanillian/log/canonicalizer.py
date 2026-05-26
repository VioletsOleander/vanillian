import logging
import re
from enum import StrEnum

from .utils import ask_for_confirm

__all__ = ["LogCanonicalizer"]

logger = logging.getLogger(__name__)


class LogCanonicalizer:
    def __init__(self) -> None:
        self.section_patterns = {
            _LogSection.DOC: re.compile(r"\\\[Doc\\\]\s"),
            _LogSection.PAPER: re.compile(r"\\\[Paper\\\]\s"),
            _LogSection.WIKI: re.compile(r"\\\[Wiki\\\]\s"),
            _LogSection.OTHER: re.compile(r"\\\[.+\\\]\s"),
        }
        self.log_item_pattern = re.compile(r"^- \[\[.+\]\]:?")
        self.paper_file_name_pattern = re.compile(
            r"^(?P<title>.+)-(?P<year>\d{4})(-(?P<publisher>.+))?$"
        )

    def canonicalize(self, lines: list[str], *, force: bool = False) -> list[str]:
        """Canonicalize log items in the provided lines.

        Args:
            lines (list[str]): The lines of the log file.
            force (bool, optional): If True, canonicalize without asking for confirmation. Defaults to False.

        Returns:
            list[str]: The canonicalized log lines.
        """
        logger.info("Starting log canonicalization.")

        current_section = _LogSection.OTHER
        for idx, line in enumerate(lines):
            if (match := _match_section(line, self.section_patterns)) is not None:
                level = logger.info if current_section != _LogSection.OTHER else logger.debug
                level("Line: %d: Entering section: '%s'", idx, match.value)
                current_section = match
            elif self.log_item_pattern.match(line) and current_section != _LogSection.OTHER:
                logger.info("Line: %d: Processing...", idx)
                lines[idx] = self._canonicalize_line(current_section, line, force=force)
                logger.info("Line: %d: Finished processing.", idx)

        logger.info("Log canonicalization completed.")

        return lines

    def _canonicalize_line(
        self,
        log_section: _LogSection,
        line: str,
        prefix_sep: str = "[[",
        suffix_sep: str = "]]",
        *,
        force: bool = False,
    ) -> str:
        """Canonicalize a log line if not already in canonical form.

        Args:
            log_section (LogSection): The section of the log.
            line (str): The log line to be canonicalized.
            prefix_sep (str, optional): The separator before the log item. Defaults to "[[".
            suffix_sep (str, optional): The separator after the log item. Defaults to "]]".
            force (bool, optional): If True, canonicalize without asking for confirmation. Defaults to False.

        Returns:
            str: The canonicalized log item line.
        """
        line_prefix, _, line_suffix = line.partition(prefix_sep)
        log_item, _, line_suffix = line_suffix.rpartition(suffix_sep)

        if self._canonicalize_item(log_section, log_item) == log_item:
            logger.debug("Log line '%s' is already canonicalized.", line)
            return line

        logger.info("Log line '%s' is not canonicalized.", line)
        if force or ask_for_confirm(
            "> Do you want to canonicalize this line? (Enter to confirm, any other key to refuse): "
        ):
            logger.info("Canonicalizing log line '%s'.", line)
            log_item = self._canonicalize_item(log_section, log_item)
            line = f"{line_prefix}{prefix_sep}{log_item}{suffix_sep}{line_suffix}"
            logger.info("Canonicalized log line: '%s'", line)
        else:
            logger.info("Skipping canonicalization for log line '%s'.", line)

        return line

    def _canonicalize_item(self, log_section: _LogSection, log_item: str, sep: str = "|") -> str:
        """Canonicalize a log item to the standard format.

        For items in the "Doc" section, the canonical format is: `doc-notes/<path>|<path>`

        For items in the "Wiki" section, the canonical format is: `wiki-notes/<path>|<path>`

        For items in the "Paper" section, the canonical format is:
        `paper-notes/<path>|<year>-<publisher>-<title>`, where `<year>`, `<publisher>`,
        and `<title>` are extracted from the filename in the last part of `<path>`.
        The format of the filename is assumed to be `<title>-<year>-<publisher>.md`, where
        `<publisher>` is optional.

        Args:
            log_section (LogSection): The section of the log.
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
            case _LogSection.WIKI:
                new_title = path.removeprefix("wiki-notes/")
            case _LogSection.PAPER:
                file_name = path.rpartition("/")[-1]

                match = self.paper_file_name_pattern.fullmatch(file_name)
                if match is None:
                    raise ValueError(
                        f"Paper file name '{file_name}' does not match expected pattern."
                    )

                paper_name = match.group("title")
                year = match.group("year")
                publisher = match.group("publisher")

                new_title = "-".join(filter(None, [year, publisher, paper_name]))
            case _:
                raise ValueError(f"Unhandled log section: {log_section}")

        return f"{path}{sep}{new_title}"


class _LogSection(StrEnum):
    DOC = "Doc"
    PAPER = "Paper"
    WIKI = "Wiki"
    OTHER = "Other"


def _match_section(
    line: str, section_patterns: dict[_LogSection, re.Pattern]
) -> _LogSection | None:
    """Return the log section type if the line matches any section pattern, else None."""
    for section_type, pattern in section_patterns.items():
        if pattern.fullmatch(line):
            return section_type
    return None
