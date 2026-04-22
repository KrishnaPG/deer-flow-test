"""Model catalog service combining LiteLLM and OpenRouter data."""

from __future__ import annotations

import time
from dataclasses import dataclass
from difflib import SequenceMatcher

import httpx
import litellm


@dataclass
class ModelInfo:
    """Model information from the catalog."""

    id: str  # e.g., "openrouter/google/gemma-3-12b-it"
    name: str  # Display name
    provider: str  # e.g., "openrouter", "openai"
    litellm_provider: str = ""  # e.g., "openrouter", "openai"
    mode: str = "chat"  # chat, embedding, image_generation, etc.
    max_input_tokens: int = 0
    max_output_tokens: int = 0
    max_tokens: int = 0
    input_cost_per_token: float = 0.0
    output_cost_per_token: float = 0.0
    supports_function_calling: bool = False
    supports_parallel_function_calling: bool = False
    supports_vision: bool = False
    supports_audio_input: bool = False
    supports_audio_output: bool = False
    supports_system_messages: bool = True
    supports_prompt_caching: bool = False
    supports_reasoning: bool = False
    supports_response_schema: bool = False
    is_free: bool = False
    description: str = ""
    architecture: str = ""  # e.g., "text+image->text"
    context_length: int = 0
    top_provider: str = ""


def _parse_litellm_models() -> list[ModelInfo]:
    """Parse models from LiteLLM's model_cost dictionary."""
    models = []
    for model_id, data in litellm.model_cost.items():
        if isinstance(data, dict):
            provider = data.get("litellm_provider", "")
            modality = data.get("mode", "chat")

            # Skip non-chat models for simplicity
            if modality not in ("chat", "completion"):
                continue

            max_input = data.get("max_input_tokens", 0) or data.get("max_tokens", 0)
            max_output = data.get("max_output_tokens", 0) or data.get("max_tokens", 0)

            models.append(
                ModelInfo(
                    id=model_id,
                    name=model_id.replace("/", " / ").replace("-", " ").title(),
                    provider=provider,
                    litellm_provider=provider,
                    mode=modality,
                    max_input_tokens=max_input,
                    max_output_tokens=max_output,
                    max_tokens=data.get("max_tokens", 0),
                    input_cost_per_token=data.get("input_cost_per_token", 0.0),
                    output_cost_per_token=data.get("output_cost_per_token", 0.0),
                    supports_function_calling=data.get("supports_function_calling", False),
                    supports_parallel_function_calling=data.get(
                        "supports_parallel_function_calling", False
                    ),
                    supports_vision=data.get("supports_vision", False),
                    supports_audio_input=data.get("supports_audio_input", False),
                    supports_audio_output=data.get("supports_audio_output", False),
                    supports_system_messages=data.get("supports_system_messages", True),
                    supports_prompt_caching=data.get("supports_prompt_caching", False),
                    supports_reasoning=data.get("supports_reasoning", False),
                    supports_response_schema=data.get("supports_response_schema", False),
                    is_free=(
                        data.get("input_cost_per_token", 0) == 0
                        and data.get("output_cost_per_token", 0) == 0
                    ),
                )
            )
    return models


def _fetch_openrouter_models() -> list[ModelInfo]:
    """Fetch models from OpenRouter API (no auth required)."""
    try:
        resp = httpx.get(
            "https://openrouter.ai/api/v1/models",
            timeout=30.0,
        )
        resp.raise_for_status()
        data = resp.json()
    except Exception:
        return []

    models = []
    for item in data.get("data", []):
        model_id = item.get("id", "")
        pricing = item.get("pricing", {})
        architecture = item.get("architecture", {})
        context_length = item.get("context_length", 0)
        max_completion = item.get("max_completion_tokens", 0)

        input_cost = float(pricing.get("prompt", "0"))
        output_cost = float(pricing.get("completion", "0"))

        # Parse modality
        modality = architecture.get("modality", "text->text")
        is_vision = "image" in modality if modality else False

        models.append(
            ModelInfo(
                id=f"openrouter/{model_id}",
                name=item.get("name", model_id),
                provider="openrouter",
                litellm_provider="openrouter",
                mode="chat",
                max_input_tokens=context_length,
                max_output_tokens=max_completion,
                input_cost_per_token=input_cost,
                output_cost_per_token=output_cost,
                supports_vision=is_vision,
                supports_function_calling=True,  # Most OpenRouter models support this
                supports_system_messages=True,
                is_free=(input_cost == 0 and output_cost == 0),
                description=item.get("description", ""),
                architecture=modality,
                context_length=context_length,
                top_provider=item.get("top_provider", ""),
            )
        )
    return models


class ModelCatalog:
    """Model catalog combining LiteLLM and OpenRouter sources."""

    def __init__(self) -> None:
        self._cache: list[ModelInfo] = []
        self._last_refresh: float = 0.0
        self._error: str = ""

    @property
    def is_cached(self) -> bool:
        """Check if catalog has been loaded."""
        return bool(self._cache)

    @property
    def last_refresh_time(self) -> float:
        """Get last refresh timestamp."""
        return self._last_refresh

    @property
    def error(self) -> str:
        """Get last error message."""
        return self._error

    def refresh(self) -> None:
        """Fetch and merge LiteLLM + OpenRouter models."""
        self._error = ""
        try:
            litellm_models = _parse_litellm_models()
            openrouter_models = _fetch_openrouter_models()

            # Merge: use OpenRouter data where available, LiteLLM as base
            merged: dict[str, ModelInfo] = {}
            for m in litellm_models:
                merged[m.id] = m
            for m in openrouter_models:
                # Prefer OpenRouter data if model exists in both
                if m.id in merged:
                    existing = merged[m.id]
                    # Enrich with OpenRouter data
                    existing.description = m.description or existing.description
                    existing.architecture = m.architecture or existing.architecture
                    existing.context_length = m.context_length or existing.context_length
                    existing.top_provider = m.top_provider or existing.top_provider
                else:
                    merged[m.id] = m

            self._cache = list(merged.values())
            self._last_refresh = time.time()
        except Exception as e:
            self._error = str(e)

    def get_cached(self) -> list[ModelInfo]:
        """Return cached models."""
        return self._cache

    def search(self, query: str, threshold: float = 0.3) -> list[ModelInfo]:
        """Fuzzy search across model name, id, provider, and description."""
        if not query or not self._cache:
            return self._cache

        query_lower = query.lower()
        scored: list[tuple[float, ModelInfo]] = []

        for m in self._cache:
            search_text = f"{m.id} {m.name} {m.provider} {m.description}".lower()
            score = max(
                SequenceMatcher(None, query_lower, m.id.lower()).ratio(),
                SequenceMatcher(None, query_lower, m.name.lower()).ratio(),
                SequenceMatcher(None, query_lower, m.provider.lower()).ratio(),
                SequenceMatcher(None, query_lower, search_text).ratio() * 0.5,
            )
            if score >= threshold:
                scored.append((score, m))

        scored.sort(key=lambda x: x[0], reverse=True)
        return [m for _, m in scored]

    def filter_models(
        self,
        models: list[ModelInfo],
        free_only: bool = False,
        function_calling: bool = False,
        vision: bool = False,
        provider: str = "",
    ) -> list[ModelInfo]:
        """Apply filters to model list."""
        result = models

        if free_only:
            result = [m for m in result if m.is_free]
        if function_calling:
            result = [m for m in result if m.supports_function_calling]
        if vision:
            result = [m for m in result if m.supports_vision]
        if provider:
            result = [m for m in result if m.provider == provider.lower()]

        return result

    def get_providers(self) -> list[str]:
        """Get unique provider names from cached models."""
        return sorted({m.provider for m in self._cache if m.provider})
