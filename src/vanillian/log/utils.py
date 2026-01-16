from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

__all__ = ["ask_for_confirm"]


def ask_for_confirm(message: str, confirm_signal: str = "") -> bool:
    received = input(message)
    return received == confirm_signal


def read_lines(file_path: Path) -> list[str]:
    with file_path.open("r", encoding="utf-8") as f:
        return f.readlines()


def write_lines_atomic(file_path: Path, lines: list[str]) -> None:
    temp_file_path = file_path.with_suffix(".tmp")
    with temp_file_path.open("w", encoding="utf-8") as f:
        f.writelines(lines)
    temp_file_path.replace(file_path)
