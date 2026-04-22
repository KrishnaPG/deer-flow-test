"""Model configuration registry with SQLite persistence."""

from __future__ import annotations

import json
import sqlite3
from pathlib import Path

from .schema import ModelConfig

# Seed data for free models
SEED_MODELS = [
    ModelConfig(
        id="gemma-free",
        name="Gemma 3 12B (Free)",
        model="openrouter/google/gemma-3-12b-it",
        provider="openrouter",
        is_free=True,
    ),
    ModelConfig(
        id="llama-free",
        name="Llama 3.1 8B (Free)",
        model="openrouter/meta-llama/llama-3.1-8b-instruct",
        provider="openrouter",
        is_free=True,
    ),
    ModelConfig(
        id="mistral-free",
        name="Mistral 7B (Free)",
        model="openrouter/mistralai/mistral-7b-instruct",
        provider="openrouter",
        is_free=True,
    ),
]


class ModelConfigRegistry:
    """In-memory cache backed by SQLite for model configurations."""

    def __init__(self, db_path: str | Path = "./models.db") -> None:
        self._db_path = Path(db_path)
        self._cache: dict[str, ModelConfig] = {}
        self._mode_defaults: dict[str, str] = {}  # mode -> model_id
        self._init_db()
        self._seed_if_empty()
        self._load_cache()

    def _init_db(self) -> None:
        """Initialize SQLite database and create tables."""
        with sqlite3.connect(self._db_path) as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS model_configs (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    model TEXT NOT NULL,
                    api_key TEXT DEFAULT '',
                    base_url TEXT DEFAULT '',
                    provider TEXT DEFAULT '',
                    is_free INTEGER DEFAULT 0,
                    metadata TEXT DEFAULT '{}',
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)
            conn.execute("""
                CREATE TABLE IF NOT EXISTS mode_defaults (
                    mode TEXT PRIMARY KEY,
                    model_id TEXT NOT NULL REFERENCES model_configs(id),
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            """)
            conn.commit()

    def _seed_if_empty(self) -> None:
        """Seed database with default free models if empty."""
        with sqlite3.connect(self._db_path) as conn:
            cursor = conn.execute("SELECT COUNT(*) FROM model_configs")
            count = cursor.fetchone()[0]
            if count == 0:
                for config in SEED_MODELS:
                    conn.execute(
                        """INSERT INTO model_configs
                           (id, name, model, api_key, base_url, provider, is_free, metadata)
                           VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
                        (
                            config.id,
                            config.name,
                            config.model,
                            config.api_key,
                            config.base_url,
                            config.provider,
                            int(config.is_free),
                            json.dumps(config.metadata),
                        ),
                    )
                conn.commit()

    def _load_cache(self) -> None:
        """Load all model configs and mode defaults into memory."""
        with sqlite3.connect(self._db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute("SELECT * FROM model_configs")
            for row in cursor:
                config = ModelConfig(
                    id=row["id"],
                    name=row["name"],
                    model=row["model"],
                    api_key=row["api_key"],
                    base_url=row["base_url"],
                    provider=row["provider"],
                    is_free=bool(row["is_free"]),
                    metadata=json.loads(row["metadata"]),
                )
                self._cache[config.id] = config

            cursor = conn.execute("SELECT mode, model_id FROM mode_defaults")
            for row in cursor:
                self._mode_defaults[row["mode"]] = row["model_id"]

    def get(self, model_id: str) -> ModelConfig | None:
        """Get model config by ID."""
        return self._cache.get(model_id)

    def list_all(self) -> list[ModelConfig]:
        """List all model configs."""
        return list(self._cache.values())

    def upsert(self, config: ModelConfig) -> None:
        """Create or update a model config."""
        with sqlite3.connect(self._db_path) as conn:
            conn.execute(
                """INSERT OR REPLACE INTO model_configs
                   (id, name, model, api_key, base_url, provider, is_free, metadata, updated_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)""",
                (
                    config.id,
                    config.name,
                    config.model,
                    config.api_key,
                    config.base_url,
                    config.provider,
                    int(config.is_free),
                    json.dumps(config.metadata),
                ),
            )
            conn.commit()
        self._cache[config.id] = config

    def delete(self, model_id: str) -> bool:
        """Delete a model config. Returns True if deleted, False if not found."""
        if model_id not in self._cache:
            return False
        with sqlite3.connect(self._db_path) as conn:
            conn.execute("DELETE FROM model_configs WHERE id = ?", (model_id,))
            conn.commit()
        del self._cache[model_id]
        # Remove from mode defaults if referenced
        modes_to_remove = [m for m, mid in self._mode_defaults.items() if mid == model_id]
        for mode in modes_to_remove:
            del self._mode_defaults[mode]
            with sqlite3.connect(self._db_path) as conn:
                conn.execute("DELETE FROM mode_defaults WHERE mode = ?", (mode,))
                conn.commit()
        return True

    def get_mode_default(self, mode: str) -> str | None:
        """Get default model ID for a mode."""
        return self._mode_defaults.get(mode)

    def set_mode_default(self, mode: str, model_id: str) -> None:
        """Set default model for a mode."""
        if model_id not in self._cache:
            raise ValueError(f"Unknown model_id: {model_id}")
        with sqlite3.connect(self._db_path) as conn:
            conn.execute(
                """INSERT OR REPLACE INTO mode_defaults (mode, model_id, updated_at)
                   VALUES (?, ?, CURRENT_TIMESTAMP)""",
                (mode, model_id),
            )
            conn.commit()
        self._mode_defaults[mode] = model_id

    def resolve_model(self, model_id: str | None, mode: str) -> ModelConfig | None:
        """Resolve model config from explicit ID or mode default.

        Resolution order:
        1. Explicit model_id (if provided)
        2. Mode default
        3. None (caller must handle)
        """
        if model_id and model_id in self._cache:
            return self._cache[model_id]
        default_id = self._mode_defaults.get(mode)
        if default_id and default_id in self._cache:
            return self._cache[default_id]
        return None

    def list_modes(self) -> dict[str, str | None]:
        """Get all modes and their default model IDs."""
        return dict(self._mode_defaults)
