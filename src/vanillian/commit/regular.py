import datetime
import subprocess
from enum import StrEnum

__all__ = ["Frequency", "GitCommitRegularly"]


class Frequency(StrEnum):
    DAILY = "daily"
    WEEKLY = "weekly"


def _make_daily_headline(commit_day: datetime.datetime) -> str:
    """Construct daily commit headline based on the given commit day."""
    date = commit_day.strftime("%Y-%m-%d")
    weekday = commit_day.strftime("%A")
    return f"log(daily): {date} {weekday}"


def _make_weekly_headline(commit_day: datetime.datetime) -> str:
    """Construct weekly commit headline based on the given commit day."""
    day_of_month = int(commit_day.strftime("%d"))
    week_of_month = (day_of_month) // 7 + 1

    month = commit_day.strftime("%B")
    year = commit_day.strftime("%Y")
    return f"log(weekly): Week{week_of_month}-of-{month} {year}"


class GitCommitRegularly:
    def _generate_headline(self, frequency: Frequency, *, late: bool = False) -> str:
        """Generate commit headline based on given frequency and late flag."""
        if frequency not in Frequency:
            raise ValueError(f"Unsupported frequency {frequency}")

        today = datetime.datetime.today()
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

        subprocess.run(command, check=True)  # noqa: S603
