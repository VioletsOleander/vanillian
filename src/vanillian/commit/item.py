# ruff: noqa: S603
import logging
import re
import subprocess
from pathlib import Path

__all__ = ["GitCommitItem"]

logger = logging.getLogger(__name__)


class GitCommitItem:
    def execute(self) -> None:
        """Construct commit messages based on staged note items and launch git commit."""
        note_file = self._get_note_file()
        if note_file is None:
            logger.info("No note files (e.g., *-notes/) staged. Nothing to do.")
            return

        note_info = self._parse_note_info(note_file=note_file)
        logger.info("Parsed note info:\n%s", note_info)

        commit_msg = self._generate_commit_message(note_file, note_info)
        logger.info("Generated commit message:")
        separator = "-" * 60
        logger.info("%s\n%s\n%s", separator, commit_msg, separator)

        try:
            temp_file = Path("GIT_COMMIT_ITEM_COMMIT_MSG_TEMP.txt")
            with temp_file.open("w", encoding="utf-8") as f:
                f.write(commit_msg + "\n")

            command = ["git", "commit", "-F", str(temp_file), "-e"]
            result = subprocess.run(command, check=False)
            if result.returncode not in (0, 1):
                raise RuntimeError(f"Git commit failed with return code {result.returncode}")
        finally:
            temp_file.unlink(missing_ok=True)

    def _get_note_file(self) -> str | None:
        """Return the relative path of the staged note file, or None if no note file is staged."""
        # Acquire all staged file paths (relative to the repository root)
        command = ["git", "diff", "--staged", "--name-only"]
        result = subprocess.run(command, capture_output=True, encoding="utf-8", check=True)

        def is_note_file(f: str) -> bool:
            return re.match(r"^.+-notes/.+\.md$", f) is not None

        staged_files = [f.strip() for f in result.stdout.splitlines()]
        return next((f for f in staged_files if is_note_file(f)), None)

    def _parse_note_info(self, note_file: str) -> dict[str, str]:
        """Parse note type, multi-level category, and title.

        Args:
            note_file(str): The relative path of the note file (e.g., "paper-notes/mlsys/distributed/gpu/Orca Paper-2022-OSDI.md").

        Returns:
            dict[str, str] with the following keys:
            - type: The note type (e.g., "paper", "tech", "doc", etc.).
            - category: The multi-level category path (e.g., "mlsys/distributed/gpu"), or an empty string if no category.
            - title: The title of the note (e.g., "Orca Paper").
            - full_title: The full title including category (e.g., "mlsys/distributed/gpu/Orca Paper" or just "Orca Paper" if no category).

        The prototypical note file path format supported is:

        - "*-notes/category/title.md"
        - "*-notes/cat1/cat2/.../title.md"
        - "*-notes/title.md"
        - "*-notes/.../title-2022.md"
        - "*-notes/.../title-2022-OSDI.md"

        Example parsing results:

        paper-notes/mlsys/distributed/gpu/Orca Paper-2022-OSDI.md
            -> type: 'paper'
            -> category: 'mlsys/distributed/gpu'
            -> title: 'Orca Paper'
            -> full_title: 'mlsys/distributed/gpu/Orca Paper'

        paper-notes/Orca Paper-2022-OSDI.md
            -> type: 'paper'
            -> category: ''
            -> title: 'Orca Paper'
            -> full_title: 'Orca Paper'

        doc-notes/TOML v1.0.0.md
            -> type: 'doc'
            -> category: ''
            -> title: 'TOML v1.0.0'
            -> full_title: 'TOML v1.0.0'

        doc-notes/python/best-practices.md
            -> type: 'doc'
            -> category: 'python'
            -> title: 'best-practices'
            -> full_title: 'python/best-practices'
        """
        # Pattern breakdown:
        # ^(.+)-notes/          --> note type (e.g., paper, tech, etc.)
        # (.+?)/                --> optional category path (with / at end), non-greedy
        # ([^/]+?)              --> title (without path separators)
        # (?:-\d{4}(?:-[A-Z]+)?)?  --> optional -YYYY or -YYYY-CONF
        # \.md$                 --> ends with .md
        #
        # We make the category part fully optional using (?:.+/)?
        match = re.match(
            r"^(.+)-notes/(?:(.+)/)?([^/]+?)(?:-\d{4}(?:-[A-Za-z]+)?)?\.md$", note_file
        )
        if match is None:
            raise ValueError(f"Invalid note file path: {note_file}")

        note_type, full_category_path, title = match.groups()

        category = full_category_path if full_category_path is not None else ""

        return {
            "type": note_type,
            "category": category,
            "title": title,
            "full_title": (f"{category}/{title}" if category else title),
        }

    def _generate_commit_message(self, note_file: str, note_info: dict[str, str]) -> str:
        """Generate commit message based on the given note file path, and its parsed info."""
        command = ["git", "cat-file", "-e", f"HEAD:{note_file}"]
        result = subprocess.run(command, capture_output=True, check=False)

        # If the note file already exists in HEAD, it's an update; otherwise, it's an addition.
        status = "update" if result.returncode == 0 else "add"

        if note_info["type"] == "doc":
            if note_info["category"]:
                return f"doc({note_info['category'].split('/')[0]}): {status} '{note_info['full_title'].partition('/')[-1]}'"
            return f"doc: {status} '{note_info['title']}'"
        return f"note({note_info['type']}): {status} '{note_info['full_title']}'"
