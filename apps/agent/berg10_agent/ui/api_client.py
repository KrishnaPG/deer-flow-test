"""HTTP client for the FastAPI REST API endpoints."""

from __future__ import annotations

import time
from dataclasses import dataclass, field
from typing import Any

import httpx
import litellm

from berg10_agent.models import ModelConfig


@dataclass
class TestResult:
    """Result of a model connection test."""

    success: bool
    latency_ms: float = 0.0
    model: str = ""
    input_tokens: int = 0
    output_tokens: int = 0
    total_tokens: int = 0
    response_content: str = ""
    error: str = ""
    capabilities: dict[str, Any] = field(default_factory=dict)


class AgentAPIClient:
    """HTTP client for the FastAPI REST API."""

    def __init__(self, base_url: str = "http://localhost:8765") -> None:
        self._base_url = base_url.rstrip("/")
        self._client = httpx.Client(timeout=30.0)

    @property
    def base_url(self) -> str:
        return self._base_url

    def update_base_url(self, base_url: str) -> None:
        """Update the base URL."""
        self._base_url = base_url.rstrip("/")

    # --- Model Config CRUD ---

    def list_models(self) -> list[dict[str, Any]]:
        """List all configured models."""
        try:
            resp = self._client.get(f"{self._base_url}/api/models")
            resp.raise_for_status()
            return resp.json()
        except Exception as e:
            raise RuntimeError(f"Failed to list models: {e}") from e

    def get_model(self, model_id: str) -> dict[str, Any] | None:
        """Get a specific model configuration."""
        try:
            resp = self._client.get(f"{self._base_url}/api/models/{model_id}")
            if resp.status_code == 404:
                return None
            resp.raise_for_status()
            return resp.json()
        except Exception as e:
            raise RuntimeError(f"Failed to get model: {e}") from e

    def create_model(self, config: ModelConfig) -> dict[str, Any]:
        """Create or update a model configuration."""
        try:
            resp = self._client.post(
                f"{self._base_url}/api/models",
                json={
                    "id": config.id,
                    "name": config.name,
                    "model": config.model,
                    "api_key": config.api_key,
                    "base_url": config.base_url,
                    "provider": config.provider,
                    "is_free": config.is_free,
                    "metadata": config.metadata,
                },
            )
            resp.raise_for_status()
            return resp.json()
        except Exception as e:
            raise RuntimeError(f"Failed to create model: {e}") from e

    def update_model(self, config: ModelConfig) -> dict[str, Any]:
        """Update an existing model configuration."""
        try:
            resp = self._client.put(
                f"{self._base_url}/api/models/{config.id}",
                json={
                    "id": config.id,
                    "name": config.name,
                    "model": config.model,
                    "api_key": config.api_key,
                    "base_url": config.base_url,
                    "provider": config.provider,
                    "is_free": config.is_free,
                    "metadata": config.metadata,
                },
            )
            resp.raise_for_status()
            return resp.json()
        except Exception as e:
            raise RuntimeError(f"Failed to update model: {e}") from e

    def delete_model(self, model_id: str) -> bool:
        """Delete a model configuration."""
        try:
            resp = self._client.delete(f"{self._base_url}/api/models/{model_id}")
            return resp.status_code == 204
        except Exception as e:
            raise RuntimeError(f"Failed to delete model: {e}") from e

    # --- Mode Defaults ---

    def list_mode_defaults(self) -> dict[str, str | None]:
        """Get all modes and their default model IDs."""
        try:
            resp = self._client.get(f"{self._base_url}/api/models/modes/defaults")
            resp.raise_for_status()
            data = resp.json()
            return {item["mode"]: item["default_model_id"] for item in data}
        except Exception:
            return {}

    def set_mode_default(self, mode: str, model_id: str) -> None:
        """Set default model for a mode."""
        try:
            resp = self._client.put(
                f"{self._base_url}/api/models/modes/{mode}/default",
                json={"model_id": model_id},
            )
            resp.raise_for_status()
        except Exception as e:
            raise RuntimeError(f"Failed to set mode default: {e}") from e

    # --- Test Connection ---

    def test_model_connection(
        self,
        config: ModelConfig,
        timeout: int = 30,
        capabilities: dict[str, Any] | None = None,
    ) -> TestResult:
        """Test connection to a model with detailed results.

        This runs synchronously - wrap in a thread for async UI.
        """
        try:
            start = time.time()

            # Suppress LiteLLM logging for test
            litellm.suppress_debug_info = True

            response = litellm.completion(
                model=config.model,
                api_key=config.api_key if config.api_key else None,
                api_base=config.base_url if config.base_url else None,
                messages=[{"role": "user", "content": "Respond with only the word OK"}],
                max_tokens=10,
                timeout=timeout,
            )

            latency = (time.time() - start) * 1000
            choice = response.choices[0]
            usage = response.usage if hasattr(response, "usage") else None

            return TestResult(
                success=True,
                latency_ms=latency,
                model=response.model if hasattr(response, "model") else config.model,
                input_tokens=getattr(usage, "prompt_tokens", 0) if usage else 0,
                output_tokens=getattr(usage, "completion_tokens", 0) if usage else 0,
                total_tokens=getattr(usage, "total_tokens", 0) if usage else 0,
                response_content=choice.message.content or "",
                capabilities=capabilities or {},
            )

        except litellm.exceptions.AuthenticationError as e:
            return TestResult(success=False, error=f"Authentication failed: {e}")
        except litellm.exceptions.NotFoundError as e:
            return TestResult(success=False, error=f"Model not found: {e}")
        except litellm.exceptions.RateLimitError as e:
            return TestResult(success=False, error=f"Rate limit exceeded: {e}")
        except litellm.exceptions.Timeout as e:
            return TestResult(success=False, error=f"Request timed out after {timeout}s: {e}")
        except litellm.exceptions.ServiceUnavailableError as e:
            return TestResult(success=False, error=f"Service unavailable: {e}")
        except Exception as e:
            return TestResult(success=False, error=f"Unexpected error: {e}")
