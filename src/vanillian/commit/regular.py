import datetime
import subprocess
from enum import StrEnum

__all__ = ["Frequency", "GitCommitRegularly"]


class Frequency(StrEnum):
    DAILY = "daily"
    WEEKLY = "weekly"


def _make_daily_headline(commit_day: datetime.date) -> str:
    """Construct daily commit headline based on the given commit day."""
    date = commit_day.strftime("%Y-%m-%d")
    weekday = commit_day.strftime("%A")
    return f"log(daily): {date} {weekday}"


def _make_weekly_headline(commit_day: datetime.date) -> str:
    """Construct weekly commit headline based on the given commit day."""
    day_of_month = commit_day.day
    week_of_month = (day_of_month - 1) // 7 + 1

    month = commit_day.strftime("%B")
    year = commit_day.year
    return f"log(weekly): Week{week_of_month}-of-{month} {year}"


class GitCommitRegularly:
    def _generate_headline(self, frequency: Frequency, *, late: bool = False) -> str:
        """Generate commit headline based on given frequency and late flag."""
        if frequency not in Frequency:
            raise ValueError(f"Unsupported frequency {frequency}")

        today = datetime.date.today()
        commit_day = today - datetime.timedelta(days=1) if late else today

        headline_makers = {
            Frequency.DAILY: _make_daily_headline,
            Frequency.WEEKLY: _make_weekly_headline,
        }

        return headline_makers[frequency](commit_day)

    def execute(self, frequency: Frequency, *, late: bool = False, edit: bool = False) -> None:
        """Generate commit headline based on given frequency and late flag, then commit changes.

        Args:
            frequency (Frequency): The frequency of the commit (daily or weekly).
            late (bool, optional): If True, generate headline as if now is one day earlier.
                Defaults to False.
            edit (bool, optional): If True, open the editor for editing the commit message before committing.
                Defaults to False.
        """
        headline = self._generate_headline(frequency, late=late)

        command = ["git", "commit", "--allow-empty"]
        if edit:
            command.append("-e")
        command.extend(["-m", headline])

        # If specify `capture_output=True`, dead lock will happen:
        # python initiates git -> git initiates editor
        # -> editor waits for user input
        # -> git waits for editor to exit -> python waits for git to exit
        # since `capture_output=True`, python captures all signals sent
        # from git, so after the git opened the editor, the editor's
        # signal of drawing a UI for user input is not sent to the terminal,
        # but block by python.
        # Therefore, the user can never interact with the editor, causing
        # git never exits, causing python never exits -> dead lock.

        # check=False to not let python capture the return code and raise exceptions
        result = subprocess.run(command)  # noqa: S603, PLW1510

        # allow the user to abort the commit (return code 1)
        if result.returncode not in (0, 1):
            raise RuntimeError(f"Git commit failed with return code {result.returncode}")
