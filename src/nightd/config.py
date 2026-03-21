"""Configuration management using pydantic-settings."""

from pathlib import Path

from pydantic import field_validator
from pydantic_settings import BaseSettings, SettingsConfigDict

PROJECT_ROOT = Path("/home/jehrhardt/code/nightd")


class Settings(BaseSettings):
    """Application settings loaded from environment variables and .env file."""

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
    )

    anthropic_api_key: str = ""
    database_path: str = "nightd.db"
    mlflow_tracking_uri: str = "http://localhost:5000"
    mlflow_experiment_name: str = "nightd-dev"

    @field_validator("database_path")
    @classmethod
    def resolve_database_path(cls, v: str) -> str:
        """Resolve database path to absolute path based on project root."""
        path = Path(v)
        if not path.is_absolute():
            path = PROJECT_ROOT / path
        return str(path)


settings = Settings()
