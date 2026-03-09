"""
Configuration management module for Twinkle Linux.

This module provides the ConfigurationManager class for handling application
configuration, including loading, saving, and validation of configuration data.
"""

import json
import logging
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Any, Dict, Optional

from pydantic import BaseModel, Field, field_validator


logger = logging.getLogger(__name__)


class MonitorConfig(BaseModel):
    """Configuration for a specific monitor."""

    name: str = "Unknown Monitor"
    bus: Optional[str] = None
    preferred_brightness: int = Field(default=75, ge=0, le=100)
    brightness_presets: Dict[str, int] = Field(
        default_factory=lambda: {"day": 80, "night": 30, "gaming": 100}
    )
    enabled_vcp_codes: list[str] = Field(default_factory=lambda: ["0x10", "0x12", "0x14"])

    @field_validator("brightness_presets")
    @classmethod
    def validate_brightness_presets(cls, v: Dict[str, int]) -> Dict[str, int]:
        """Validate that all brightness preset values are in valid range."""
        for key, value in v.items():
            if not 0 <= value <= 100:
                raise ValueError(f"Brightness preset '{key}' must be between 0 and 100")
        return v


class UIConfig(BaseModel):
    """UI-related configuration settings."""

    show_brightness_in_tray: bool = True
    tray_icon_style: str = Field(default="percentage", pattern="^(percentage|icon|both)$")
    slider_position: str = Field(default="follow_cursor", pattern="^(follow_cursor|center|top_left)$")
    theme: str = Field(default="system", pattern="^(system|light|dark)$")
    enabled_vcp_codes: list[int] = Field(
        default_factory=lambda: [0x10, 0x12, 0x14, 0x60, 0x62]
    )


class BehaviorConfig(BaseModel):
    """Application behavior configuration."""

    auto_start: bool = True
    minimize_to_tray: bool = True
    brightness_step: int = Field(default=5, ge=1, le=50)
    response_timeout_ms: int = Field(default=500, ge=100, le=5000)
    max_retries: int = Field(default=3, ge=0, le=10)


class ShortcutsConfig(BaseModel):
    """Keyboard shortcuts configuration."""

    increase_brightness: str = "Ctrl+Alt+Up"
    decrease_brightness: str = "Ctrl+Alt+Down"


class AppConfig(BaseModel):
    """Main application configuration model."""

    version: str = "1.0"
    monitors: Dict[str, MonitorConfig] = Field(default_factory=dict)
    ui: UIConfig = Field(default_factory=UIConfig)
    behavior: BehaviorConfig = Field(default_factory=BehaviorConfig)
    shortcuts: ShortcutsConfig = Field(default_factory=ShortcutsConfig)

    @field_validator("version")
    @classmethod
    def validate_version(cls, v: str) -> str:
        """Validate configuration version format."""
        if not v:
            raise ValueError("Configuration version cannot be empty")
        return v


class ConfigurationManager:
    """
    Manages application configuration loading, saving, and validation.

    This class provides methods to load configuration from JSON files,
    save configuration to JSON files, and access configuration values.
    """

    def __init__(self, config_path: Optional[Path] = None) -> None:
        """
        Initialize the ConfigurationManager.

        Args:
            config_path: Path to the configuration file. If None, uses default
                        location in XDG config directory.
        """
        if config_path is None:
            config_path = self._get_default_config_path()

        self._config_path: Path = config_path
        self._config: Optional[AppConfig] = None

    @staticmethod
    def _get_default_config_path() -> Path:
        """
        Get the default configuration file path.

        Returns:
            Path to the default configuration file.
        """
        try:
            from xdg_base_dirs import xdg_config_home

            config_dir = xdg_config_home() / "twinkle-linux"
        except ImportError:
            # Fallback to ~/.config if pyxdg is not available
            config_dir = Path.home() / ".config" / "twinkle-linux"

        config_dir.mkdir(parents=True, exist_ok=True)
        return config_dir / "config.json"

    @property
    def config_path(self) -> Path:
        """Get the path to the configuration file."""
        return self._config_path

    def load(self) -> AppConfig:
        """
        Load configuration from file.

        If the configuration file doesn't exist, creates a default configuration
        and saves it to the file.

        Returns:
            The loaded or default AppConfig instance.
        """
        if not self._config_path.exists():
            logger.info("Configuration file not found, creating default configuration")
            self._config = self._get_default_config()
            self.save()
            return self._config

        try:
            with open(self._config_path, "r", encoding="utf-8") as f:
                config_data = json.load(f)

            self._config = AppConfig.model_validate(config_data)
            logger.info(f"Configuration loaded from {self._config_path}")
            return self._config

        except json.JSONDecodeError as e:
            logger.error(f"Failed to parse configuration file: {e}")
            logger.info("Falling back to default configuration")
            self._config = self._get_default_config()
            return self._config

        except Exception as e:
            logger.error(f"Failed to load configuration: {e}")
            logger.info("Falling back to default configuration")
            self._config = self._get_default_config()
            return self._config

    def save(self) -> None:
        """
        Save current configuration to file.

        Raises:
            RuntimeError: If no configuration is loaded.
        """
        if self._config is None:
            raise RuntimeError("No configuration loaded. Call load() first.")

        try:
            self._config_path.parent.mkdir(parents=True, exist_ok=True)

            with open(self._config_path, "w", encoding="utf-8") as f:
                json.dump(
                    self._config.model_dump(mode="json"),
                    f,
                    indent=2,
                    ensure_ascii=False,
                )

            logger.info(f"Configuration saved to {self._config_path}")

        except Exception as e:
            logger.error(f"Failed to save configuration: {e}")
            raise

    def get(self) -> AppConfig:
        """
        Get the current configuration.

        Returns:
            The current AppConfig instance.

        Raises:
            RuntimeError: If no configuration is loaded.
        """
        if self._config is None:
            self._config = self.load()
        return self._config

    def update(self, **kwargs: Any) -> None:
        """
        Update configuration values.

        Args:
            **kwargs: Configuration key-value pairs to update.

        Raises:
            RuntimeError: If no configuration is loaded.
        """
        if self._config is None:
            self._config = self.load()

        config_dict = self._config.model_dump(mode="json")

        def update_nested(d: Dict[str, Any], keys: list[str], value: Any) -> None:
            """Recursively update nested dictionary."""
            if len(keys) == 1:
                d[keys[0]] = value
            else:
                update_nested(d[keys[0]], keys[1:], value)

        for key, value in kwargs.items():
            keys = key.split(".")
            update_nested(config_dict, keys, value)

        self._config = AppConfig.model_validate(config_dict)

    def reload(self) -> AppConfig:
        """
        Reload configuration from file.

        Returns:
            The reloaded AppConfig instance.
        """
        self._config = None
        return self.load()

    def _get_default_config(self) -> AppConfig:
        """
        Get the default configuration.

        Returns:
            Default AppConfig instance.
        """
        return AppConfig()

    def reset_to_defaults(self) -> None:
        """Reset configuration to default values and save."""
        self._config = self._get_default_config()
        self.save()
        logger.info("Configuration reset to defaults")
